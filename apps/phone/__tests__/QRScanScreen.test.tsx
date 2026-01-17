import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { Alert } from 'react-native';
import { QRScanScreen } from '../src/screens/QRScanScreen';
import { authService } from '@holiday-wheel/shared';

// Mock Alert
jest.spyOn(Alert, 'alert');

// Mock authService
jest.mock('@holiday-wheel/shared', () => ({
  authService: {
    setBaseUrl: jest.fn(),
  },
}));

// Create mock navigation
const createMockNavigation = () => ({
  replace: jest.fn(),
  goBack: jest.fn(),
  navigate: jest.fn(),
});

describe('QRScanScreen', () => {
  let mockNavigation: ReturnType<typeof createMockNavigation>;

  beforeEach(() => {
    jest.clearAllMocks();
    mockNavigation = createMockNavigation();
  });

  describe('rendering', () => {
    it('renders camera and UI elements', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <QRScanScreen navigation={mockNavigation as never} />
        );
      });

      const json = tree?.toJSON();
      expect(json).not.toBeNull();
    });

    it('shows scan instructions', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <QRScanScreen navigation={mockNavigation as never} />
        );
      });

      const instance = tree?.root;
      const texts = instance?.findAllByType('Text' as never);
      const textContents = texts?.map((t) => {
        const children = t.props.children;
        return typeof children === 'string' ? children : '';
      });

      expect(textContents?.some((t) => t.includes('Scan QR Code'))).toBe(true);
    });
  });

  describe('QR code handling', () => {
    it('parses valid QR code and navigates', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <QRScanScreen navigation={mockNavigation as never} />
        );
      });

      // Find Camera and trigger QR code read
      const camera = tree?.root.findByProps({ testID: 'camera' });
      act(() => {
        camera?.props.onReadCode({
          nativeEvent: {
            codeStringValue:
              'holidaywheel://join?room=TEST123&server=http%3A%2F%2F192.168.1.100%3A5000',
          },
        });
      });

      expect(authService.setBaseUrl).toHaveBeenCalledWith(
        'http://192.168.1.100:5000'
      );
      expect(mockNavigation.replace).toHaveBeenCalledWith('Controller', {
        room: 'TEST123',
      });
    });

    it('shows alert for invalid QR code prefix', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <QRScanScreen navigation={mockNavigation as never} />
        );
      });

      const camera = tree?.root.findByProps({ testID: 'camera' });
      act(() => {
        camera?.props.onReadCode({
          nativeEvent: {
            codeStringValue: 'https://example.com/not-valid',
          },
        });
      });

      expect(Alert.alert).toHaveBeenCalledWith(
        'Invalid QR Code',
        'This QR code is not a Holiday Wheel game code.'
      );
      expect(mockNavigation.replace).not.toHaveBeenCalled();
    });

    it('shows alert for missing room parameter', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <QRScanScreen navigation={mockNavigation as never} />
        );
      });

      const camera = tree?.root.findByProps({ testID: 'camera' });
      act(() => {
        camera?.props.onReadCode({
          nativeEvent: {
            codeStringValue: 'holidaywheel://join?server=http%3A%2F%2Ftest.com',
          },
        });
      });

      expect(Alert.alert).toHaveBeenCalledWith(
        'Invalid QR Code',
        'Missing room or server information.'
      );
      expect(mockNavigation.replace).not.toHaveBeenCalled();
    });

    it('shows alert for missing server parameter', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <QRScanScreen navigation={mockNavigation as never} />
        );
      });

      const camera = tree?.root.findByProps({ testID: 'camera' });
      act(() => {
        camera?.props.onReadCode({
          nativeEvent: {
            codeStringValue: 'holidaywheel://join?room=TEST123',
          },
        });
      });

      expect(Alert.alert).toHaveBeenCalledWith(
        'Invalid QR Code',
        'Missing room or server information.'
      );
    });

    it('prevents duplicate scans', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <QRScanScreen navigation={mockNavigation as never} />
        );
      });

      const camera = tree?.root.findByProps({ testID: 'camera' });
      const validQR =
        'holidaywheel://join?room=TEST123&server=http%3A%2F%2Ftest.com';

      // First scan
      act(() => {
        camera?.props.onReadCode({
          nativeEvent: { codeStringValue: validQR },
        });
      });

      // Second scan should be ignored
      act(() => {
        camera?.props.onReadCode({
          nativeEvent: { codeStringValue: validQR },
        });
      });

      expect(mockNavigation.replace).toHaveBeenCalledTimes(1);
    });
  });

  describe('cancel button', () => {
    it('renders cancel button', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <QRScanScreen navigation={mockNavigation as never} />
        );
      });

      // Verify Cancel text is rendered
      const allTexts = tree?.root.findAllByType('Text' as never);
      const cancelText = allTexts?.find(
        (t) => t.props.children === 'Cancel'
      );

      expect(cancelText).toBeDefined();
    });
  });
});
