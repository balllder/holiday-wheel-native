// Mock for react-native-passkeys
module.exports = {
  isSupported: jest.fn(() => true),
  isAutoFillAvalilable: jest.fn(() => false),
  create: jest.fn(() => Promise.resolve(null)),
  get: jest.fn(() => Promise.resolve(null)),
};
