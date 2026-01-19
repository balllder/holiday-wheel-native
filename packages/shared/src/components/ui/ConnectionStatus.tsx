import React, { useEffect, useRef, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  Animated,
  TouchableOpacity,
  ViewStyle,
  StyleProp,
} from 'react-native';
import { useGameStore, ConnectionStatus as ConnectionStatusType } from '../../stores/gameStore';
import { theme } from '../../constants/theme';

export interface ConnectionStatusProps {
  /** Custom style for the container */
  style?: StyleProp<ViewStyle>;
  /** Callback when retry button is pressed */
  onRetry?: () => void;
  /** Whether to show in compact mode (dot only) */
  compact?: boolean;
  /** Test ID for testing */
  testID?: string;
}

interface StatusConfig {
  color: string;
  glowColor: string;
  label: string;
  showPulse: boolean;
  showRetry: boolean;
}

const getStatusConfig = (
  status: ConnectionStatusType,
  reconnectAttempt: number
): StatusConfig => {
  switch (status) {
    case 'connected':
      return {
        color: theme.colors.success,
        glowColor: theme.colors.successGlow,
        label: 'Connected',
        showPulse: false,
        showRetry: false,
      };
    case 'connecting':
      return {
        color: theme.colors.warning,
        glowColor: theme.colors.warningGlow,
        label: 'Connecting...',
        showPulse: true,
        showRetry: false,
      };
    case 'reconnecting':
      return {
        color: '#f97316', // Orange
        glowColor: 'rgba(249, 115, 22, 0.4)',
        label: `Reconnecting (${reconnectAttempt})...`,
        showPulse: true,
        showRetry: false,
      };
    case 'disconnected':
    default:
      return {
        color: theme.colors.danger,
        glowColor: theme.colors.dangerGlow,
        label: 'Disconnected',
        showPulse: false,
        showRetry: true,
      };
  }
};

/**
 * ConnectionStatus - Shows the current connection status with visual indicators
 *
 * Features:
 * - Color-coded status dot (green/yellow/orange/red)
 * - Pulsing animation for connecting/reconnecting states
 * - Reconnect attempt counter during reconnection
 * - Retry button when disconnected
 * - Compact mode for minimal UI
 *
 * @example
 * // Full display
 * <ConnectionStatus onRetry={handleRetry} />
 *
 * // Compact mode (dot only)
 * <ConnectionStatus compact />
 */
export function ConnectionStatus({
  style,
  onRetry,
  compact = false,
  testID,
}: ConnectionStatusProps): React.JSX.Element {
  const connectionStatus = useGameStore((state) => state.connectionStatus);
  const reconnectAttempt = useGameStore((state) => state.reconnectAttempt);

  const pulseAnim = useRef(new Animated.Value(1)).current;
  const opacityAnim = useRef(new Animated.Value(1)).current;
  const scaleAnim = useRef(new Animated.Value(1)).current;

  const statusConfig = getStatusConfig(connectionStatus, reconnectAttempt);

  // Pulsing animation for connecting/reconnecting states
  useEffect(() => {
    if (statusConfig.showPulse) {
      const pulse = Animated.loop(
        Animated.parallel([
          Animated.sequence([
            Animated.timing(pulseAnim, {
              toValue: 1.3,
              duration: 800,
              useNativeDriver: true,
            }),
            Animated.timing(pulseAnim, {
              toValue: 1,
              duration: 800,
              useNativeDriver: true,
            }),
          ]),
          Animated.sequence([
            Animated.timing(opacityAnim, {
              toValue: 0.4,
              duration: 800,
              useNativeDriver: true,
            }),
            Animated.timing(opacityAnim, {
              toValue: 1,
              duration: 800,
              useNativeDriver: true,
            }),
          ]),
        ])
      );
      pulse.start();
      return () => pulse.stop();
    } else {
      // Reset animations when not pulsing
      pulseAnim.setValue(1);
      opacityAnim.setValue(1);
    }
  }, [statusConfig.showPulse, pulseAnim, opacityAnim]);

  // Scale animation on status change
  useEffect(() => {
    Animated.sequence([
      Animated.timing(scaleAnim, {
        toValue: 1.2,
        duration: 150,
        useNativeDriver: true,
      }),
      Animated.spring(scaleAnim, {
        toValue: 1,
        friction: 5,
        tension: 100,
        useNativeDriver: true,
      }),
    ]).start();
  }, [connectionStatus, scaleAnim]);

  const handleRetryPress = useCallback(() => {
    onRetry?.();
  }, [onRetry]);

  if (compact) {
    return (
      <Animated.View
        style={[
          styles.compactContainer,
          {
            transform: [{ scale: scaleAnim }],
          },
          style,
        ]}
        testID={testID}
      >
        <Animated.View
          style={[
            styles.statusDot,
            {
              backgroundColor: statusConfig.color,
              shadowColor: statusConfig.color,
              transform: [{ scale: pulseAnim }],
              opacity: opacityAnim,
            },
          ]}
          testID={testID ? `${testID}-dot` : undefined}
        />
      </Animated.View>
    );
  }

  return (
    <Animated.View
      style={[
        styles.container,
        {
          transform: [{ scale: scaleAnim }],
        },
        style,
      ]}
      testID={testID}
    >
      <View style={styles.statusRow}>
        <Animated.View
          style={[
            styles.statusDot,
            {
              backgroundColor: statusConfig.color,
              shadowColor: statusConfig.color,
              transform: [{ scale: pulseAnim }],
              opacity: opacityAnim,
            },
          ]}
          testID={testID ? `${testID}-dot` : undefined}
        />
        <Text
          style={[styles.statusLabel, { color: statusConfig.color }]}
          testID={testID ? `${testID}-label` : undefined}
        >
          {statusConfig.label}
        </Text>
      </View>

      {statusConfig.showRetry && onRetry && (
        <TouchableOpacity
          style={styles.retryButton}
          onPress={handleRetryPress}
          activeOpacity={0.7}
          testID={testID ? `${testID}-retry` : undefined}
        >
          <Text style={styles.retryText}>Retry</Text>
        </TouchableOpacity>
      )}
    </Animated.View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: theme.colors.glassBackground,
    borderRadius: theme.borderRadius.lg,
    paddingVertical: theme.spacing.sm,
    paddingHorizontal: theme.spacing.md,
    borderWidth: 1,
    borderColor: theme.colors.glassBorder,
  },
  compactContainer: {
    padding: theme.spacing.xs,
  },
  statusRow: {
    flexDirection: 'row',
    alignItems: 'center',
    flex: 1,
  },
  statusDot: {
    width: 10,
    height: 10,
    borderRadius: theme.borderRadius.full,
    marginRight: theme.spacing.sm,
    shadowOffset: { width: 0, height: 0 },
    shadowOpacity: 0.8,
    shadowRadius: 6,
    elevation: 4,
  },
  statusLabel: {
    fontSize: theme.typography.fontSize.sm,
    fontWeight: theme.typography.fontWeight.medium,
  },
  retryButton: {
    backgroundColor: theme.colors.danger,
    paddingVertical: theme.spacing.xs,
    paddingHorizontal: theme.spacing.md,
    borderRadius: theme.borderRadius.sm,
    marginLeft: theme.spacing.md,
  },
  retryText: {
    color: theme.colors.text,
    fontSize: theme.typography.fontSize.sm,
    fontWeight: theme.typography.fontWeight.semibold,
  },
});

export default ConnectionStatus;
