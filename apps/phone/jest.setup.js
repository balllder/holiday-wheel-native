/* eslint-env jest */
// Mock AsyncStorage
jest.mock('@react-native-async-storage/async-storage', () =>
  require('@react-native-async-storage/async-storage/jest/async-storage-mock')
);

// Mock react-native-camera-kit
jest.mock('react-native-camera-kit', () => {
  const React = require('react');
  return {
    Camera: ({ onReadCode, ...props }) =>
      React.createElement('Camera', {
        ...props,
        testID: 'camera',
        onReadCode,
      }),
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
