/**
 * Holiday Wheel of Fortune - React Native App
 */

import React, { useEffect, useState, useRef, useCallback } from 'react';
import {
  StatusBar,
  ActivityIndicator,
  View,
  StyleSheet,
  Linking,
} from 'react-native';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { NavigationContainerRef } from '@react-navigation/native';
import { useAuthStore, configService } from '@holiday-wheel/shared';
import { AppNavigator, RootStackParamList } from './src/navigation/AppNavigator';

// Deep link configuration
const linking = {
  prefixes: ['holidaywheel://'],
  config: {
    screens: {
      Controller: {
        path: 'join',
        parse: {
          room: (room: string) => room,
        },
      },
    },
  },
};

// Parse deep link URL and extract params
function parseDeepLink(url: string): { room?: string; server?: string } | null {
  try {
    // Handle holidaywheel://join?room=ROOM&server=URL
    if (url.startsWith('holidaywheel://join')) {
      const queryString = url.split('?')[1];
      if (!queryString) return null;

      const params: { room?: string; server?: string } = {};
      const pairs = queryString.split('&');

      for (const pair of pairs) {
        const [key, value] = pair.split('=');
        if (key === 'room') {
          params.room = decodeURIComponent(value);
        } else if (key === 'server') {
          params.server = decodeURIComponent(value);
        }
      }

      return params;
    }
    return null;
  } catch {
    console.error('Error parsing deep link:', url);
    return null;
  }
}

function App(): React.JSX.Element {
  const [isLoading, setIsLoading] = useState(true);
  const [pendingDeepLink, setPendingDeepLink] = useState<{
    room: string;
    server?: string;
  } | null>(null);
  const navigationRef = useRef<NavigationContainerRef<RootStackParamList>>(null);
  const setAuth = useAuthStore((state) => state.setAuth);
  const user = useAuthStore((state) => state.user);

  // Handle incoming deep links
  const handleDeepLink = useCallback(async (url: string | null) => {
    if (!url) return;

    const params = parseDeepLink(url);
    if (params?.room) {
      // Save server URL if provided
      if (params.server) {
        await configService.setServerUrl(params.server);
      }

      // If user is logged in, navigate immediately
      // Otherwise, store the pending deep link
      if (user && navigationRef.current?.isReady()) {
        navigationRef.current.navigate('Controller', { room: params.room });
      } else {
        setPendingDeepLink({ room: params.room, server: params.server });
      }
    }
  }, [user]);

  useEffect(() => {
    // Check for stored auth on app start
    const checkAuth = async () => {
      try {
        const token = await AsyncStorage.getItem('@auth_token');
        const userStr = await AsyncStorage.getItem('@auth_user');

        if (token && userStr) {
          const userObj = JSON.parse(userStr);
          setAuth(userObj, token);
        }
      } catch (error) {
        console.error('Error checking auth:', error);
      } finally {
        setIsLoading(false);
      }
    };

    checkAuth();
  }, [setAuth]);

  useEffect(() => {
    // Check for initial URL (app opened via deep link)
    Linking.getInitialURL().then(handleDeepLink);

    // Listen for deep links while app is running
    const subscription = Linking.addEventListener('url', (event) => {
      handleDeepLink(event.url);
    });

    return () => subscription.remove();
  }, [handleDeepLink]);

  // Handle pending deep link after user logs in
  useEffect(() => {
    if (user && pendingDeepLink && navigationRef.current?.isReady()) {
      navigationRef.current.navigate('Controller', { room: pendingDeepLink.room });
      setPendingDeepLink(null);
    }
  }, [user, pendingDeepLink]);

  if (isLoading) {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color="#d4af37" />
      </View>
    );
  }

  return (
    <SafeAreaProvider>
      <StatusBar barStyle="light-content" backgroundColor="#0d0628" />
      <AppNavigator navigationRef={navigationRef} linking={linking} />
    </SafeAreaProvider>
  );
}

const styles = StyleSheet.create({
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#0d0628',
  },
});

export default App;
