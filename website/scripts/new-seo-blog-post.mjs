import fs from 'node:fs';
import path from 'node:path';
import { formatDisplayDate, parseArgs, readTextFile } from './lib/seo-utils.mjs';

const args = parseArgs(process.argv.slice(2));

const requiredArgs = ['slug', 'headline', 'description', 'owner'];
const missingArgs = requiredArgs.filter((name) => !args[name]);

if (missingArgs.length > 0) {
  console.error(`Missing required args: ${missingArgs.join(', ')}`);
  console.error(
    'Usage: npm run seo:new-blog -- --slug my-post --headline "My headline" --description "Meta description" --owner Oliver [--primary-keyword "keyword"] [--date 2026-03-30] [--eyebrow SEO]'
  );
  process.exit(1);
}

const slug = args.slug.trim();
const headline = args.headline.trim();
const description = args.description.trim();
const owner = args.owner.trim();
const primaryKeyword = (args['primary-keyword'] || headline).trim();
const isoDate = (args.date || new Date().toISOString().slice(0, 10)).trim();
const eyebrow = (args.eyebrow || 'SEO').trim();
const lead = (
  args.lead ||
  'Replace this lead with a concise introduction that reflects real DoWhiz workflows and user intent.'
).trim();
const pageTitle = (args.title || `${headline} | DoWhiz`).trim();
const canonicalUrl = `https://dowhiz.com/blog/${slug}/`;
const destinationPath = path.resolve(process.cwd(), 'public', 'blog', slug, 'index.html');

if (fs.existsSync(destinationPath)) {
  console.error(`Refusing to overwrite existing page: ${destinationPath}`);
  process.exit(1);
}

const template = readTextFile('seo/templates/blog-post.template.html');
const rendered = template
  .replaceAll('__META_DESCRIPTION__', description)
  .replaceAll('__CANONICAL_URL__', canonicalUrl)
  .replaceAll('__PAGE_TITLE__', pageTitle)
  .replaceAll('__HEADLINE__', headline)
  .replaceAll('__DATE_PUBLISHED__', isoDate)
  .replaceAll('__DATE_MODIFIED__', isoDate)
  .replaceAll('__OWNER__', owner)
  .replaceAll('__EYEBROW__', eyebrow)
  .replaceAll('__LEAD__', lead)
  .replaceAll('__PRIMARY_KEYWORD__', primaryKeyword)
  .replaceAll('__PUBLISHED_LABEL__', formatDisplayDate(isoDate));

fs.mkdirSync(path.dirname(destinationPath), { recursive: true });
fs.writeFileSync(destinationPath, rendered);

console.log(`Created ${destinationPath}`);
console.log('Next steps:');
console.log('- Add a matching entry to website/seo/content-registry.json');
console.log('- Add or update a tracked keyword in website/seo/keywords.json');
console.log('- Run npm run seo:build');
