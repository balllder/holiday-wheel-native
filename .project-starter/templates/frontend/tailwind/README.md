# Tailwind CSS Templates

This directory contains Tailwind CSS configuration templates and example components following best practices.

## 📁 Directory Structure

```
templates/frontend/tailwind/
├── README.md                      # This file
├── tailwind.config.js             # Recommended Tailwind configuration
├── custom-utilities.css           # Example custom utilities with project prefix
└── components/
    ├── README.md                  # Component usage guide
    ├── Button.tsx                 # Button component example
    └── Card.tsx                   # Card component with compound pattern
```

## 🚀 Quick Start

### 1. Configuration

Copy and customize `tailwind.config.js`:

```bash
cp templates/frontend/tailwind/tailwind.config.js ./tailwind.config.js
```

**Customizations needed:**
- Replace content paths with your project structure
- Update brand colors (primary, secondary)
- Add project-specific custom utilities
- Configure dark mode preference

### 2. Custom Utilities

Copy `custom-utilities.css` to your styles directory:

```bash
cp templates/frontend/tailwind/custom-utilities.css ./src/styles/custom-utilities.css
```

**IMPORTANT:** Replace `ps-` prefix with your project prefix throughout:
- `ps-card-hover` → `yourproject-card-hover`
- `ps-skeleton` → `yourproject-skeleton`
- etc.

### 3. Components

Copy example components to your project:

```bash
cp templates/frontend/tailwind/components/Button.tsx ./src/components/ui/
cp templates/frontend/tailwind/components/Card.tsx ./src/components/ui/
```

**Dependencies required:**
```bash
npm install clsx tailwind-merge
```

## 📖 Documentation

Comprehensive guides available in `/docs/frontend/`:

- **[TAILWIND_BEST_PRACTICES.md](../../../docs/frontend/TAILWIND_BEST_PRACTICES.md)**
  - Tailwind-first approach
  - Custom class naming conventions
  - When to use custom CSS
  - Component patterns
  - Performance best practices

- **[RESPONSIVE_DESIGN.md](../../../docs/frontend/RESPONSIVE_DESIGN.md)**
  - Mobile-first philosophy
  - Breakpoint strategy
  - Common responsive patterns
  - Responsive typography and spacing
  - Testing responsive design

## 🎯 Key Principles

1. **Tailwind-First**: Use Tailwind utilities before writing custom CSS
2. **Naming Convention**: Use project prefix for custom utilities (e.g., `ps-`, `rf-`)
3. **Mobile-First**: Design for smallest screen, enhance upward
4. **Variant Objects**: Use object mapping for component variants
5. **clsx & tailwind-merge**: Proper conditional classes and overrides

## 📦 What's Included

### tailwind.config.js

- ✅ Proper content configuration for PurgeCSS
- ✅ Dark mode setup
- ✅ Custom color palette (brand colors)
- ✅ Custom animations (fade-in, slide-in, shimmer)
- ✅ Custom utilities plugin
- ✅ Tailwind plugins (@tailwindcss/forms, typography, aspect-ratio)

### custom-utilities.css

- ✅ Custom hover effects (`ps-card-hover`, `ps-button-hover`)
- ✅ Text utilities (`ps-text-balance`, `ps-text-pretty`)
- ✅ Gradient utilities (`ps-gradient-primary`, etc.)
- ✅ Scrollbar styling (`ps-scrollbar-thin`, `ps-scrollbar-hide`)
- ✅ Glassmorphism effects (`ps-glass`, `ps-glass-dark`)
- ✅ Skeleton loaders (`ps-skeleton`)
- ✅ Custom alerts and badges

### components/

- ✅ **Button.tsx**: Full-featured button with variants, sizes, loading, icons
- ✅ **Card.tsx**: Compound component pattern with Header, Content, Footer
- ✅ TypeScript types
- ✅ Accessibility built-in
- ✅ Dark mode support

## 🛠️ Customization Guide

### Step 1: Choose Your Prefix

Decide on a project prefix (2-4 characters):
- `ps-` (Project Starter)
- `rf-` (RentalForge)
- `app-` (Generic app)
- `[yourproject]-`

### Step 2: Find & Replace

Replace `ps-` throughout all files:

```bash
# Example: Replace ps- with rf-
find templates/frontend/tailwind -type f -exec sed -i 's/ps-/rf-/g' {} +
```

### Step 3: Customize Brand Colors

Update `tailwind.config.js`:

```js
colors: {
  primary: {
    500: '#YOUR_PRIMARY_COLOR',
    // ... other shades
  },
}
```

### Step 4: Add Project-Specific Utilities

Add your custom utilities to `custom-utilities.css`:

```css
@layer utilities {
  .yourprefix-custom-utility {
    @apply /* Tailwind classes */;
  }
}
```

## 📚 Usage Examples

### Using Custom Utilities

```tsx
// Skeleton loader
<div className="ps-skeleton h-20 w-full rounded" />

// Card with hover effect
<div className="bg-white p-4 rounded-lg ps-card-hover">
  Content
</div>

// Gradient background
<div className="ps-gradient-primary text-white p-6">
  Gradient content
</div>
```

### Using Components

```tsx
import { Button } from '@/components/ui/Button';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/Card';

// Button variants
<Button variant="primary" size="lg" loading={isLoading}>
  Submit
</Button>

// Card composition
<Card variant="elevated" hoverable>
  <CardHeader>
    <CardTitle>Title</CardTitle>
  </CardHeader>
  <CardContent>
    Content goes here
  </CardContent>
</Card>
```

## ⚠️ Important Notes

1. **These are templates, not production code**: Adapt to your needs
2. **Replace prefix**: Use your project prefix, not `ps-`
3. **Review dependencies**: Ensure you have the required npm packages
4. **Test dark mode**: If using dark mode, test all components
5. **Accessibility**: Maintain ARIA attributes and focus states

## 🔗 Related Resources

- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [clsx](https://github.com/lukeed/clsx)
- [tailwind-merge](https://github.com/dcastil/tailwind-merge)
- [@tailwindcss/forms](https://github.com/tailwindlabs/tailwindcss-forms)
- [@tailwindcss/typography](https://github.com/tailwindlabs/tailwindcss-typography)

---

*Last Updated: 2026-01-24*
