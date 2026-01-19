/* eslint-env jest */

// Variable names must start with 'mock' for Jest to allow them in mock factories

// Mock AsyncStorage
const mockAsyncStorage = {
  setItem: jest.fn(() => Promise.resolve()),
  getItem: jest.fn(() => Promise.resolve(null)),
  removeItem: jest.fn(() => Promise.resolve()),
  clear: jest.fn(() => Promise.resolve()),
  getAllKeys: jest.fn(() => Promise.resolve([])),
  multiGet: jest.fn(() => Promise.resolve([])),
  multiSet: jest.fn(() => Promise.resolve()),
  multiRemove: jest.fn(() => Promise.resolve()),
};

jest.mock('@react-native-async-storage/async-storage', () => ({
  __esModule: true,
  default: mockAsyncStorage,
  ...mockAsyncStorage,
}));

// Make mock accessible globally for tests
global.mockAsyncStorage = mockAsyncStorage;

// Mock react-native
const mockStyleSheet = {
  create: (styles) => styles,
  flatten: (style) => {
    if (Array.isArray(style)) {
      return Object.assign({}, ...style.filter(Boolean));
    }
    return style || {};
  },
};

jest.mock('react-native', () => {
  const React = require('react');
  const mockComponent = (name) => {
    return ({ children, ...props }) =>
      React.createElement(name, props, children);
  };
  return {
    View: mockComponent('View'),
    Text: mockComponent('Text'),
    TouchableOpacity: mockComponent('TouchableOpacity'),
    Modal: mockComponent('Modal'),
    Pressable: mockComponent('Pressable'),
    StyleSheet: mockStyleSheet,
    Animated: {
      View: mockComponent('Animated.View'),
      Text: mockComponent('Animated.Text'),
      Value: jest.fn(() => ({
        setValue: jest.fn(),
        interpolate: jest.fn(() => ({ __getValue: jest.fn() })),
      })),
      timing: jest.fn(() => ({
        start: jest.fn((cb) => cb && cb()),
        stop: jest.fn(),
      })),
      spring: jest.fn(() => ({
        start: jest.fn((cb) => cb && cb()),
        stop: jest.fn(),
      })),
      sequence: jest.fn(() => ({
        start: jest.fn((cb) => cb && cb()),
        stop: jest.fn(),
      })),
      parallel: jest.fn(() => ({
        start: jest.fn((cb) => cb && cb()),
        stop: jest.fn(),
      })),
      delay: jest.fn(() => ({
        start: jest.fn((cb) => cb && cb()),
        stop: jest.fn(),
      })),
      loop: jest.fn((animation) => ({
        start: jest.fn((cb) => {
          animation.start && animation.start();
          cb && cb();
        }),
        stop: jest.fn(),
      })),
      createAnimatedComponent: jest.fn((component) => component),
    },
    Easing: {
      linear: jest.fn(),
      ease: jest.fn(),
      out: jest.fn((fn) => fn),
    },
    Platform: {
      OS: 'ios',
      select: jest.fn((obj) => obj.ios || obj.default),
    },
    Dimensions: {
      get: jest.fn(() => ({ width: 400, height: 800 })),
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
    },
  };
});

// Mock react-native-reanimated
jest.mock('react-native-reanimated', () => {
  const React = require('react');
  const mockComponent = (name) => {
    return ({ children, ...props }) =>
      React.createElement(name, props, children);
  };

  // Mock shared value that stores and returns values
  const mockUseSharedValue = (initialValue) => {
    const ref = { value: initialValue };
    return ref;
  };

  // Mock animated style that returns empty object
  const mockUseAnimatedStyle = (styleGetter) => {
    // In tests, return an empty object
    return {};
  };

  // Mock animation functions that immediately complete
  const mockWithTiming = (toValue, config, callback) => {
    if (callback) callback(true);
    return toValue;
  };

  const mockWithSpring = (toValue, config, callback) => {
    if (callback) callback(true);
    return toValue;
  };

  const mockWithSequence = (...values) => {
    return values[values.length - 1];
  };

  const mockWithDelay = (delay, value) => {
    return value;
  };

  const mockRunOnJS = (fn) => fn;

  const mockInterpolate = (value, inputRange, outputRange) => {
    return outputRange[0];
  };

  return {
    __esModule: true,
    default: {
      View: mockComponent('Animated.View'),
      Text: mockComponent('Animated.Text'),
      Image: mockComponent('Animated.Image'),
      createAnimatedComponent: (component) => component,
    },
    useSharedValue: mockUseSharedValue,
    useAnimatedStyle: mockUseAnimatedStyle,
    withTiming: mockWithTiming,
    withSpring: mockWithSpring,
    withSequence: mockWithSequence,
    withDelay: mockWithDelay,
    runOnJS: mockRunOnJS,
    interpolate: mockInterpolate,
    Easing: {
      linear: jest.fn(),
      ease: jest.fn(),
      inOut: jest.fn(() => jest.fn()),
    },
  };
});

// Mock react-native-svg
jest.mock('react-native-svg', () => {
  const React = require('react');
  const mockComponent = (name) => {
    return ({ children, ...props }) =>
      React.createElement(name, props, children);
  };
  return {
    __esModule: true,
    default: mockComponent('Svg'),
    Svg: mockComponent('Svg'),
    Path: mockComponent('Path'),
    G: mockComponent('G'),
    Circle: mockComponent('Circle'),
    Text: mockComponent('SvgText'),
    Rect: mockComponent('Rect'),
    Line: mockComponent('Line'),
    Defs: mockComponent('Defs'),
    LinearGradient: mockComponent('LinearGradient'),
    Stop: mockComponent('Stop'),
  };
});
