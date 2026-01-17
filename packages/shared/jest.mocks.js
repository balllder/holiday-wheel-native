// Mock react-native for shared package tests
const mockComponent = (name) => {
  const React = require('react');
  return ({ children, ...props }) => React.createElement(name, props, children);
};

const StyleSheet = {
  create: (styles) => styles,
  flatten: (style) => {
    if (Array.isArray(style)) {
      return Object.assign({}, ...style.filter(Boolean));
    }
    return style || {};
  },
};

module.exports = {
  View: mockComponent('View'),
  Text: mockComponent('Text'),
  StyleSheet,
  Animated: {
    View: mockComponent('Animated.View'),
    Text: mockComponent('Animated.Text'),
    Value: jest.fn(() => ({
      setValue: jest.fn(),
      interpolate: jest.fn(() => ({ __getValue: jest.fn() })),
    })),
    timing: jest.fn(() => ({
      start: jest.fn((cb) => cb && cb()),
    })),
    spring: jest.fn(() => ({
      start: jest.fn((cb) => cb && cb()),
    })),
    createAnimatedComponent: jest.fn((component) => component),
  },
  Easing: {
    linear: jest.fn(),
    ease: jest.fn(),
    out: jest.fn((fn) => fn),
  },
  // SVG mocks
  Svg: mockComponent('Svg'),
  Path: mockComponent('Path'),
  G: mockComponent('G'),
  Circle: mockComponent('Circle'),
  Rect: mockComponent('Rect'),
  // Default export for SVG
  default: mockComponent('Svg'),
};
