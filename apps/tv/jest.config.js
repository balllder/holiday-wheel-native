module.exports = {
  preset: 'react-native',
  setupFiles: ['./jest.setup.js'],
  transformIgnorePatterns: [
    'node_modules/(?!(react-native|@react-native|@react-navigation|react-native-.*|@react-native-.*|socket.io-client|engine.io-client|@socket.io|zustand|@holiday-wheel/shared|expo-modules-core|@invertase)/)',
  ],
  moduleNameMapper: {
    '^@react-native-google-signin/google-signin$': '<rootDir>/__mocks__/@react-native-google-signin/google-signin.js',
    '^@invertase/react-native-apple-authentication$': '<rootDir>/__mocks__/@invertase/react-native-apple-authentication.js',
  },
};
