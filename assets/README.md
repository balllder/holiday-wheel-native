# App Assets

This folder contains source assets and generated icons for the Holiday Wheel apps.

## Files

- `app-icon.svg` - Source SVG for app icons (1024x1024 viewBox)

## Generating Icons

Run the icon generation script to create all required sizes:

```bash
# From project root
npm install sharp
node scripts/generate-icons.js
```

This creates `generated-icons/` with all sizes for iOS and tvOS.

## Generated Icon Sizes

### iOS (Square)
| Size | Usage |
|------|-------|
| 1024x1024 | App Store |
| 180x180 | iPhone @3x |
| 120x120 | iPhone @2x |
| 167x167 | iPad Pro @2x |
| 152x152 | iPad @2x |

### tvOS (16:9 Ratio)
| Size | Usage |
|------|-------|
| 400x240 | Small app icon |
| 1280x768 | Large app icon |
| 1920x720 | Top Shelf |
| 2320x720 | Top Shelf Wide |

### tvOS Layers
For parallax effect, three layers are generated (Front/Middle/Back).
For a simple icon, all layers use the same image.

## Adding to Xcode

1. Open `apps/phone/ios/phone.xcworkspace` or `apps/tv/ios/tv.xcworkspace`
2. Select `Images.xcassets` in the project navigator
3. Select `AppIcon`
4. Drag the appropriately sized PNGs from `generated-icons/ios/`
5. For tvOS, use images from `generated-icons/tvos/` and `generated-icons/tvos/layers/`

## Customizing the Icon

Edit `app-icon.svg` with any vector editor (Figma, Illustrator, Inkscape).
The icon features:
- Dark purple gradient background (#0d0628 to #1a0a3e)
- Colorful 8-segment wheel with dollar values
- Gold decorative ring and pointer
- "H" center hub in gold

Re-run the generation script after editing.
