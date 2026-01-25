# Tailwind Component Examples

This directory contains example components demonstrating Tailwind CSS best practices.

## Components

### Button.tsx

Demonstrates:
- ✅ Variant-based styling with object mapping
- ✅ Using `clsx` for conditional classes
- ✅ Loading and disabled states
- ✅ Icon support (left and right)
- ✅ Responsive sizing
- ✅ Accessibility (focus states, disabled)
- ✅ TypeScript type safety

**Key Patterns:**
```tsx
// Define variants as objects
const variantClasses = {
  primary: '...',
  secondary: '...',
};

// Use clsx for conditional logic
className={clsx(
  'base-classes',
  variantClasses[variant],
  { 'conditional': condition }
)}
```

### Card.tsx

Demonstrates:
- ✅ Variant-based styling
- ✅ Using `tailwind-merge` for proper class overrides
- ✅ Compound components pattern (Card, CardHeader, CardContent, CardFooter)
- ✅ Flexible padding options
- ✅ Dark mode support
- ✅ Hoverable states

**Key Patterns:**
```tsx
// Use tailwind-merge to properly override classes
import { twMerge } from 'tailwind-merge';

className={twMerge(
  'default-classes',
  className  // User className properly overrides defaults
)}

// Compound components for flexibility
<Card>
  <CardHeader>...</CardHeader>
  <CardContent>...</CardContent>
  <CardFooter>...</CardFooter>
</Card>
```

## Dependencies

Install these packages to use the examples:

```bash
npm install clsx tailwind-merge
```

## Usage

These are **reference implementations**. Adapt them to your project's needs:

1. Copy the component you need
2. Replace `ps-` prefix with your project prefix
3. Customize variants and styles for your brand
4. Add additional variants as needed
5. Update TypeScript types for your use case

## Key Takeaways

1. **Object Mapping for Variants**: Better than nested ternaries
2. **clsx for Conditionals**: Clean conditional class logic
3. **tailwind-merge**: Proper class overrides
4. **Compound Components**: Flexibility and composition
5. **TypeScript**: Type safety for props
6. **Accessibility**: Focus states, ARIA, disabled states
7. **Dark Mode**: Built-in support with `dark:` variants

## Additional Resources

- [clsx Documentation](https://github.com/lukeed/clsx)
- [tailwind-merge Documentation](https://github.com/dcastil/tailwind-merge)
- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
