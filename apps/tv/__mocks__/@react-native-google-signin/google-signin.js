// Mock for @react-native-google-signin/google-signin
const statusCodes = {
  SIGN_IN_CANCELLED: 'SIGN_IN_CANCELLED',
  IN_PROGRESS: 'IN_PROGRESS',
  PLAY_SERVICES_NOT_AVAILABLE: 'PLAY_SERVICES_NOT_AVAILABLE',
};

const GoogleSignin = {
  configure: jest.fn(),
  hasPlayServices: jest.fn(() => Promise.resolve(true)),
  signIn: jest.fn(() => Promise.resolve({ data: { idToken: 'mock-token' } })),
  signOut: jest.fn(() => Promise.resolve()),
  isSignedIn: jest.fn(() => Promise.resolve(false)),
  getCurrentUser: jest.fn(() => Promise.resolve(null)),
};

const isErrorWithCode = jest.fn((error) => {
  return error && typeof error.code === 'string';
});

module.exports = {
  GoogleSignin,
  statusCodes,
  isErrorWithCode,
};
