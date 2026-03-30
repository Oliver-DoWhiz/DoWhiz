import {
  canonicalToPath,
  escapeHtml,
  escapeXml,
  fileExists,
  formatDisplayDate,
  normalizeKeyword,
  readJsonFile,
  readTextFile,
  replaceBetweenMarkers,
  stripDoWhizSuffix,
  writeTextFile
} from './lib/seo-utils.mjs';

const CONTENT_REGISTRY_PATH = 'seo/content-registry.json';
const KEYWORDS_PATH = 'seo/keywords.json';
const BLOG_INDEX_PATH = 'public/blog/index.html';
const SITEMAP_PATH = 'public/sitemap.xml';
const CONTENT_EXPORT_PATH = 'seo/exports/content-inventory.generated.json';
const KEYWORD_EXPORT_PATH = 'seo/exports/keyword-target-map.generated.json';

const BLOG_INDEX_LINKS_START = '<!-- SEO_GENERATED_BLOG_LINKS_START -->';
const BLOG_INDEX_LINKS_END = '<!-- SEO_GENERATED_BLOG_LINKS_END -->';
const BLOG_INDEX_CARDS_START = '<!-- SEO_GENERATED_BLOG_CARDS_START -->';
const BLOG_INDEX_CARDS_END = '<!-- SEO_GENERATED_BLOG_CARDS_END -->';

function sortPages(a, b) {
  return (a.sort_order ?? 0) - (b.sort_order ?? 0);
}

function sortBlogPosts(a, b) {
  const publishedDelta = (b.published_at || '').localeCompare(a.published_at || '');
  if (publishedDelta !== 0) {
    return publishedDelta;
  }

  return sortPages(a, b);
}

function validateRegistry(pages, keywords) {
  if (!Array.isArray(pages) || pages.length === 0) {
    throw new Error('SEO content registry is empty.');
  }

  const seenIds = new Set();
  const seenPaths = new Set();
  const seenCanonicals = new Set();

  for (const page of pages) {
    if (!page.id || !page.slug || !page.url_path || !page.canonical_url || !page.file_path) {
      throw new Error(`Registry page is missing required identity fields: ${JSON.stringify(page, null, 2)}`);
    }

    if (!fileExists(page.file_path)) {
      throw new Error(`Registry page file is missing: ${page.file_path}`);
    }

    if (seenIds.has(page.id)) {
      throw new Error(`Duplicate registry page id: ${page.id}`);
    }
    seenIds.add(page.id);

    if (seenPaths.has(page.url_path)) {
      throw new Error(`Duplicate registry url_path: ${page.url_path}`);
    }
    seenPaths.add(page.url_path);

    if (seenCanonicals.has(page.canonical_url)) {
      throw new Error(`Duplicate registry canonical_url: ${page.canonical_url}`);
    }
    seenCanonicals.add(page.canonical_url);

    if (!page.title || !page.description || !page.primary_keyword || !page.lastmod || !page.status) {
      throw new Error(`Registry page is missing required metadata: ${page.id}`);
    }

    if (page.include_in_blog_index) {
      const requiredBlogFields = ['blog_tag', 'blog_index_title', 'blog_related_description', 'blog_summary'];
      const missingFields = requiredBlogFields.filter((field) => !page[field]);
      if (missingFields.length > 0) {
        throw new Error(`Registry blog page ${page.id} is missing blog index fields: ${missingFields.join(', ')}`);
      }

      if (!Array.isArray(page.blog_highlights) || page.blog_highlights.length === 0) {
        throw new Error(`Registry blog page ${page.id} is missing blog_highlights.`);
      }
    }
  }

  const pageByCanonical = new Map(pages.map((page) => [page.canonical_url, page]));
  for (const keyword of keywords) {
    if (!keyword.keyword || !keyword.cluster || !keyword.intent || !keyword.language || !keyword.country) {
      throw new Error(`Keyword inventory row is missing required fields: ${JSON.stringify(keyword, null, 2)}`);
    }

    if (keyword.target_url && !pageByCanonical.has(keyword.target_url)) {
      throw new Error(`Keyword target_url is not in content registry: ${keyword.keyword} -> ${keyword.target_url}`);
    }
  }
}

function buildSitemap(pages) {
  const sitemapPages = [...pages]
    .filter((page) => page.include_in_sitemap && page.status === 'published')
    .sort(sortPages);

  const urlEntries = sitemapPages.map((page) => {
    const lines = [
      '  <url>',
      `    <loc>${escapeXml(page.canonical_url)}</loc>`
    ];

    if (page.lastmod) {
      lines.push(`    <lastmod>${escapeXml(page.lastmod)}</lastmod>`);
    }

    lines.push('  </url>');
    return lines.join('\n');
  });

  return ['<?xml version="1.0" encoding="UTF-8"?>', '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">', ...urlEntries, '</urlset>', ''].join('\n');
}

function renderBlogIndexLinks(blogPosts) {
  return blogPosts
    .map((post) => {
      return [
        '            <a class="related-item" href="' + escapeHtml(post.url_path) + '">',
        '              <span class="related-item-title">' + escapeHtml(post.blog_index_title || stripDoWhizSuffix(post.title)) + '</span>',
        '              <span class="related-item-desc">' + escapeHtml(post.blog_related_description) + '</span>',
        '            </a>'
      ].join('\n');
    })
    .join('\n');
}

function renderBlogIndexCards(blogPosts) {
  const featuredId = blogPosts.find((post) => post.featured_on_blog_index)?.id;

  return blogPosts
    .map((post) => {
      const classes = ['blog-card'];
      if (post.id === featuredId) {
        classes.push('featured');
      }

      const highlights = (post.blog_highlights || [])
        .map((item) => `              <li>${escapeHtml(item)}</li>`)
        .join('\n');

      return [
        `          <article class="${classes.join(' ')}" id="${escapeHtml(post.slug)}">`,
        '            <div class="blog-meta">',
        `              <span class="blog-tag">${escapeHtml(post.blog_tag)}</span>`,
        `              <span class="blog-date">${escapeHtml(formatDisplayDate(post.published_at))}</span>`,
        '            </div>',
        `            <h2>${escapeHtml(post.blog_index_title || stripDoWhizSuffix(post.title))}</h2>`,
        '            <p>',
        `              ${escapeHtml(post.blog_summary)}`,
        '            </p>',
        '            <ul>',
        highlights,
        '            </ul>',
        `            <a class="back-link" href="${escapeHtml(post.url_path)}">Read article</a>`,
        '          </article>'
      ].join('\n');
    })
    .join('\n\n');
}

function buildContentExport(pages, keywords, registryUpdatedAt) {
  const trackedKeywordsByPath = new Map();
  for (const keyword of keywords) {
    const targetPath = canonicalToPath(keyword.target_url);
    if (!targetPath) {
      continue;
    }

    if (!trackedKeywordsByPath.has(targetPath)) {
      trackedKeywordsByPath.set(targetPath, []);
    }

    trackedKeywordsByPath.get(targetPath).push({
      keyword: keyword.keyword,
      cluster: keyword.cluster,
      intent: keyword.intent,
      priority: keyword.priority,
      status: keyword.status
    });
  }

  const payload = {
    generated_from_registry_updated_at: registryUpdatedAt,
    total_pages: pages.length,
    sitemap_pages: pages.filter((page) => page.include_in_sitemap && page.status === 'published').length,
    pages: [...pages]
      .sort(sortPages)
      .map((page) => ({
        ...page,
        tracked_keywords: trackedKeywordsByPath.get(page.url_path) || []
      }))
  };

  return `${JSON.stringify(payload, null, 2)}\n`;
}

function buildKeywordExport(keywords, pages, inventoryUpdatedAt) {
  const pagesByCanonical = new Map(pages.map((page) => [page.canonical_url, page]));
  const payload = {
    generated_from_keyword_inventory_updated_at: inventoryUpdatedAt,
    total_keywords: keywords.length,
    keywords: keywords.map((keyword) => {
      const targetPage = keyword.target_url ? pagesByCanonical.get(keyword.target_url) : null;
      return {
        ...keyword,
        target_page_id: targetPage?.id || null,
        target_page_path: targetPage?.url_path || null,
        target_primary_keyword: targetPage?.primary_keyword || null,
        normalized_keyword: normalizeKeyword(keyword.keyword)
      };
    })
  };

  return `${JSON.stringify(payload, null, 2)}\n`;
}

function main() {
  const registry = readJsonFile(CONTENT_REGISTRY_PATH);
  const keywordInventory = readJsonFile(KEYWORDS_PATH);
  const pages = registry.pages || [];
  const keywords = keywordInventory.keywords || [];

  validateRegistry(pages, keywords);

  const sitemap = buildSitemap(pages);
  const blogIndexTemplate = readTextFile(BLOG_INDEX_PATH);
  const blogPosts = pages
    .filter((page) => page.page_type === 'blog_post' && page.include_in_blog_index && page.status === 'published')
    .sort(sortBlogPosts);

  const updatedBlogIndex = replaceBetweenMarkers(
    replaceBetweenMarkers(blogIndexTemplate, BLOG_INDEX_LINKS_START, BLOG_INDEX_LINKS_END, renderBlogIndexLinks(blogPosts)),
    BLOG_INDEX_CARDS_START,
    BLOG_INDEX_CARDS_END,
    renderBlogIndexCards(blogPosts)
  );

  const filesWritten = [];

  if (writeTextFile(SITEMAP_PATH, sitemap)) {
    filesWritten.push(SITEMAP_PATH);
  }

  if (writeTextFile(BLOG_INDEX_PATH, updatedBlogIndex)) {
    filesWritten.push(BLOG_INDEX_PATH);
  }

  if (writeTextFile(CONTENT_EXPORT_PATH, buildContentExport(pages, keywords, registry.updated_at || null))) {
    filesWritten.push(CONTENT_EXPORT_PATH);
  }

  if (writeTextFile(KEYWORD_EXPORT_PATH, buildKeywordExport(keywords, pages, keywordInventory.updated_at || null))) {
    filesWritten.push(KEYWORD_EXPORT_PATH);
  }

  if (filesWritten.length === 0) {
    console.log('SEO artifacts are already up to date.');
    return;
  }

  console.log('Updated SEO artifacts:');
  for (const file of filesWritten) {
    console.log(`- ${file}`);
  }
}

main();
