import { ButtonHTMLAttributes, ReactNode } from 'react';
import clsx from 'clsx';

/**
 * Button Component - Tailwind Best Practices Example
 *
 * Demonstrates:
 * - Variant-based styling with object mapping
 * - Using clsx for conditional classes
 * - Responsive design
 * - Accessibility (disabled, loading states)
 * - Type safety with TypeScript
 */

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** Visual style variant */
  variant?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'danger';

  /** Size variant */
  size?: 'sm' | 'md' | 'lg';

  /** Loading state */
  loading?: boolean;

  /** Icon to display before text */
  leftIcon?: ReactNode;

  /** Icon to display after text */
  rightIcon?: ReactNode;

  /** Full width button */
  fullWidth?: boolean;

  children: ReactNode;
}

/**
 * Variant class mappings
 *
 * Pattern: Define variants as objects for better maintainability
 * instead of nested ternaries or long conditional strings
 */
const variantClasses = {
  primary:
    'bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500 disabled:bg-blue-300',
  secondary:
    'bg-gray-200 text-gray-800 hover:bg-gray-300 focus:ring-gray-500 disabled:bg-gray-100 disabled:text-gray-400',
  outline:
    'border-2 border-blue-600 text-blue-600 hover:bg-blue-50 focus:ring-blue-500 disabled:border-blue-300 disabled:text-blue-300',
  ghost:
    'text-gray-700 hover:bg-gray-100 focus:ring-gray-500 disabled:text-gray-400',
  danger:
    'bg-red-600 text-white hover:bg-red-700 focus:ring-red-500 disabled:bg-red-300',
};

const sizeClasses = {
  sm: 'px-3 py-1.5 text-sm',
  md: 'px-4 py-2 text-base',
  lg: 'px-6 py-3 text-lg',
};

export function Button({
  variant = 'primary',
  size = 'md',
  loading = false,
  disabled = false,
  leftIcon,
  rightIcon,
  fullWidth = false,
  className,
  children,
  ...props
}: ButtonProps) {
  return (
    <button
      className={clsx(
        // Base styles - always applied
        'inline-flex items-center justify-center gap-2',
        'font-semibold rounded-lg',
        'transition-colors duration-150',
        'focus:outline-none focus:ring-2 focus:ring-offset-2',
        'disabled:cursor-not-allowed disabled:opacity-60',

        // Variant styles
        variantClasses[variant],

        // Size styles
        sizeClasses[size],

        // Conditional styles
        {
          'w-full': fullWidth,
          'cursor-wait': loading,
        },

        // Custom className (allows overrides)
        className
      )}
      disabled={disabled || loading}
      {...props}
    >
      {/* Loading spinner */}
      {loading && (
        <svg
          className="animate-spin h-4 w-4"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
        >
          <circle
            className="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            strokeWidth="4"
          />
          <path
            className="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          />
        </svg>
      )}

      {/* Left icon */}
      {!loading && leftIcon && <span className="flex-shrink-0">{leftIcon}</span>}

      {/* Button text */}
      <span>{children}</span>

      {/* Right icon */}
      {!loading && rightIcon && <span className="flex-shrink-0">{rightIcon}</span>}
    </button>
  );
}

/**
 * Usage Examples:
 *
 * // Primary button
 * <Button>Click Me</Button>
 *
 * // Secondary button with icon
 * <Button variant="secondary" leftIcon={<Icon />}>
 *   Save
 * </Button>
 *
 * // Loading state
 * <Button loading>Processing...</Button>
 *
 * // Danger button
 * <Button variant="danger" size="lg">
 *   Delete
 * </Button>
 *
 * // Custom styling (via className prop)
 * <Button className="shadow-xl">
 *   Custom Shadow
 * </Button>
 */
