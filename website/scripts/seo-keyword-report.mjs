import path from 'node:path';
import {
  canonicalToPath,
  loadStructuredRows,
  normalizeKeyword,
  normalizeUrlPath,
  parseArgs,
  parseInteger,
  parseNumber,
  readJsonFile,
  writeTextFile
} from './lib/seo-utils.mjs';

const KEYWORDS_PATH = 'seo/keywords.json';
const CONTENT_REGISTRY_PATH = 'seo/content-registry.json';
const DEFAULT_SEARCH_CONSOLE_PATH = 'seo/metrics/search_console_latest.csv';
const DEFAULT_KEYWORD_VOLUME_PATH = 'seo/metrics/keyword_volume_latest.csv';
const OUTPUT_DIR = 'reports';

function findFirstValue(row, keys) {
  for (const key of keys) {
    if (row[key] !== undefined && row[key] !== null && row[key] !== '') {
      return row[key];
    }
  }
  return '';
}

function aggregateSearchConsoleRows(rawRows) {
  const grouped = new Map();

  for (const row of rawRows) {
    const keyword = normalizeKeyword(findFirstValue(row, ['query', 'keyword']));
    if (!keyword) {
      continue;
    }

    const pagePath = normalizeUrlPath(findFirstValue(row, ['page', 'url', 'target_url']));
    const impressions = parseInteger(findFirstValue(row, ['impressions']), 0);
    const clicks = parseInteger(findFirstValue(row, ['clicks']), 0);
    const ctrValue = parseNumber(findFirstValue(row, ['ctr']), NaN);
    const ctr = Number.isFinite(ctrValue) ? ctrValue : impressions > 0 ? clicks / impressions : 0;
    const position = parseNumber(findFirstValue(row, ['position', 'avg_position']), NaN);

    if (!grouped.has(keyword)) {
      grouped.set(keyword, {
        actual_urls: new Map(),
        clicks: 0,
        impressions: 0,
        weighted_position_total: 0,
        weighted_position_impressions: 0
      });
    }

    const aggregate = grouped.get(keyword);
    aggregate.clicks += clicks;
    aggregate.impressions += impressions;

    if (Number.isFinite(position)) {
      aggregate.weighted_position_total += position * Math.max(impressions, 1);
      aggregate.weighted_position_impressions += Math.max(impressions, 1);
    }

    if (pagePath) {
      const existingPageMetrics = aggregate.actual_urls.get(pagePath) || { clicks: 0, impressions: 0, ctr: 0 };
      const nextImpressions = existingPageMetrics.impressions + impressions;
      const nextClicks = existingPageMetrics.clicks + clicks;
      aggregate.actual_urls.set(pagePath, {
        clicks: nextClicks,
        impressions: nextImpressions,
        ctr: nextImpressions > 0 ? nextClicks / nextImpressions : ctr
      });
    }
  }

  return grouped;
}

function aggregateVolumeRows(rawRows) {
  const grouped = new Map();

  for (const row of rawRows) {
    const keyword = normalizeKeyword(findFirstValue(row, ['keyword', 'query']));
    if (!keyword) {
      continue;
    }

    const volume = parseInteger(findFirstValue(row, ['search_volume', 'avg_monthly_searches', 'volume']), 0);
    const current = grouped.get(keyword);

    if (!current || volume > current.search_volume) {
      grouped.set(keyword, {
        search_volume: volume,
        country: findFirstValue(row, ['country']),
        language: findFirstValue(row, ['language']),
        source: findFirstValue(row, ['source'])
      });
    }
  }

  return grouped;
}

function priorityRank(priority) {
  switch ((priority || '').toLowerCase()) {
    case 'high':
      return 0;
    case 'medium':
      return 1;
    default:
      return 2;
  }
}

function chooseTopPage(actualUrlMetrics) {
  return [...actualUrlMetrics.entries()].sort((left, right) => right[1].impressions - left[1].impressions)[0]?.[0] || '';
}

function buildRecommendation(row) {
  if (row.target_page === '(unassigned)') {
    return 'Create or assign a target page';
  }

  if (row.cannibalization) {
    return 'Consolidate or retarget competing pages';
  }

  if (!row.has_volume_data) {
    return 'Collect search volume data';
  }

  if (row.impressions >= 100 && row.ctr < 0.03) {
    return 'Test title and meta for CTR';
  }

  if (row.avg_position >= 4 && row.avg_position <= 20) {
    return 'Refresh on-page content and internal links';
  }

  if (row.avg_position > 20 && row.impressions >= 50) {
    return 'Expand content depth and supporting links';
  }

  if (row.clicks === 0 && row.impressions === 0) {
    return 'No ranking data yet';
  }

  return 'Monitor';
}

function formatDecimal(value, digits = 1) {
  if (!Number.isFinite(value)) {
    return '-';
  }
  return value.toFixed(digits);
}

function formatPercent(value) {
  if (!Number.isFinite(value)) {
    return '-';
  }
  return `${(value * 100).toFixed(1)}%`;
}

function buildMarkdownReport({
  generatedAt,
  dataMode,
  searchConsolePath,
  keywordVolumePath,
  rows,
  missingTargetPages,
  noVolumeData,
  rankingOpportunities,
  highImpressionLowCtr,
  cannibalizationCandidates
}) {
  const lines = [
    '# SEO Keyword Report',
    '',
    `Generated at: ${generatedAt}`,
    `Search Console input: \`${searchConsolePath}\``,
    `Keyword volume input: \`${keywordVolumePath}\``,
    `Data mode: ${dataMode}`,
    '',
    '## Keyword Table',
    '',
    '| Keyword | Target page | Avg position | Impressions | Clicks | CTR | Search volume | Status / action |',
    '| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |'
  ];

  for (const row of rows) {
    lines.push(
      `| ${row.keyword} | ${row.target_page} | ${formatDecimal(row.avg_position)} | ${row.impressions} | ${row.clicks} | ${formatPercent(row.ctr)} | ${row.search_volume_display} | ${row.recommended_action} |`
    );
  }

  const renderKeywordList = (items, formatter) => {
    if (items.length === 0) {
      return ['- (none)'];
    }
    return items.map(formatter);
  };

  lines.push('');
  lines.push('## Missing Target Pages');
  lines.push('');
  lines.push(...renderKeywordList(missingTargetPages, (row) => `- ${row.keyword}: ${row.recommended_action}`));

  lines.push('');
  lines.push('## Keywords With No Volume Data');
  lines.push('');
  lines.push(...renderKeywordList(noVolumeData, (row) => `- ${row.keyword}: add search volume before making priority decisions.`));

  lines.push('');
  lines.push('## Keywords Ranking In Positions 4-20');
  lines.push('');
  lines.push(
    ...renderKeywordList(
      rankingOpportunities,
      (row) => `- ${row.keyword}: avg position ${formatDecimal(row.avg_position)}, target ${row.target_page}.`
    )
  );

  lines.push('');
  lines.push('## High-Impression Low-CTR Pages');
  lines.push('');
  lines.push(
    ...renderKeywordList(
      highImpressionLowCtr,
      (row) => `- ${row.keyword}: ${row.impressions} impressions, CTR ${formatPercent(row.ctr)}, target ${row.target_page}.`
    )
  );

  lines.push('');
  lines.push('## Likely Keyword Cannibalization Candidates');
  lines.push('');
  lines.push(
    ...renderKeywordList(
      cannibalizationCandidates,
      (row) =>
        `- ${row.keyword}: tracked target ${row.target_page}, ranking pages ${row.actual_pages.join(', ')}.`
    )
  );

  lines.push('');
  return `${lines.join('\n')}\n`;
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const searchConsolePath = args['search-console'] || DEFAULT_SEARCH_CONSOLE_PATH;
  const keywordVolumePath = args['keyword-volume'] || DEFAULT_KEYWORD_VOLUME_PATH;

  const searchConsoleSource = loadStructuredRows(searchConsolePath);
  const keywordVolumeSource = loadStructuredRows(keywordVolumePath);

  const inventory = readJsonFile(KEYWORDS_PATH).keywords || [];
  const pages = readJsonFile(CONTENT_REGISTRY_PATH).pages || [];
  const pageByPath = new Map(pages.map((page) => [page.url_path, page]));

  const searchGroups = aggregateSearchConsoleRows(searchConsoleSource.rows);
  const volumeGroups = aggregateVolumeRows(keywordVolumeSource.rows);
  const generatedAt = new Date().toISOString();
  const reportDate = generatedAt.slice(0, 10);
  const dataMode =
    searchConsolePath.includes('/fixtures/') || keywordVolumePath.includes('/fixtures/')
      ? 'fixture'
      : searchConsoleSource.format === 'missing' || keywordVolumeSource.format === 'missing'
        ? 'missing-inputs'
        : 'live-or-local';

  const rows = inventory.map((keywordRow) => {
    const keywordKey = normalizeKeyword(keywordRow.keyword);
    const searchMetrics = searchGroups.get(keywordKey);
    const volumeMetrics = volumeGroups.get(keywordKey);
    const targetPath = canonicalToPath(keywordRow.target_url);
    const targetPage = pageByPath.get(targetPath);
    const actualPages = searchMetrics ? [...searchMetrics.actual_urls.keys()] : [];
    const topRankingPage = searchMetrics ? chooseTopPage(searchMetrics.actual_urls) : '';
    const avgPosition =
      searchMetrics && searchMetrics.weighted_position_impressions > 0
        ? searchMetrics.weighted_position_total / searchMetrics.weighted_position_impressions
        : NaN;
    const impressions = searchMetrics?.impressions || 0;
    const clicks = searchMetrics?.clicks || 0;
    const ctr = impressions > 0 ? clicks / impressions : 0;
    const cannibalization =
      actualPages.length > 1 || (topRankingPage && targetPath && topRankingPage !== targetPath);

    const row = {
      keyword: keywordRow.keyword,
      cluster: keywordRow.cluster,
      priority: keywordRow.priority,
      target_page: targetPath || '(unassigned)',
      target_page_title: targetPage?.title || '',
      avg_position: avgPosition,
      impressions,
      clicks,
      ctr,
      search_volume: volumeMetrics?.search_volume ?? null,
      search_volume_display: volumeMetrics?.search_volume ?? '-',
      has_volume_data: Boolean(volumeMetrics && Number.isFinite(volumeMetrics.search_volume) && volumeMetrics.search_volume > 0),
      actual_pages: actualPages,
      top_ranking_page: topRankingPage,
      cannibalization,
      recommended_action: '',
      notes: keywordRow.notes
    };

    row.recommended_action = buildRecommendation(row);
    return row;
  });

  rows.sort((left, right) => {
    const priorityDelta = priorityRank(left.priority) - priorityRank(right.priority);
    if (priorityDelta !== 0) {
      return priorityDelta;
    }
    return right.impressions - left.impressions;
  });

  const missingTargetPages = rows.filter((row) => row.target_page === '(unassigned)');
  const noVolumeData = rows.filter((row) => !row.has_volume_data);
  const rankingOpportunities = rows.filter((row) => Number.isFinite(row.avg_position) && row.avg_position >= 4 && row.avg_position <= 20);
  const highImpressionLowCtr = rows.filter((row) => row.impressions >= 100 && row.ctr < 0.03);
  const cannibalizationCandidates = rows.filter((row) => row.cannibalization);

  const reportPayload = {
    generated_at: generatedAt,
    data_mode: dataMode,
    search_console_input: path.resolve(process.cwd(), searchConsolePath),
    keyword_volume_input: path.resolve(process.cwd(), keywordVolumePath),
    summary: {
      tracked_keywords: rows.length,
      missing_target_pages: missingTargetPages.length,
      missing_volume_data: noVolumeData.length,
      ranking_opportunities: rankingOpportunities.length,
      high_impression_low_ctr: highImpressionLowCtr.length,
      cannibalization_candidates: cannibalizationCandidates.length
    },
    rows,
    sections: {
      missing_target_pages: missingTargetPages,
      no_volume_data: noVolumeData,
      ranking_opportunities: rankingOpportunities,
      high_impression_low_ctr: highImpressionLowCtr,
      cannibalization_candidates: cannibalizationCandidates
    }
  };

  const markdown = buildMarkdownReport({
    generatedAt,
    dataMode,
    searchConsolePath,
    keywordVolumePath,
    rows,
    missingTargetPages,
    noVolumeData,
    rankingOpportunities,
    highImpressionLowCtr,
    cannibalizationCandidates
  });

  const markdownPath = `${OUTPUT_DIR}/seo-keyword-report-${reportDate}.md`;
  const jsonPath = `${OUTPUT_DIR}/seo-keyword-report-${reportDate}.json`;

  writeTextFile(markdownPath, markdown);
  writeTextFile(jsonPath, `${JSON.stringify(reportPayload, null, 2)}\n`);

  console.log(`Wrote ${markdownPath}`);
  console.log(`Wrote ${jsonPath}`);
}

main();
