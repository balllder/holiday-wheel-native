/* eslint-env jest, node */
/* global globalThis */

// Configure React act() environment to suppress act() warnings
globalThis.IS_REACT_ACT_ENVIRONMENT = true;

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

// Mock Google Sign-In
jest.mock('@react-native-google-signin/google-signin', () => ({
  GoogleSignin: {
    configure: jest.fn(),
    hasPlayServices: jest.fn(() => Promise.resolve(true)),
    signIn: jest.fn(() =>
      Promise.resolve({ data: { idToken: 'google-id-token' } })
    ),
  },
  statusCodes: {
    SIGN_IN_CANCELLED: 'SIGN_IN_CANCELLED',
    IN_PROGRESS: 'IN_PROGRESS',
    PLAY_SERVICES_NOT_AVAILABLE: 'PLAY_SERVICES_NOT_AVAILABLE',
  },
  isErrorWithCode: jest.fn(() => false),
}));

// Mock Apple Auth
jest.mock('@invertase/react-native-apple-authentication', () => ({
  __esModule: true,
  default: {
    performRequest: jest.fn(() =>
      Promise.resolve({
        user: 'apple-user-id',
        identityToken: 'apple-identity-token',
        email: 'apple@example.com',
        fullName: { givenName: 'Apple', familyName: 'User' },
      })
    ),
    getCredentialStateForUser: jest.fn(() => Promise.resolve(1)),
    Operation: { LOGIN: 0 },
    Scope: { EMAIL: 0, FULL_NAME: 1 },
    State: { AUTHORIZED: 1 },
    Error: { CANCELED: '1001' },
  },
}));
