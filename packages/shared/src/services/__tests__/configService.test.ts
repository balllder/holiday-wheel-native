import { configService } from '../configService';

// Use the global mock from jest.setup.js
declare const global: {
  mockAsyncStorage: {
    getItem: jest.Mock;
    setItem: jest.Mock;
    removeItem: jest.Mock;
    clear: jest.Mock;
    getAllKeys: jest.Mock;
    multiGet: jest.Mock;
    multiSet: jest.Mock;
    multiRemove: jest.Mock;
  };
};

describe('configService', () => {
  const mockGetItem = global.mockAsyncStorage.getItem;
  const mockSetItem = global.mockAsyncStorage.setItem;

  beforeEach(() => {
    // Clear all mocks before each test
    jest.clearAllMocks();
    // Clear cached URL
    configService.clearCache();
  });

  describe('getServerUrl', () => {
    it('returns cached URL if available', async () => {
      // First set a URL to cache it
      await configService.setServerUrl('http://cached.test:5000');

      // Should return cached URL without hitting storage
      const url = await configService.getServerUrl();

      expect(url).toBe('http://cached.test:5000');
      // getItem should not be called since URL is cached
      expect(mockGetItem).not.toHaveBeenCalled();
    });

    it('loads URL from AsyncStorage if not cached', async () => {
      mockGetItem.mockResolvedValueOnce('http://stored.test:5000');

      const url = await configService.getServerUrl();

      expect(url).toBe('http://stored.test:5000');
      expect(mockGetItem).toHaveBeenCalledWith('@holiday_wheel_server_url');
    });

    it('returns default URL if nothing stored', async () => {
      mockGetItem.mockResolvedValueOnce(null);

      const url = await configService.getServerUrl();

      expect(url).toBe('http://192.168.1.100:5000');
    });

    it('returns default URL if AsyncStorage throws', async () => {
      mockGetItem.mockRejectedValueOnce(new Error('Storage error'));
      const consoleSpy = jest.spyOn(console, 'error').mockImplementation();

      const url = await configService.getServerUrl();

      expect(url).toBe('http://192.168.1.100:5000');
      expect(consoleSpy).toHaveBeenCalled();
      consoleSpy.mockRestore();
    });
  });

  describe('setServerUrl', () => {
    it('saves URL to cache and AsyncStorage', async () => {
      mockSetItem.mockResolvedValueOnce(undefined);

      await configService.setServerUrl('http://new.test:5000');

      expect(mockSetItem).toHaveBeenCalledWith(
        '@holiday_wheel_server_url',
        'http://new.test:5000'
      );

      // Verify it's cached
      const url = await configService.getServerUrl();
      expect(url).toBe('http://new.test:5000');
    });

    it('handles AsyncStorage errors gracefully', async () => {
      mockSetItem.mockRejectedValueOnce(new Error('Storage error'));
      const consoleSpy = jest.spyOn(console, 'error').mockImplementation();

      // Should not throw
      await configService.setServerUrl('http://fail.test:5000');

      expect(consoleSpy).toHaveBeenCalled();

      // URL should still be cached in memory
      const url = await configService.getServerUrl();
      expect(url).toBe('http://fail.test:5000');

      consoleSpy.mockRestore();
    });
  });

  describe('getDefaultUrls', () => {
    it('returns default URL configuration', () => {
      const defaults = configService.getDefaultUrls();

      expect(defaults).toEqual({
        android_emulator: 'http://10.0.2.2:5000',
        ios_simulator: 'http://localhost:5000',
        device: 'http://192.168.1.100:5000',
      });
    });
  });

  describe('clearCache', () => {
    it('clears the cached URL', async () => {
      // Set a URL first
      await configService.setServerUrl('http://cached.test:5000');

      // Clear cache
      configService.clearCache();

      // Now getServerUrl should try AsyncStorage again
      mockGetItem.mockResolvedValueOnce('http://different.test:5000');
      const url = await configService.getServerUrl();

      expect(mockGetItem).toHaveBeenCalled();
      expect(url).toBe('http://different.test:5000');
    });
  });
});
