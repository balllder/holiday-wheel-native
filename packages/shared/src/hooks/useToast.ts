import React, { useState, useCallback, useMemo } from 'react';
import { Toast, ToastType, ToastProps } from '../components/Toast';

export interface UseToastOptions {
  /** Default duration for toasts (default: 3000ms) */
  defaultDuration?: number;
  /** Default type for toasts (default: 'info') */
  defaultType?: ToastType;
}

export interface ToastState {
  message: string;
  type: ToastType;
  duration: number;
}

export interface UseToastReturn {
  /** Show a toast notification */
  showToast: (message: string, type?: ToastType, duration?: number) => void;
  /** Hide the current toast */
  hideToast: () => void;
  /** Toast component to render */
  ToastComponent: React.FC;
  /** Current visibility state */
  isVisible: boolean;
}

/**
 * useToast - Hook for managing toast notifications
 *
 * Provides a simple API to show/hide toast notifications with
 * automatic state management.
 *
 * @example
 * function MyComponent() {
 *   const { showToast, ToastComponent } = useToast();
 *
 *   const handleSave = async () => {
 *     try {
 *       await saveData();
 *       showToast('Saved successfully!', 'success');
 *     } catch (error) {
 *       showToast('Failed to save', 'error');
 *     }
 *   };
 *
 *   return (
 *     <View>
 *       <Button onPress={handleSave} title="Save" />
 *       <ToastComponent />
 *     </View>
 *   );
 * }
 *
 * @example
 * // With custom defaults
 * const { showToast, ToastComponent } = useToast({
 *   defaultDuration: 5000,
 *   defaultType: 'info',
 * });
 */
export function useToast(options: UseToastOptions = {}): UseToastReturn {
  const { defaultDuration = 3000, defaultType = 'info' } = options;

  const [visible, setVisible] = useState(false);
  const [toastState, setToastState] = useState<ToastState>({
    message: '',
    type: defaultType,
    duration: defaultDuration,
  });

  const showToast = useCallback(
    (message: string, type?: ToastType, duration?: number) => {
      setToastState({
        message,
        type: type ?? defaultType,
        duration: duration ?? defaultDuration,
      });
      setVisible(true);
    },
    [defaultType, defaultDuration]
  );

  const hideToast = useCallback(() => {
    setVisible(false);
  }, []);

  // Memoize the ToastComponent to prevent unnecessary re-renders
  const ToastComponent = useMemo(() => {
    const Component: React.FC = () =>
      React.createElement(Toast, {
        message: toastState.message,
        visible: visible,
        onHide: hideToast,
        duration: toastState.duration,
        type: toastState.type,
        testID: 'toast-notification',
      } as ToastProps);

    Component.displayName = 'ToastComponent';
    return Component;
  }, [toastState.message, toastState.type, toastState.duration, visible, hideToast]);

  return {
    showToast,
    hideToast,
    ToastComponent,
    isVisible: visible,
  };
}

/**
 * Convenience shortcuts for common toast types
 */
export function useToastHelpers(options: UseToastOptions = {}) {
  const { showToast, hideToast, ToastComponent, isVisible } = useToast(options);

  const showInfo = useCallback(
    (message: string, duration?: number) => {
      showToast(message, 'info', duration);
    },
    [showToast]
  );

  const showSuccess = useCallback(
    (message: string, duration?: number) => {
      showToast(message, 'success', duration);
    },
    [showToast]
  );

  const showError = useCallback(
    (message: string, duration?: number) => {
      showToast(message, 'error', duration);
    },
    [showToast]
  );

  const showWarning = useCallback(
    (message: string, duration?: number) => {
      showToast(message, 'warning', duration);
    },
    [showToast]
  );

  return {
    showToast,
    showInfo,
    showSuccess,
    showError,
    showWarning,
    hideToast,
    ToastComponent,
    isVisible,
  };
}

export default useToast;
