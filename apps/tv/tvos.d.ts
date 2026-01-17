// Type declarations for tvOS-specific React Native APIs
/* eslint-disable @typescript-eslint/no-unused-vars */

import type { ViewProps as RNViewProps } from 'react-native';

declare module 'react-native' {
  /**
   * TVFocusGuideView - tvOS focus management container
   */
  export interface TVFocusGuideViewProps extends ViewProps {
    destinations?: React.RefObject<unknown>[];
    autoFocus?: boolean;
    trapFocusUp?: boolean;
    trapFocusDown?: boolean;
    trapFocusLeft?: boolean;
    trapFocusRight?: boolean;
  }

  export const TVFocusGuideView: React.ComponentType<TVFocusGuideViewProps>;

  /**
   * TV Event types
   */
  export interface HWEvent {
    eventType:
      | 'blur'
      | 'focus'
      | 'select'
      | 'longSelect'
      | 'up'
      | 'down'
      | 'left'
      | 'right'
      | 'playPause'
      | 'menu'
      | 'swipeUp'
      | 'swipeDown'
      | 'swipeLeft'
      | 'swipeRight'
      | 'pan';
    eventKeyAction?: -1 | 0 | 1;
    tag?: number;
    body?: {
      x?: number;
      y?: number;
      velocityX?: number;
      velocityY?: number;
      state?: string;
    };
  }

  /**
   * useTVEventHandler - Hook for handling TV remote events
   */
  export function useTVEventHandler(
    callback: (evt: HWEvent) => void
  ): void;

  /**
   * hasTVPreferredFocus - prop for setting initial focus
   */
  export interface ViewProps {
    hasTVPreferredFocus?: boolean;
    isTVSelectable?: boolean;
  }

  export interface TouchableOpacityProps {
    hasTVPreferredFocus?: boolean;
    isTVSelectable?: boolean;
  }
}
