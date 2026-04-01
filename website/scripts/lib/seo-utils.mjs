import fs from 'node:fs';
import path from 'node:path';

export function readJsonFile(relativePath) {
  const absolutePath = path.resolve(process.cwd(), relativePath);
  return JSON.parse(fs.readFileSync(absolutePath, 'utf8'));
}

export function ensureDir(relativePath) {
  const absolutePath = path.resolve(process.cwd(), relativePath);
  fs.mkdirSync(absolutePath, { recursive: true });
  return absolutePath;
}

export function writeTextFile(relativePath, nextContent) {
  const absolutePath = path.resolve(process.cwd(), relativePath);
  fs.mkdirSync(path.dirname(absolutePath), { recursive: true });
  const previousContent = fs.existsSync(absolutePath) ? fs.readFileSync(absolutePath, 'utf8') : null;

  if (previousContent === nextContent) {
    return false;
  }

  fs.writeFileSync(absolutePath, nextContent);
  return true;
}

export function readTextFile(relativePath) {
  return fs.readFileSync(path.resolve(process.cwd(), relativePath), 'utf8');
}

export function fileExists(relativePath) {
  return fs.existsSync(path.resolve(process.cwd(), relativePath));
}

export function normalizeWhitespace(value = '') {
  return String(value).replace(/\s+/g, ' ').trim();
}

export function normalizeKeyword(value = '') {
  return normalizeWhitespace(value).toLowerCase();
}

export function normalizeUrlPath(value = '') {
  const trimmed = normalizeWhitespace(value);
  if (!trimmed) {
    return '';
  }

  let pathname = trimmed;
  if (/^https?:\/\//i.test(trimmed)) {
    pathname = new URL(trimmed).pathname || '/';
  }

  if (!pathname.startsWith('/')) {
    pathname = `/${pathname}`;
  }

  if (pathname !== '/' && !pathname.endsWith('/')) {
    pathname = `${pathname}/`;
  }

  return pathname;
}

export function canonicalToPath(canonicalUrl = '') {
  return normalizeUrlPath(canonicalUrl);
}

export function stripDoWhizSuffix(title = '') {
  return normalizeWhitespace(String(title).replace(/\s*\|\s*DoWhiz\s*$/i, ''));
}

export function formatDisplayDate(isoDate) {
  const utcDate = new Date(`${isoDate}T00:00:00Z`);
  return new Intl.DateTimeFormat('en-US', {
    month: 'long',
    day: 'numeric',
    year: 'numeric',
    timeZone: 'UTC'
  }).format(utcDate);
}

export function escapeHtml(value = '') {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

export function escapeXml(value = '') {
  return escapeHtml(value);
}

export function parseNumber(value, fallback = 0) {
  if (value === undefined || value === null || value === '') {
    return fallback;
  }

  const parsed = Number.parseFloat(String(value).replace(/,/g, ''));
  return Number.isFinite(parsed) ? parsed : fallback;
}

export function parseInteger(value, fallback = 0) {
  if (value === undefined || value === null || value === '') {
    return fallback;
  }

  const parsed = Number.parseInt(String(value).replace(/,/g, ''), 10);
  return Number.isFinite(parsed) ? parsed : fallback;
}

export function replaceBetweenMarkers(input, startMarker, endMarker, replacement) {
  const startIndex = input.indexOf(startMarker);
  const endIndex = input.indexOf(endMarker);

  if (startIndex === -1 || endIndex === -1 || endIndex < startIndex) {
    throw new Error(`Missing markers: ${startMarker} / ${endMarker}`);
  }

  const before = input.slice(0, startIndex + startMarker.length);
  const endLineStart = input.lastIndexOf('\n', endIndex) + 1;
  const after = input.slice(endLineStart);
  return `${before}\n${replacement}\n${after}`;
}

function parseCsvLine(line) {
  const cells = [];
  let current = '';
  let inQuotes = false;

  for (let index = 0; index < line.length; index += 1) {
    const char = line[index];
    const next = line[index + 1];

    if (char === '"') {
      if (inQuotes && next === '"') {
        current += '"';
        index += 1;
      } else {
        inQuotes = !inQuotes;
      }
      continue;
    }

    if (char === ',' && !inQuotes) {
      cells.push(current);
      current = '';
      continue;
    }

    current += char;
  }

  cells.push(current);
  return cells;
}

export function parseCsvText(csvText) {
  const normalized = csvText.replace(/^\uFEFF/, '');
  const lines = normalized
    .split(/\r?\n/)
    .map((line) => line.trimEnd())
    .filter((line) => line.length > 0);

  if (lines.length === 0) {
    return [];
  }

  const header = parseCsvLine(lines[0]).map((cell) => normalizeWhitespace(cell));
  return lines.slice(1).map((line) => {
    const cells = parseCsvLine(line);
    const row = {};
    header.forEach((columnName, index) => {
      row[columnName] = cells[index] ?? '';
    });
    return row;
  });
}

export function loadStructuredRows(relativePath) {
  const absolutePath = path.resolve(process.cwd(), relativePath);
  if (!fs.existsSync(absolutePath)) {
    return { rows: [], format: 'missing', absolutePath };
  }

  const extension = path.extname(absolutePath).toLowerCase();
  const raw = fs.readFileSync(absolutePath, 'utf8');

  if (extension === '.json') {
    const parsed = JSON.parse(raw);
    const rows = Array.isArray(parsed) ? parsed : parsed.rows || parsed.keywords || parsed.pages || [];
    return { rows, format: 'json', absolutePath };
  }

  if (extension === '.csv') {
    return { rows: parseCsvText(raw), format: 'csv', absolutePath };
  }

  throw new Error(`Unsupported data file extension for ${relativePath}`);
}

export function parseArgs(argv) {
  const args = {};
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (!token.startsWith('--')) {
      continue;
    }

    const key = token.slice(2);
    const next = argv[index + 1];
    if (!next || next.startsWith('--')) {
      args[key] = true;
      continue;
    }

    args[key] = next;
    index += 1;
  }

  return args;
}
