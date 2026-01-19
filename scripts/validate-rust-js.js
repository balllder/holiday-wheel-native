#!/usr/bin/env node

/**
 * Validates extracted JavaScript files using Node.js syntax checking.
 *
 * Usage: node scripts/validate-rust-js.js [extracted-dir]
 */

const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const DEFAULT_DIR = path.join(__dirname, '../.extracted-js');

function validateFile(filepath) {
  const result = spawnSync('node', ['--check', filepath], {
    encoding: 'utf-8',
    stdio: ['pipe', 'pipe', 'pipe']
  });

  return {
    file: path.basename(filepath),
    success: result.status === 0,
    error: result.stderr || null
  };
}

function main() {
  const extractedDir = process.argv[2] || DEFAULT_DIR;

  if (!fs.existsSync(extractedDir)) {
    console.error(`Error: Extracted JS directory not found: ${extractedDir}`);
    console.error('Run extract-rust-js.js first');
    process.exit(1);
  }

  // Read manifest if available
  const manifestPath = path.join(extractedDir, 'manifest.json');
  let manifest = null;
  if (fs.existsSync(manifestPath)) {
    manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf-8'));
  }

  // Get all JS files
  const jsFiles = fs.readdirSync(extractedDir)
    .filter(f => f.endsWith('.js'))
    .map(f => path.join(extractedDir, f));

  if (jsFiles.length === 0) {
    console.error('No JavaScript files found to validate');
    process.exit(1);
  }

  console.log(`Validating ${jsFiles.length} extracted JavaScript files...\n`);

  let passed = 0;
  let failed = 0;
  const failures = [];

  for (const filepath of jsFiles) {
    const result = validateFile(filepath);

    if (result.success) {
      console.log(`  ✓ ${result.file}`);
      passed++;
    } else {
      console.log(`  ✗ ${result.file}`);
      failed++;
      failures.push(result);
    }
  }

  console.log(`\n${'='.repeat(50)}`);
  console.log(`Results: ${passed} passed, ${failed} failed`);

  if (failures.length > 0) {
    console.log('\nFailure details:');
    for (const f of failures) {
      console.log(`\n--- ${f.file} ---`);
      console.log(f.error);

      // If manifest exists, show source location
      if (manifest) {
        const entry = manifest.find(m => m.file === f.file);
        if (entry) {
          console.log(`Source: apps/backend-rust/src/routes/mod.rs line ${entry.sourceLine} (page: ${entry.page})`);
        }
      }
    }
    process.exit(1);
  }

  console.log('\nAll JavaScript syntax is valid!');
}

main();
