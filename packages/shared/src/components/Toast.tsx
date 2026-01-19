import React, { useEffect, useRef, useCallback, useState } from 'react';
import {
  Animated,
  StyleSheet,
  Text,
  View,
  ViewStyle,
  Platform,
} from 'react-native';
import { MODERN_THEME } from '../constants';

export type ToastType = 'info' | 'success' | 'error' | 'warning';

export interface ToastProps {
  /** Toast message to display */
  message: string;
  /** Whether the toast is visible */
  visible: boolean;
  /** Callback when toast should hide */
  onHide: () => void;
  /** Auto-dismiss duration in ms (default: 3000) */
  duration?: number;
  /** Toast type for styling and icon (default: 'info') */
  type?: ToastType;
  /** Test ID for testing */
  testID?: string;
}

/**
 * Get icon character based on toast type
 */
const getTypeIcon = (type: ToastType): string => {
  switch (type) {
    case 'success':
      return '\u2713'; // Checkmark
    case 'error':
      return '\u2717'; // X mark
    case 'warning':
      return '\u26A0'; // Warning triangle
    case 'info':
    default:
      return '\u2139'; // Info circle
  }
};

/**
 * Get accent color based on toast type
 */
const getTypeColor = (type: ToastType): string => {
  switch (type) {
    case 'success':
      return MODERN_THEME.colors.success;
    case 'error':
      return MODERN_THEME.colors.danger;
    case 'warning':
      return MODERN_THEME.colors.warning;
    case 'info':
    default:
      return MODERN_THEME.colors.info;
  }
};

/**
 * Get glow color based on toast type
 */
const getTypeGlow = (type: ToastType): string => {
  switch (type) {
    case 'success':
      return MODERN_THEME.colors.successGlow;
    case 'error':
      return MODERN_THEME.colors.dangerGlow;
    case 'warning':
      return MODERN_THEME.colors.warningGlow;
    case 'info':
    default:
      return MODERN_THEME.colors.infoGlow;
  }
};

/**
 * Toast - Non-modal overlay notification
 *
 * Features:
 * - Appears at top of screen with safe area awareness
 * - Smooth fade + slide animation
 * - Type-based styling (info, success, error, warning)
 * - Auto-dismiss after configurable duration
 * - Gold accent border matching app theme
 * - Dark semi-transparent background with blur effect
 *
 * @example
 * <Toast
 *   message="Game saved successfully!"
 *   visible={showToast}
 *   onHide={() => setShowToast(false)}
 *   type="success"
 * />
 */
export function Toast({
  message,
  visible,
  onHide,
  duration = 3000,
  type = 'info',
  testID,
}: ToastProps): React.JSX.Element | null {
  const opacity = useRef(new Animated.Value(0)).current;
  const translateY = useRef(new Animated.Value(-100)).current;
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [shouldRender, setShouldRender] = useState(false);

  const hideToast = useCallback(() => {
    Animated.parallel([
      Animated.timing(opacity, {
        toValue: 0,
        duration: MODERN_THEME.animation.normal,
        useNativeDriver: true,
      }),
      Animated.timing(translateY, {
        toValue: -100,
        duration: MODERN_THEME.animation.normal,
        useNativeDriver: true,
      }),
    ]).start(() => {
      setShouldRender(false);
      onHide();
    });
  }, [opacity, translateY, onHide]);

  const showToast = useCallback(() => {
    // Clear any existing timer
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }

    // Enable rendering
    setShouldRender(true);

    // Animate in
    Animated.parallel([
      Animated.timing(opacity, {
        toValue: 1,
        duration: MODERN_THEME.animation.normal,
        useNativeDriver: true,
      }),
      Animated.spring(translateY, {
        toValue: 0,
        friction: 8,
        tension: 80,
        useNativeDriver: true,
      }),
    ]).start();

    // Set auto-dismiss timer
    timerRef.current = setTimeout(() => {
      hideToast();
    }, duration);
  }, [opacity, translateY, duration, hideToast]);

  useEffect(() => {
    if (visible) {
      showToast();
    }

    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      }
    };
  }, [visible, showToast]);

  // Don't render if not visible
  if (!shouldRender) {
    return null;
  }

  const typeColor = getTypeColor(type);
  const typeIcon = getTypeIcon(type);
  const typeGlow = getTypeGlow(type);

  const containerStyle: ViewStyle = {
    borderLeftColor: typeColor,
    shadowColor: typeGlow,
  };

  return (
    <Animated.View
      style={[
        styles.container,
        containerStyle,
        {
          opacity,
          transform: [{ translateY }],
        },
      ]}
      testID={testID}
      pointerEvents="none"
    >
      <View style={styles.content}>
        <View style={[styles.iconContainer, { backgroundColor: typeColor }]}>
          <Text style={styles.icon}>{typeIcon}</Text>
        </View>
        <View style={styles.textContainer}>
          <Text style={styles.message} numberOfLines={3}>
            {message}
          </Text>
        </View>
      </View>
      <View style={[styles.accentBorder, { backgroundColor: MODERN_THEME.colors.primary }]} />
    </Animated.View>
  );
}

const styles = StyleSheet.create({
  container: {
    position: 'absolute',
    top: Platform.OS === 'ios' ? 60 : 40, // Safe area for notch
    left: 16,
    right: 16,
    backgroundColor: 'rgba(26, 10, 62, 0.95)', // Dark semi-transparent
    borderRadius: MODERN_THEME.borderRadius.lg,
    borderLeftWidth: 4,
    borderWidth: 1,
    borderColor: MODERN_THEME.colors.primary, // Gold accent border
    zIndex: MODERN_THEME.zIndex.toast,
    // Shadow/elevation
    ...MODERN_THEME.shadows.large,
    shadowOpacity: 0.6,
    elevation: 15,
    // Glassmorphism effect simulation
    overflow: 'hidden',
  },
  content: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 14,
    paddingHorizontal: 16,
  },
  iconContainer: {
    width: 32,
    height: 32,
    borderRadius: MODERN_THEME.borderRadius.full,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  icon: {
    fontSize: 16,
    color: MODERN_THEME.colors.text,
    fontWeight: MODERN_THEME.typography.fontWeight.bold,
  },
  textContainer: {
    flex: 1,
  },
  message: {
    fontSize: 16,
    fontWeight: MODERN_THEME.typography.fontWeight.semibold,
    color: MODERN_THEME.colors.text,
    lineHeight: 22,
  },
  accentBorder: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    height: 2,
    opacity: 0.6,
  },
});

export default Toast;
