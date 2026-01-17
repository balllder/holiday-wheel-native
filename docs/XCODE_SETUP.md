# Xcode Distribution Setup Guide

This guide walks you through configuring both apps for App Store submission.

## Prerequisites

- [ ] Apple Developer Account ($99/year) at https://developer.apple.com
- [ ] Xcode 15+ installed on your Mac
- [ ] Apple Developer app or website access for certificates

---

## Part 1: Register Bundle IDs

Before opening Xcode, register your Bundle IDs in the Apple Developer Portal.

1. Go to https://developer.apple.com/account
2. Click **Certificates, Identifiers & Profiles**
3. Click **Identifiers** → **+** button
4. Select **App IDs** → Continue
5. Select **App** → Continue
6. Register these two Bundle IDs:

| App | Bundle ID | Description |
|-----|-----------|-------------|
| iOS Controller | `com.brefwiz.holidaywheel-controller` | Holiday Wheel Controller |
| tvOS Host | `com.brefwiz.holidaywheel` | Holiday Wheel |

For each, enable these capabilities:
- [x] App Groups (if you want shared data later)
- [x] Push Notifications (optional, for future updates)

---

## Part 2: iOS App (Phone Controller)

### Step 2.1: Open Project
```bash
cd apps/phone/ios
open phone.xcworkspace
```

> **Important:** Open `.xcworkspace`, not `.xcodeproj`

### Step 2.2: Configure Signing

1. Select **phone** project in navigator (blue icon)
2. Select **phone** target
3. Go to **Signing & Capabilities** tab
4. Check **Automatically manage signing**
5. Select your **Team** from dropdown
6. Bundle Identifier should be: `com.brefwiz.holidaywheel-controller`

### Step 2.3: Set Build Settings

1. Go to **Build Settings** tab
2. Search for and verify these settings:

| Setting | Value |
|---------|-------|
| Product Bundle Identifier | `com.brefwiz.holidaywheel-controller` |
| Product Name | `Holiday Wheel` |
| Marketing Version | `1.0.0` |
| Current Project Version | `1` |
| iOS Deployment Target | `15.0` |

### Step 2.4: Configure Release Scheme

1. Click **Product** → **Scheme** → **Edit Scheme**
2. Select **Archive** on the left
3. Set **Build Configuration** to **Release**
4. Close the dialog

### Step 2.5: Add App Icons

1. In navigator, expand **phone** → **Images.xcassets**
2. Select **AppIcon**
3. Drag your app icons to the appropriate slots:

| Size | Usage |
|------|-------|
| 1024x1024 | App Store |
| 180x180 | iPhone @3x |
| 120x120 | iPhone @2x |
| 167x167 | iPad Pro @2x |
| 152x152 | iPad @2x |

> **Tip:** Use https://appicon.co to generate all sizes from one image

### Step 2.6: Test Archive Build

1. Select **Any iOS Device (arm64)** as destination
2. Click **Product** → **Archive**
3. Wait for build to complete
4. Organizer window will open with your archive

---

## Part 3: tvOS App (Apple TV)

### Step 3.1: Open Project
```bash
cd apps/tv/ios
open tv.xcworkspace
```

### Step 3.2: Create tvOS Target

Since the tvOS target needs to be created in Xcode:

1. Select **tv** project in navigator
2. Click **+** at bottom of targets list
3. Choose **tvOS** → **App** → Next
4. Configure:
   - Product Name: `Holiday Wheel`
   - Bundle Identifier: `com.brefwiz.holidaywheel`
   - Language: Swift
   - User Interface: Storyboard
5. Click **Finish**

### Step 3.3: Configure tvOS Target

1. Delete the auto-generated Swift files (we use React Native)
2. Add existing files from `tv-tvOS/` folder:
   - Right-click target → **Add Files to "tv"**
   - Select: `AppDelegate.swift`, `Info.plist`, `LaunchScreen.storyboard`
   - Check **Copy items if needed**

3. Configure Build Phases:
   - Add **Run Script** phase for React Native bundle
   - Copy the script from the iOS target

### Step 3.4: Configure Signing

1. Select tvOS target
2. Go to **Signing & Capabilities**
3. Check **Automatically manage signing**
4. Select your **Team**
5. Bundle Identifier: `com.brefwiz.holidaywheel`

### Step 3.5: Set Build Settings

| Setting | Value |
|---------|-------|
| Product Bundle Identifier | `com.brefwiz.holidaywheel` |
| Product Name | `Holiday Wheel` |
| Marketing Version | `1.0.0` |
| Current Project Version | `1` |
| tvOS Deployment Target | `15.0` |

### Step 3.6: Add tvOS App Icons

tvOS requires layered app icons. In **Images.xcassets**:

1. Create new **App Icon & Top Shelf Image** (Brand Assets)
2. Add images for:

| Asset | Size | Purpose |
|-------|------|---------|
| App Icon - Front | 400x240 | Foreground layer |
| App Icon - Middle | 400x240 | Middle layer |
| App Icon - Back | 400x240 | Background layer |
| Top Shelf Image | 1920x720 | When app is highlighted |
| Top Shelf Wide | 2320x720 | Wide format |

> **Tip:** For simple icons, use same image for all 3 layers

---

## Part 4: Create Archives for Submission

### iOS App
```
1. Open phone.xcworkspace
2. Select "Any iOS Device (arm64)"
3. Product → Archive
4. Wait for completion
5. In Organizer: Distribute App → App Store Connect → Upload
```

### tvOS App
```
1. Open tv.xcworkspace
2. Select "Any tvOS Device (arm64)"
3. Product → Archive
4. Wait for completion
5. In Organizer: Distribute App → App Store Connect → Upload
```

---

## Part 5: App Store Connect Setup

### Create Apps

1. Go to https://appstoreconnect.apple.com
2. Click **My Apps** → **+** → **New App**
3. Create two apps:

**iOS App:**
- Platform: iOS
- Name: Holiday Wheel Controller
- Primary Language: English (U.S.)
- Bundle ID: com.brefwiz.holidaywheel-controller
- SKU: holidaywheel-controller-ios

**tvOS App:**
- Platform: tvOS
- Name: Holiday Wheel
- Primary Language: English (U.S.)
- Bundle ID: com.brefwiz.holidaywheel
- SKU: holidaywheel-tvos

### Required Information

For each app, fill in:

1. **App Information**
   - Category: Games → Word
   - Content Rights: Does not contain third-party content

2. **Pricing and Availability**
   - Price: Free (or your choice)
   - Availability: All countries

3. **App Privacy**
   - Privacy Policy URL: (your hosted URL)
   - Data collection: None collected

4. **Version Information**
   - Screenshots (see sizes below)
   - Description (from earlier)
   - Keywords
   - Support URL
   - Marketing URL (optional)

### Screenshot Sizes

**iOS:**
| Device | Size |
|--------|------|
| iPhone 6.7" | 1290 x 2796 |
| iPhone 6.5" | 1284 x 2778 |
| iPhone 5.5" | 1242 x 2208 |
| iPad Pro 12.9" | 2048 x 2732 |

**tvOS:**
| Size |
|------|
| 1920 x 1080 |

---

## Part 6: Submit for Review

1. Upload builds from Xcode Organizer
2. Wait for processing (5-30 minutes)
3. Select build in App Store Connect
4. Answer compliance questions:
   - Export Compliance: No (unless using custom encryption)
   - Content Rights: Yes, I own the rights
   - Advertising Identifier: No

5. Click **Submit for Review**

---

## Troubleshooting

### "No signing certificate"
→ Xcode → Preferences → Accounts → Download Manual Profiles

### "Bundle ID already in use"
→ Someone else registered it. Use a different Bundle ID.

### Archive fails with code signing error
→ Ensure Automatically manage signing is checked and team selected

### Build fails on tvOS
→ Run `pod install` in the ios directory first

---

## Version Updates

For updates after initial release:

1. Increment `CURRENT_PROJECT_VERSION` in Xcode (e.g., 1 → 2)
2. Optionally update `MARKETING_VERSION` (e.g., 1.0.0 → 1.0.1)
3. Archive and upload
4. In App Store Connect, add "What's New" text
5. Submit for review

---

## Quick Reference

| App | Bundle ID | App Store Name |
|-----|-----------|----------------|
| iOS | `com.brefwiz.holidaywheel-controller` | Holiday Wheel Controller |
| tvOS | `com.brefwiz.holidaywheel` | Holiday Wheel |

| Setting | iOS Value | tvOS Value |
|---------|-----------|------------|
| Deployment Target | 15.0 | 15.0 |
| Version | 1.0.0 | 1.0.0 |
| Build | 1 | 1 |
