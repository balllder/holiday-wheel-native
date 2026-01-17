import AsyncStorage from '@react-native-async-storage/async-storage';

const STORAGE_KEY = '@holiday_wheel_server_url';

// Default server URLs by platform
const DEFAULT_URLS = {
  android_emulator: 'http://10.0.2.2:5000',
  ios_simulator: 'http://localhost:5000',
  device: 'http://192.168.1.100:5000', // Change this to your actual server IP
};

class ConfigService {
  private serverUrl: string | null = null;

  async getServerUrl(): Promise<string> {
    if (this.serverUrl) {
      return this.serverUrl;
    }

    try {
      const stored = await AsyncStorage.getItem(STORAGE_KEY);
      if (stored) {
        this.serverUrl = stored;
        return stored;
      }
    } catch (error) {
      console.error('Failed to load server URL from storage:', error);
    }

    // Return a default based on common development setup
    return DEFAULT_URLS.device;
  }

  async setServerUrl(url: string): Promise<void> {
    this.serverUrl = url;
    try {
      await AsyncStorage.setItem(STORAGE_KEY, url);
    } catch (error) {
      console.error('Failed to save server URL to storage:', error);
    }
  }

  getDefaultUrls() {
    return DEFAULT_URLS;
  }

  clearCache(): void {
    this.serverUrl = null;
  }
}

export const configService = new ConfigService();
