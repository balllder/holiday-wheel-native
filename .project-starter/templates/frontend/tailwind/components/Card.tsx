import { HTMLAttributes, ReactNode } from 'react';
import { twMerge } from 'tailwind-merge';

/**
 * Card Component - Tailwind Best Practices Example
 *
 * Demonstrates:
 * - Variant-based styling
 * - Using tailwind-merge for proper class overrides
 * - Flexible padding options
 * - Responsive design support
 * - Composition pattern with compound components
 */

export interface CardProps extends HTMLAttributes<HTMLDivElement> {
  /** Visual style variant */
  variant?: 'default' | 'elevated' | 'bordered' | 'ghost';

  /** Padding size */
  padding?: 'none' | 'sm' | 'md' | 'lg';

  /** Enable hover effect */
  hoverable?: boolean;

  children: ReactNode;
}

const variantClasses = {
  default: 'bg-white dark:bg-gray-800 rounded-lg shadow-sm',
  elevated: 'bg-white dark:bg-gray-800 rounded-lg shadow-lg',
  bordered: 'bg-white dark:bg-gray-800 rounded-lg border-2 border-gray-200 dark:border-gray-700',
  ghost: 'bg-transparent',
};

const paddingClasses = {
  none: '',
  sm: 'p-3',
  md: 'p-4 md:p-6',
  lg: 'p-6 md:p-8',
};

export function Card({
  variant = 'default',
  padding = 'md',
  hoverable = false,
  className,
  children,
  ...props
}: CardProps) {
  return (
    <div
      className={twMerge(
        // Variant styles
        variantClasses[variant],

        // Padding
        paddingClasses[padding],

        // Hover effect
        hoverable && 'ps-card-hover cursor-pointer',

        // Custom className (properly merged)
        className
      )}
      {...props}
    >
      {children}
    </div>
  );
}

/**
 * Card Header Component
 */
export interface CardHeaderProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
}

export function CardHeader({ className, children, ...props }: CardHeaderProps) {
  return (
    <div
      className={twMerge(
        'border-b border-gray-200 dark:border-gray-700 pb-4 mb-4',
        className
      )}
      {...props}
    >
      {children}
    </div>
  );
}

/**
 * Card Title Component
 */
export interface CardTitleProps extends HTMLAttributes<HTMLHeadingElement> {
  as?: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6';
  children: ReactNode;
}

export function CardTitle({
  as: Component = 'h3',
  className,
  children,
  ...props
}: CardTitleProps) {
  return (
    <Component
      className={twMerge('text-xl font-semibold text-gray-900 dark:text-white', className)}
      {...props}
    >
      {children}
    </Component>
  );
}

/**
 * Card Description Component
 */
export interface CardDescriptionProps extends HTMLAttributes<HTMLParagraphElement> {
  children: ReactNode;
}

export function CardDescription({ className, children, ...props }: CardDescriptionProps) {
  return (
    <p
      className={twMerge('text-sm text-gray-600 dark:text-gray-400', className)}
      {...props}
    >
      {children}
    </p>
  );
}

/**
 * Card Content Component
 */
export interface CardContentProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
}

export function CardContent({ className, children, ...props }: CardContentProps) {
  return (
    <div className={twMerge('text-gray-700 dark:text-gray-300', className)} {...props}>
      {children}
    </div>
  );
}

/**
 * Card Footer Component
 */
export interface CardFooterProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
}

export function CardFooter({ className, children, ...props }: CardFooterProps) {
  return (
    <div
      className={twMerge(
        'border-t border-gray-200 dark:border-gray-700 pt-4 mt-4',
        'flex items-center justify-end gap-2',
        className
      )}
      {...props}
    >
      {children}
    </div>
  );
}

/**
 * Usage Examples:
 *
 * // Simple card
 * <Card>
 *   <p>Content</p>
 * </Card>
 *
 * // Card with compound components
 * <Card variant="elevated" padding="lg">
 *   <CardHeader>
 *     <CardTitle>Card Title</CardTitle>
 *     <CardDescription>Card description goes here</CardDescription>
 *   </CardHeader>
 *   <CardContent>
 *     <p>Main content area</p>
 *   </CardContent>
 *   <CardFooter>
 *     <Button variant="secondary">Cancel</Button>
 *     <Button>Submit</Button>
 *   </CardFooter>
 * </Card>
 *
 * // Hoverable card
 * <Card hoverable onClick={() => console.log('clicked')}>
 *   <CardTitle>Clickable Card</CardTitle>
 * </Card>
 *
 * // Custom styling (properly merged with tailwind-merge)
 * <Card className="bg-gradient-to-r from-blue-500 to-purple-600">
 *   <CardContent className="text-white">
 *     Custom gradient background
 *   </CardContent>
 * </Card>
 */
