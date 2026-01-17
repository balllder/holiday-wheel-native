/* eslint-env jest */
// Mock AsyncStorage
jest.mock('@react-native-async-storage/async-storage', () =>
  require('@react-native-async-storage/async-storage/jest/async-storage-mock')
);

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
    Text: mockComponent('Text'),
    Rect: mockComponent('Rect'),
    Line: mockComponent('Line'),
    Defs: mockComponent('Defs'),
    LinearGradient: mockComponent('LinearGradient'),
    Stop: mockComponent('Stop'),
  };
});

// Mock react-native-qrcode-svg
jest.mock('react-native-qrcode-svg', () => {
  const React = require('react');
  return {
    __esModule: true,
    default: (props) => React.createElement('QRCode', props),
  };
});

// Mock tvOS-specific APIs via global
global.useTVEventHandler = jest.fn();
