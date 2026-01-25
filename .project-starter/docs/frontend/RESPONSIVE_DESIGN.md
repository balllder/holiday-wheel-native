# Responsive Design Patterns

> **Philosophy**: Mobile-first design ensures the best experience across all devices. Design for the smallest screen first, then progressively enhance for larger screens.

## Table of Contents

- [Mobile-First Philosophy](#mobile-first-philosophy)
- [Breakpoint Strategy](#breakpoint-strategy)
- [Common Responsive Patterns](#common-responsive-patterns)
- [Responsive Typography](#responsive-typography)
- [Responsive Spacing](#responsive-spacing)
- [Responsive Images](#responsive-images)
- [Layout Patterns](#layout-patterns)
- [Testing Responsive Design](#testing-responsive-design)

---

## Mobile-First Philosophy

### Why Mobile-First?

1. **Performance**: Mobile users get the fastest experience
2. **Progressive Enhancement**: Add features for larger screens
3. **Simplicity**: Forces focus on essential content
4. **Future-Proof**: New devices tend to be smaller, not larger

### Mobile-First in Practice

```tsx
// ✅ CORRECT - Mobile-first approach
<div className="
  text-base           /* Mobile default */
  md:text-lg          /* Tablet and up */
  lg:text-xl          /* Desktop and up */
">
  Content scales up from mobile
</div>

// ❌ WRONG - Desktop-first approach
<div className="
  text-xl             /* Desktop default */
  md:text-lg          /* Scale down for tablet */
  sm:text-base        /* Scale down for mobile */
">
  Content scales down - poor mobile experience
</div>
```

---

## Breakpoint Strategy

### Tailwind Default Breakpoints

| Breakpoint | Min Width | Typical Devices |
|------------|-----------|----------------|
| `sm` | 640px | Large phones (landscape), small tablets |
| `md` | 768px | Tablets (portrait) |
| `lg` | 1024px | Tablets (landscape), small laptops |
| `xl` | 1280px | Laptops, desktops |
| `2xl` | 1536px | Large desktops |

### Custom Breakpoints

If needed, customize in `tailwind.config.js`:

```js
module.exports = {
  theme: {
    screens: {
      'xs': '475px',      // Extra small phones
      'sm': '640px',      // Large phones
      'md': '768px',      // Tablets
      'lg': '1024px',     // Small laptops
      'xl': '1280px',     // Desktops
      '2xl': '1536px',    // Large desktops
      '3xl': '1920px',    // Ultra-wide
    },
  },
}
```

### Breakpoint Usage Patterns

```tsx
// Multi-breakpoint responsive design
<div className="
  w-full              /* Mobile: full width */
  sm:w-11/12          /* Large phone: 91.67% width */
  md:w-3/4            /* Tablet: 75% width */
  lg:w-2/3            /* Laptop: 66.67% width */
  xl:w-1/2            /* Desktop: 50% width */
  max-w-7xl           /* Never wider than 1280px */
  mx-auto             /* Center on all screens */
">
  Responsive container
</div>
```

---

## Common Responsive Patterns

### 1. Stack to Grid

Transform stacked layout to grid as screen grows.

```tsx
<div className="
  grid grid-cols-1    /* Mobile: 1 column (stacked) */
  gap-4
  sm:grid-cols-2      /* Large phone: 2 columns */
  md:grid-cols-2      /* Tablet: 2 columns */
  lg:grid-cols-3      /* Laptop: 3 columns */
  xl:grid-cols-4      /* Desktop: 4 columns */
">
  {items.map(item => (
    <Card key={item.id} {...item} />
  ))}
</div>
```

### 2. Sidebar Layout

```tsx
// Mobile: stacked, Desktop: sidebar + content
<div className="flex flex-col lg:flex-row gap-6">
  {/* Sidebar */}
  <aside className="
    w-full            /* Mobile: full width */
    lg:w-64           /* Desktop: fixed 256px width */
    lg:shrink-0       /* Don't shrink on desktop */
  ">
    <Sidebar />
  </aside>

  {/* Main content */}
  <main className="flex-1 min-w-0">
    <Content />
  </main>
</div>
```

### 3. Navigation

```tsx
// Mobile: hamburger menu, Desktop: horizontal nav
<header className="bg-white shadow">
  <div className="container mx-auto px-4 py-4">
    <div className="flex items-center justify-between">
      <Logo />

      {/* Mobile menu button */}
      <button className="lg:hidden">
        <MenuIcon />
      </button>

      {/* Desktop navigation */}
      <nav className="hidden lg:flex gap-6">
        <NavLink href="/about">About</NavLink>
        <NavLink href="/services">Services</NavLink>
        <NavLink href="/contact">Contact</NavLink>
      </nav>
    </div>

    {/* Mobile menu (shown when open) */}
    <nav className="lg:hidden mt-4 space-y-2">
      <MobileNavLink href="/about">About</MobileNavLink>
      <MobileNavLink href="/services">Services</MobileNavLink>
      <MobileNavLink href="/contact">Contact</MobileNavLink>
    </nav>
  </div>
</header>
```

### 4. Hero Section

```tsx
<section className="
  py-12 md:py-20 lg:py-32    /* Responsive vertical padding */
">
  <div className="container mx-auto px-4">
    <div className="
      flex flex-col lg:flex-row  /* Stack on mobile, side-by-side on desktop */
      items-center
      gap-8 lg:gap-12
    ">
      {/* Text content */}
      <div className="
        w-full lg:w-1/2
        text-center lg:text-left  /* Center on mobile, left-align on desktop */
      ">
        <h1 className="
          text-3xl md:text-4xl lg:text-5xl xl:text-6xl
          font-bold
          mb-4 lg:mb-6
        ">
          Responsive Hero
        </h1>
        <p className="
          text-base md:text-lg lg:text-xl
          text-gray-600
          mb-6 lg:mb-8
        ">
          Description text that scales with screen size
        </p>
        <Button size="lg">Get Started</Button>
      </div>

      {/* Image */}
      <div className="w-full lg:w-1/2">
        <img
          src="/hero.jpg"
          alt="Hero"
          className="w-full h-auto rounded-lg shadow-lg"
        />
      </div>
    </div>
  </div>
</section>
```

### 5. Card Grid with Variable Columns

```tsx
<div className="
  grid
  grid-cols-1           /* Mobile: 1 card per row */
  sm:grid-cols-2        /* Large phone: 2 cards per row */
  lg:grid-cols-3        /* Desktop: 3 cards per row */
  gap-4 md:gap-6 lg:gap-8  /* Responsive gap */
">
  {products.map(product => (
    <ProductCard key={product.id} {...product} />
  ))}
</div>
```

---

## Responsive Typography

### Scale Typography with Viewport

```tsx
<h1 className="
  text-2xl        /* Mobile: 24px */
  sm:text-3xl     /* Large phone: 30px */
  md:text-4xl     /* Tablet: 36px */
  lg:text-5xl     /* Laptop: 48px */
  xl:text-6xl     /* Desktop: 60px */
  font-bold
  leading-tight   /* Keep line height tight */
">
  Responsive Heading
</h1>

<p className="
  text-sm md:text-base lg:text-lg
  leading-relaxed
">
  Body text that scales appropriately
</p>
```

### Using clamp() for Fluid Typography

For even smoother scaling, use CSS clamp:

```css
@layer utilities {
  .ps-text-fluid-xl {
    font-size: clamp(1.5rem, 4vw, 3rem);
    /* Min: 24px, Scales with viewport, Max: 48px */
  }

  .ps-text-fluid-lg {
    font-size: clamp(1.25rem, 3vw, 2rem);
    /* Min: 20px, Scales with viewport, Max: 32px */
  }

  .ps-text-fluid-base {
    font-size: clamp(1rem, 2vw, 1.125rem);
    /* Min: 16px, Scales with viewport, Max: 18px */
  }
}
```

---

## Responsive Spacing

### Padding and Margin

```tsx
<section className="
  px-4 py-8           /* Mobile: smaller padding */
  sm:px-6 sm:py-10    /* Large phone: medium padding */
  md:px-8 md:py-12    /* Tablet: larger padding */
  lg:px-12 lg:py-16   /* Desktop: largest padding */
">
  <div className="
    space-y-4         /* Mobile: 16px vertical spacing */
    md:space-y-6      /* Tablet: 24px vertical spacing */
    lg:space-y-8      /* Desktop: 32px vertical spacing */
  ">
    <Card />
    <Card />
    <Card />
  </div>
</section>
```

### Gap in Flex/Grid

```tsx
<div className="
  flex flex-wrap
  gap-3 md:gap-4 lg:gap-6
">
  <Item />
  <Item />
  <Item />
</div>
```

---

## Responsive Images

### Responsive Image Container

```tsx
<div className="
  w-full
  h-48 sm:h-64 md:h-80 lg:h-96  /* Responsive height */
  overflow-hidden
  rounded-lg
">
  <img
    src="/image.jpg"
    alt="Description"
    className="w-full h-full object-cover"
  />
</div>
```

### Picture Element with Breakpoints

```tsx
<picture>
  <source
    media="(min-width: 1024px)"
    srcSet="/image-large.jpg"
  />
  <source
    media="(min-width: 768px)"
    srcSet="/image-medium.jpg"
  />
  <img
    src="/image-small.jpg"
    alt="Description"
    className="w-full h-auto"
  />
</picture>
```

### Background Images

```tsx
<div className="
  h-64 md:h-96 lg:h-screen
  bg-cover bg-center
  bg-[url('/mobile-bg.jpg')]
  md:bg-[url('/tablet-bg.jpg')]
  lg:bg-[url('/desktop-bg.jpg')]
">
  Content with responsive background
</div>
```

---

## Layout Patterns

### Container Pattern

```tsx
// Responsive container with max-width
<div className="
  w-full
  max-w-7xl           /* Never wider than 1280px */
  mx-auto             /* Center horizontally */
  px-4 sm:px-6 lg:px-8  /* Responsive horizontal padding */
">
  <Content />
</div>
```

### Full-Width Sections with Constrained Content

```tsx
<section className="w-full bg-gray-100 py-12 md:py-20">
  <div className="container mx-auto px-4 sm:px-6 lg:px-8">
    {/* Content constrained but section full-width */}
    <h2>Section Title</h2>
  </div>
</section>
```

### Asymmetric Layouts

```tsx
// 2-column layout with different proportions
<div className="
  grid grid-cols-1 lg:grid-cols-12
  gap-6
">
  {/* Sidebar: 1/3 on desktop */}
  <aside className="lg:col-span-4">
    <Sidebar />
  </aside>

  {/* Main: 2/3 on desktop */}
  <main className="lg:col-span-8">
    <Content />
  </main>
</div>
```

---

## Testing Responsive Design

### Browser DevTools

1. Open Chrome DevTools (F12)
2. Toggle device toolbar (Ctrl+Shift+M)
3. Test common devices:
   - iPhone SE (375px)
   - iPhone 12 Pro (390px)
   - iPad (768px)
   - iPad Pro (1024px)
   - Desktop (1920px)

### Responsive Testing Checklist

- [ ] Mobile (320px - 640px)
  - [ ] Navigation works (hamburger menu)
  - [ ] Touch targets ≥ 44px
  - [ ] Text is readable without zooming
  - [ ] Images scale properly

- [ ] Tablet (641px - 1024px)
  - [ ] Layout adapts (2-column grids work)
  - [ ] Sidebar collapses or adjusts
  - [ ] Typography scales appropriately

- [ ] Desktop (1025px+)
  - [ ] Multi-column layouts work
  - [ ] Max-width containers prevent over-stretching
  - [ ] Hover states work
  - [ ] Content doesn't get too wide

### Manual Testing

```bash
# Using Playwright for responsive testing
npx playwright test --project=mobile
npx playwright test --project=tablet
npx playwright test --project=desktop
```

---

## Common Responsive Issues

### Issue 1: Text Too Small on Mobile

```tsx
// ❌ WRONG - Text too small
<p className="text-xs">
  This text is hard to read on mobile
</p>

// ✅ CORRECT - Minimum 16px on mobile
<p className="text-base md:text-sm">
  Readable on mobile, can be smaller on desktop
</p>
```

### Issue 2: Touch Targets Too Small

```tsx
// ❌ WRONG - Touch target too small
<button className="p-1">
  <Icon size={16} />
</button>

// ✅ CORRECT - Minimum 44x44px touch target
<button className="p-3 min-h-[44px] min-w-[44px]">
  <Icon size={20} />
</button>
```

### Issue 3: Horizontal Overflow

```tsx
// ❌ WRONG - Can cause horizontal scroll
<div className="w-[500px]">
  Fixed width larger than mobile screen
</div>

// ✅ CORRECT - Responsive width
<div className="w-full max-w-[500px]">
  Scales down on mobile
</div>
```

### Issue 4: Images Not Scaling

```tsx
// ❌ WRONG - Image might overflow
<img src="/image.jpg" alt="Large image" />

// ✅ CORRECT - Responsive image
<img
  src="/image.jpg"
  alt="Large image"
  className="w-full h-auto max-w-full"
/>
```

---

## Best Practices Summary

1. **Always mobile-first**: Design for smallest screen, enhance upward
2. **Test on real devices**: Emulators are helpful but not perfect
3. **Use Tailwind breakpoints**: Consistent breakpoints across project
4. **Touch targets ≥ 44px**: Essential for mobile usability
5. **Readable text**: Minimum 16px font size on mobile
6. **Avoid horizontal scroll**: Use max-width and responsive containers
7. **Progressive enhancement**: Core functionality works on mobile, enhanced on desktop
8. **Test across breakpoints**: Not just mobile and desktop

---

## Resources

- [Responsive Design - MDN](https://developer.mozilla.org/en-US/docs/Learn/CSS/CSS_layout/Responsive_Design)
- [Tailwind Responsive Design](https://tailwindcss.com/docs/responsive-design)
- [Mobile-First CSS - Web.dev](https://web.dev/responsive-web-design-basics/)

---

*Last Updated: 2026-01-24*
