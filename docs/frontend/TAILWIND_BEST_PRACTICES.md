# Tailwind CSS Best Practices

> **Philosophy**: Use Tailwind classes FIRST, custom CSS LAST. Embrace the framework instead of fighting it.

## Table of Contents

- [Tailwind-First Approach](#tailwind-first-approach)
- [Custom Class Naming Convention](#custom-class-naming-convention)
- [When to Use Custom CSS](#when-to-use-custom-css)
- [Component Patterns](#component-patterns)
- [Responsive Design Patterns](#responsive-design-patterns)
- [Performance Best Practices](#performance-best-practices)
- [Common Pitfalls](#common-pitfalls)

---

## Tailwind-First Approach

**Principle:** Exhaust Tailwind's utility classes before writing custom CSS.

### Why Tailwind-First?

- ✅ **Consistency**: Standardized spacing, colors, and sizing
- ✅ **Performance**: Optimized and purged in production
- ✅ **Maintainability**: No custom CSS to maintain
- ✅ **Developer Experience**: No switching between files
- ✅ **Responsive**: Built-in responsive utilities

### Examples

#### ❌ WRONG - Fighting the Framework

```tsx
// Component file
<div className="custom-card">
  <h2 className="custom-heading">Title</h2>
  <p className="custom-text">Description</p>
</div>

/* Separate CSS file */
.custom-card {
  background-color: white;
  padding: 1rem;
  border-radius: 0.5rem;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

.custom-heading {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
}

.custom-text {
  color: #6b7280;
  line-height: 1.5;
}
```

#### ✅ CORRECT - Using Tailwind

```tsx
<div className="bg-white p-4 rounded-lg shadow-sm">
  <h2 className="text-xl font-semibold mb-2">Title</h2>
  <p className="text-gray-500 leading-relaxed">Description</p>
</div>
```

**Benefits:**
- No context switching between files
- Responsive modifiers available (`md:p-6`, `lg:text-2xl`)
- Dark mode support (`dark:bg-gray-800`)
- No unused CSS in production

---

## Custom Class Naming Convention

**Pattern:** Use project prefix for custom utilities to avoid conflicts.

### Choosing a Prefix

- Use your project/company name abbreviation
- Keep it short (2-4 characters)
- Examples: `rf-` (RentalForge), `ps-` (Project Starter), `app-`

### Example Custom Utilities

```css
/* templates/frontend/tailwind/custom-utilities.css */
@layer utilities {
  /* Complex hover effects */
  .ps-card-hover {
    @apply transition-all duration-200 hover:shadow-lg hover:scale-[1.02];
  }

  /* Text wrapping (browser-specific) */
  .ps-text-balance {
    text-wrap: balance;
  }

  /* Custom gradients */
  .ps-gradient-primary {
    @apply bg-gradient-to-r from-blue-600 to-blue-800;
  }

  .ps-gradient-secondary {
    @apply bg-gradient-to-r from-purple-600 to-pink-600;
  }

  /* Scrollbar styling */
  .ps-scrollbar-thin {
    scrollbar-width: thin;
    scrollbar-color: theme('colors.gray.400') transparent;
  }

  .ps-scrollbar-thin::-webkit-scrollbar {
    width: 8px;
  }

  .ps-scrollbar-thin::-webkit-scrollbar-track {
    background: transparent;
  }

  .ps-scrollbar-thin::-webkit-scrollbar-thumb {
    background-color: theme('colors.gray.400');
    border-radius: 4px;
  }
}
```

### When to Create Custom Utilities

**✅ Create Custom Utility When:**
- Used in 3+ places
- Combines multiple Tailwind classes
- Browser-specific feature not in Tailwind
- Complex pattern worth naming

**❌ Don't Create Custom Utility When:**
- Only used once or twice
- Simple combination of 2-3 Tailwind classes
- Already available in Tailwind

---

## When to Use Custom CSS

**Default:** Use Tailwind. **Exception:** When Tailwind can't do it or it's impractical.

### Valid Reasons for Custom CSS

#### 1. Complex Animations

Tailwind's animation utilities are limited. Complex animations need custom CSS.

```css
@keyframes ps-shimmer {
  0% { background-position: -1000px 0; }
  100% { background-position: 1000px 0; }
}

.ps-skeleton {
  @apply bg-gray-200;
  animation: ps-shimmer 2s infinite linear;
  background: linear-gradient(
    to right,
    #f0f0f0 0%,
    #e0e0e0 20%,
    #f0f0f0 40%,
    #f0f0f0 100%
  );
  background-size: 1000px 100%;
}

@keyframes ps-slide-in-up {
  from {
    transform: translateY(100%);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.ps-toast-enter {
  animation: ps-slide-in-up 0.3s ease-out;
}
```

#### 2. Browser-Specific Hacks

```css
.ps-sticky-header {
  @apply sticky top-0 z-50;

  /* Safari-specific fix for sticky positioning */
  @supports (-webkit-backdrop-filter: blur(10px)) {
    -webkit-backdrop-filter: blur(10px);
    backdrop-filter: blur(10px);
  }
}
```

#### 3. Third-Party Library Overrides

```css
/* Override React-DatePicker styles */
.ps-datepicker .react-datepicker {
  @apply border-gray-300 rounded-lg shadow-lg;
}

.ps-datepicker .react-datepicker__day--selected {
  @apply bg-blue-600 text-white;
}
```

#### 4. Performance-Critical Repeated Patterns

If a component is rendered hundreds of times, extract to a class:

```css
/* Instead of repeating these 8+ classes on each list item */
.ps-list-item {
  @apply flex items-center justify-between px-4 py-3 border-b border-gray-200 hover:bg-gray-50 transition-colors;
}
```

---

## Component Patterns

### Variant-Based Components

Use object mapping for component variants instead of conditional classes.

```tsx
// components/Card.tsx
interface CardProps {
  variant?: 'default' | 'elevated' | 'bordered' | 'ghost';
  padding?: 'sm' | 'md' | 'lg';
  children: React.ReactNode;
}

const cardVariants = {
  default: 'bg-white rounded-lg shadow-sm',
  elevated: 'bg-white rounded-lg shadow-lg',
  bordered: 'bg-white rounded-lg border-2 border-gray-200',
  ghost: 'bg-transparent',
};

const cardPadding = {
  sm: 'p-3',
  md: 'p-4',
  lg: 'p-6',
};

export function Card({
  variant = 'default',
  padding = 'md',
  children
}: CardProps) {
  return (
    <div className={`${cardVariants[variant]} ${cardPadding[padding]}`}>
      {children}
    </div>
  );
}

// Usage
<Card variant="elevated" padding="lg">
  <h2>Content</h2>
</Card>
```

### Using clsx for Conditional Classes

```tsx
import clsx from 'clsx';

interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  loading?: boolean;
  disabled?: boolean;
  children: React.ReactNode;
}

export function Button({
  variant = 'primary',
  size = 'md',
  loading = false,
  disabled = false,
  children
}: ButtonProps) {
  return (
    <button
      className={clsx(
        // Base styles
        'font-semibold rounded-lg transition-colors',

        // Size variants
        {
          'px-3 py-1.5 text-sm': size === 'sm',
          'px-4 py-2 text-base': size === 'md',
          'px-6 py-3 text-lg': size === 'lg',
        },

        // Color variants
        {
          'bg-blue-600 text-white hover:bg-blue-700': variant === 'primary',
          'bg-gray-200 text-gray-800 hover:bg-gray-300': variant === 'secondary',
          'bg-red-600 text-white hover:bg-red-700': variant === 'danger',
        },

        // State
        {
          'opacity-50 cursor-not-allowed': disabled || loading,
          'cursor-wait': loading,
        }
      )}
      disabled={disabled || loading}
    >
      {loading ? 'Loading...' : children}
    </button>
  );
}
```

### Tailwind Merge for Class Conflicts

Use `tailwind-merge` to properly handle conflicting classes:

```tsx
import { twMerge } from 'tailwind-merge';

interface BoxProps {
  className?: string;
  children: React.ReactNode;
}

export function Box({ className, children }: BoxProps) {
  return (
    <div className={twMerge('p-4 bg-white rounded-lg', className)}>
      {children}
    </div>
  );
}

// Usage - className overrides work correctly
<Box className="p-8 bg-gray-100">
  {/* p-8 overrides p-4, bg-gray-100 overrides bg-white */}
</Box>
```

---

## Responsive Design Patterns

### Mobile-First Approach

**Always design for mobile first, then scale up.**

```tsx
// ✅ CORRECT - Mobile-first
<div className="
  grid grid-cols-1      /* Mobile: 1 column */
  gap-4
  md:grid-cols-2        /* Tablet: 2 columns */
  lg:grid-cols-3        /* Desktop: 3 columns */
  xl:grid-cols-4        /* Large desktop: 4 columns */
">
  {items.map(item => <Card key={item.id} {...item} />)}
</div>

// ❌ WRONG - Desktop-first
<div className="
  grid grid-cols-4      /* Starts with 4 columns */
  gap-4
  lg:grid-cols-3        /* Reduces to 3 */
  md:grid-cols-2        /* Reduces to 2 */
  sm:grid-cols-1        /* Finally 1 */
">
  {items.map(item => <Card key={item.id} {...item} />)}
</div>
```

### Responsive Spacing

```tsx
// Variable spacing based on screen size
<section className="
  px-4 py-8           /* Mobile: smaller padding */
  md:px-6 md:py-12    /* Tablet: medium padding */
  lg:px-8 lg:py-16    /* Desktop: larger padding */
">
  <h1 className="
    text-2xl           /* Mobile: smaller text */
    md:text-3xl        /* Tablet: medium text */
    lg:text-4xl        /* Desktop: larger text */
  ">
    Responsive Heading
  </h1>
</section>
```

### Responsive Layout Patterns

#### Stack to Grid

```tsx
// Mobile: stacked, Desktop: side-by-side
<div className="flex flex-col lg:flex-row gap-6">
  <aside className="lg:w-64">
    <Sidebar />
  </aside>
  <main className="flex-1">
    <Content />
  </main>
</div>
```

#### Hidden/Visible at Breakpoints

```tsx
// Mobile menu icon, hidden on desktop
<button className="lg:hidden">
  <MenuIcon />
</button>

// Desktop navigation, hidden on mobile
<nav className="hidden lg:flex gap-4">
  <NavLink href="/about">About</NavLink>
  <NavLink href="/contact">Contact</NavLink>
</nav>
```

### Container Patterns

```tsx
// Responsive container with max-width
<div className="
  w-full
  max-w-7xl
  mx-auto
  px-4 sm:px-6 lg:px-8
">
  <Content />
</div>
```

---

## Performance Best Practices

### 1. Avoid Arbitrary Values in Hot Paths

Arbitrary values force JIT compilation and can slow down rendering.

```tsx
// ❌ SLOW - Arbitrary values in loops
{items.map(item => (
  <div
    key={item.id}
    className={`w-[${item.width}px] h-[${item.height}px]`}
  >
    {item.content}
  </div>
))}

// ✅ FAST - Use inline styles for dynamic values
{items.map(item => (
  <div
    key={item.id}
    style={{ width: `${item.width}px`, height: `${item.height}px` }}
  >
    {item.content}
  </div>
))}

// ✅ BETTER - Use Tailwind classes when possible
{items.map(item => (
  <div key={item.id} className="w-full h-48">
    {item.content}
  </div>
))}
```

### 2. Use PurgeCSS/Content Configuration

Ensure your `tailwind.config.js` has proper content paths:

```js
// tailwind.config.js
module.exports = {
  content: [
    './src/**/*.{js,jsx,ts,tsx}',
    './public/index.html',
  ],
  // ...
}
```

### 3. Extract Common Patterns

If a set of classes appears 10+ times, consider extracting:

```tsx
// Before: Repeated everywhere
<button className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">

// After: Extract to component
<Button variant="primary">Click Me</Button>
```

### 4. Avoid Deep Nesting

```tsx
// ❌ Hard to maintain
<div className="p-4">
  <div className="mb-4">
    <div className="flex items-center">
      <div className="mr-2">
        <Icon />
      </div>
      <div className="flex-1">
        <Title />
      </div>
    </div>
  </div>
</div>

// ✅ Extract to components
<CardHeader>
  <HeaderIcon icon={<Icon />} />
  <HeaderTitle title="Title" />
</CardHeader>
```

---

## Common Pitfalls

### 1. Fighting the Framework

**Problem:** Writing custom CSS for things Tailwind already provides.

```tsx
// ❌ WRONG
.custom-flex {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

// ✅ CORRECT
<div className="flex items-center justify-between">
```

### 2. Not Using @layer

**Problem:** Custom CSS not properly purged in production.

```css
/* ❌ WRONG - Won't be purged */
.my-custom-class {
  color: red;
}

/* ✅ CORRECT - Properly layered */
@layer components {
  .ps-alert {
    @apply p-4 rounded-lg border;
  }
}
```

### 3. Inconsistent Spacing

**Problem:** Using arbitrary values instead of Tailwind's spacing scale.

```tsx
// ❌ WRONG - Arbitrary spacing
<div className="mb-[13px] mt-[27px]">

// ✅ CORRECT - Tailwind spacing scale
<div className="mb-3 mt-6">
```

### 4. Not Using Dark Mode

**Problem:** Ignoring dark mode support.

```tsx
// ❌ WRONG - No dark mode
<div className="bg-white text-black">

// ✅ CORRECT - Dark mode support
<div className="bg-white dark:bg-gray-800 text-black dark:text-white">
```

### 5. Long Class Strings

**Problem:** Unreadable long class strings.

```tsx
// ❌ WRONG - Hard to read
<div className="flex items-center justify-between px-4 py-3 bg-white rounded-lg shadow-sm border border-gray-200 hover:shadow-md transition-shadow">

// ✅ CORRECT - Multi-line with logical grouping
<div className="
  flex items-center justify-between
  px-4 py-3
  bg-white rounded-lg
  border border-gray-200
  shadow-sm hover:shadow-md
  transition-shadow
">
```

---

## Checklist for New Components

Before shipping a component, verify:

- [ ] Uses Tailwind classes where possible
- [ ] Custom CSS is justified and documented
- [ ] Custom utilities follow naming convention (project prefix)
- [ ] Responsive design follows mobile-first approach
- [ ] Dark mode support included (if applicable)
- [ ] No arbitrary values in hot paths
- [ ] Classes are organized and readable
- [ ] Extracted to components if used 3+ times

---

## References

- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [Tailwind CSS Best Practices](https://tailwindcss.com/docs/reusing-styles)
- [clsx](https://github.com/lukeed/clsx) - Conditional class utility
- [tailwind-merge](https://github.com/dcastil/tailwind-merge) - Merge Tailwind classes

---

*Last Updated: 2026-01-24*
