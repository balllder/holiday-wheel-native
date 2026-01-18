// Mock for @invertase/react-native-apple-authentication
const appleAuth = {
  performRequest: jest.fn(() =>
    Promise.resolve({
      user: 'mock-user-id',
      email: 'test@example.com',
      identityToken: 'mock-identity-token',
      fullName: {
        givenName: 'Test',
        familyName: 'User',
      },
    })
  ),
  getCredentialStateForUser: jest.fn(() => Promise.resolve(1)), // State.AUTHORIZED
  Operation: {
    LOGIN: 1,
    REFRESH: 2,
    LOGOUT: 3,
  },
  Scope: {
    EMAIL: 0,
    FULL_NAME: 1,
  },
  State: {
    REVOKED: 0,
    AUTHORIZED: 1,
    NOT_FOUND: 2,
    TRANSFERRED: 3,
  },
  Error: {
    CANCELED: '1001',
    FAILED: '1002',
    INVALID_RESPONSE: '1003',
    NOT_HANDLED: '1004',
    UNKNOWN: '1005',
  },
};

module.exports = appleAuth;
module.exports.default = appleAuth;
