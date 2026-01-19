#!/usr/bin/env node

/**
 * Extracts embedded JavaScript from Rust source files.
 * Handles double-brace escaping used in Rust format!() strings.
 *
 * Usage: node scripts/extract-rust-js.js [rust-file] [output-dir]
 */

const fs = require('fs');
const path = require('path');

const ROUTES_FILE = path.join(__dirname, '../apps/backend-rust/src/routes/mod.rs');
const OUTPUT_DIR = path.join(__dirname, '../.extracted-js');

// Known page functions in order of appearance
const PAGE_FUNCTIONS = ['join', 'index', 'register', 'lobby', 'game', 'admin'];

function unescapeRustBraces(content) {
  // Convert Rust double-braces back to single braces for valid JS
  return content.replace(/\{\{/g, '{').replace(/\}\}/g, '}');
}

function findPageForPosition(rustContent, position) {
  // Find which page function this script belongs to
  const beforeScript = rustContent.substring(0, position);

  let lastPage = 'unknown';
  for (const funcName of PAGE_FUNCTIONS) {
    const funcPattern = new RegExp(`pub async fn ${funcName}\\s*\\(`, 'g');
    let match;
    while ((match = funcPattern.exec(beforeScript)) !== null) {
      lastPage = funcName;
    }
  }

  return lastPage;
}

function extractScripts(rustContent) {
  const scripts = [];

  // Match <script> blocks (excluding external src scripts like CDN)
  // This regex finds <script> without src attribute
  const scriptRegex = /<script>([^]*?)<\/script>/g;
  let match;

  while ((match = scriptRegex.exec(rustContent)) !== null) {
    const scriptContent = match[1].trim();

    // Skip empty scripts or very short ones (likely just whitespace)
    if (scriptContent.length < 10) continue;

    // Calculate line number
    const lineNumber = rustContent.substring(0, match.index).split('\n').length;

    // Determine which page this belongs to
    const pageName = findPageForPosition(rustContent, match.index);

    // Unescape Rust braces
    const unescaped = unescapeRustBraces(scriptContent);

    scripts.push({
      pageName,
      lineNumber,
      content: unescaped,
      length: unescaped.split('\n').length
    });
  }

  return scripts;
}

function main() {
  const rustFile = process.argv[2] || ROUTES_FILE;
  const outputDir = process.argv[3] || OUTPUT_DIR;

  if (!fs.existsSync(rustFile)) {
    console.error(`Error: Rust file not found: ${rustFile}`);
    process.exit(1);
  }

  console.log(`Extracting JavaScript from: ${rustFile}`);

  const rustContent = fs.readFileSync(rustFile, 'utf-8');
  const scripts = extractScripts(rustContent);

  if (scripts.length === 0) {
    console.error('No <script> blocks found');
    process.exit(1);
  }

  // Create output directory
  fs.mkdirSync(outputDir, { recursive: true });

  // Track scripts per page for naming (in case of multiple scripts per page)
  const pageCount = {};
  const manifest = [];

  for (const script of scripts) {
    pageCount[script.pageName] = (pageCount[script.pageName] || 0) + 1;
    const suffix = pageCount[script.pageName] > 1 ? `-${pageCount[script.pageName]}` : '';
    const filename = `${script.pageName}${suffix}.js`;
    const filepath = path.join(outputDir, filename);

    fs.writeFileSync(filepath, script.content);
    console.log(`  Extracted: ${filename} (line ${script.lineNumber}, ${script.length} lines)`);

    manifest.push({
      file: filename,
      page: script.pageName,
      sourceLine: script.lineNumber,
      lines: script.length
    });
  }

  // Write manifest for validation script
  fs.writeFileSync(
    path.join(outputDir, 'manifest.json'),
    JSON.stringify(manifest, null, 2)
  );

  console.log(`\nExtracted ${scripts.length} scripts to: ${outputDir}`);
  return scripts.length;
}

const count = main();
process.exit(count > 0 ? 0 : 1);
