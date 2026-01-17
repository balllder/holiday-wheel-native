#!/usr/bin/env node

/**
 * App Icon Generator for Holiday Wheel
 *
 * Generates all required icon sizes for iOS and tvOS from the source SVG.
 *
 * Usage:
 *   node scripts/generate-icons.js
 *
 * This script uses npx to run sharp-cli, no installation required.
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Paths
const SOURCE_SVG = path.join(__dirname, '../assets/app-icon.svg');
const OUTPUT_DIR = path.join(__dirname, '../assets/generated-icons');

// iOS icon sizes (square)
const IOS_ICONS = [
  { size: 1024, name: 'AppStore' },
  { size: 180, name: 'iPhone-60@3x' },
  { size: 120, name: 'iPhone-60@2x' },
  { size: 167, name: 'iPad-Pro-83.5@2x' },
  { size: 152, name: 'iPad-76@2x' },
  { size: 76, name: 'iPad-76@1x' },
  { size: 87, name: 'iPhone-Settings-29@3x' },
  { size: 58, name: 'Settings-29@2x' },
  { size: 29, name: 'Settings-29@1x' },
  { size: 80, name: 'Spotlight-40@2x' },
  { size: 60, name: 'Notification-20@3x' },
  { size: 40, name: 'Notification-20@2x' },
  { size: 20, name: 'Notification-20@1x' },
];

// tvOS sizes (16:9 ratio for parallax effect)
const TVOS_ICONS = [
  { width: 400, height: 240, name: 'tvOS-Small' },
  { width: 1280, height: 768, name: 'tvOS-Large' },
  { width: 1920, height: 720, name: 'TopShelf' },
  { width: 2320, height: 720, name: 'TopShelf-Wide' },
];

function ensureDir(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

function runSharp(inputPath, outputPath, width, height) {
  const cmd = `npx --yes sharp-cli resize ${width} ${height} --input "${inputPath}" --output "${outputPath}"`;
  try {
    execSync(cmd, { stdio: 'pipe' });
    return true;
  } catch (err) {
    console.error(`  Error resizing to ${width}x${height}: ${err.message}`);
    return false;
  }
}

function generateSquareIcon(size, name, subDir) {
  const outputPath = path.join(OUTPUT_DIR, subDir, `${name}-${size}x${size}.png`);
  ensureDir(path.dirname(outputPath));

  if (runSharp(SOURCE_SVG, outputPath, size, size)) {
    console.log(`  Created: ${name}-${size}x${size}.png`);
  }
}

function generateRectIcon(width, height, name, subDir) {
  const outputPath = path.join(OUTPUT_DIR, subDir, `${name}-${width}x${height}.png`);
  ensureDir(path.dirname(outputPath));

  // For rectangular icons, resize to fit height and let sharp handle it
  // Sharp CLI will resize while maintaining aspect ratio with fit contain
  const cmd = `npx --yes sharp-cli resize ${width} ${height} --fit contain --background "#0d0628" --input "${SOURCE_SVG}" --output "${outputPath}"`;

  try {
    execSync(cmd, { stdio: 'pipe' });
    console.log(`  Created: ${name}-${width}x${height}.png`);
  } catch (err) {
    console.error(`  Error creating ${name}: ${err.message}`);
  }
}

function main() {
  console.log('Holiday Wheel App Icon Generator');
  console.log('================================\n');

  // Check source exists
  if (!fs.existsSync(SOURCE_SVG)) {
    console.error(`Error: Source SVG not found at ${SOURCE_SVG}`);
    process.exit(1);
  }

  ensureDir(OUTPUT_DIR);

  // Generate iOS icons
  console.log('Generating iOS icons...');
  for (const icon of IOS_ICONS) {
    generateSquareIcon(icon.size, icon.name, 'ios');
  }

  // Generate tvOS icons
  console.log('\nGenerating tvOS icons...');
  for (const icon of TVOS_ICONS) {
    generateRectIcon(icon.width, icon.height, icon.name, 'tvos');
  }

  // Generate tvOS icon layers (all same image for simple icons)
  console.log('\nGenerating tvOS icon layers (Front/Middle/Back)...');
  const layers = ['Front', 'Middle', 'Back'];
  for (const layer of layers) {
    generateRectIcon(400, 240, `tvOS-Small-${layer}`, 'tvos/layers');
    generateRectIcon(1280, 768, `tvOS-Large-${layer}`, 'tvos/layers');
  }

  console.log('\n================================');
  console.log('Icon generation complete!');
  console.log(`\nOutput directory: ${OUTPUT_DIR}`);
  console.log('\nNext steps:');
  console.log('1. Open Xcode projects on your Mac');
  console.log('2. Drag icons from assets/generated-icons/ios/ to Images.xcassets/AppIcon');
  console.log('3. For tvOS, use the layered icons from assets/generated-icons/tvos/layers/');
}

main();
