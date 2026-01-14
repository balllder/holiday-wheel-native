/**
 * Holiday Wheel of Fortune - React Native App
 */

import React, { useEffect, useState } from 'react';
import { StatusBar, ActivityIndicator, View, StyleSheet } from 'react-native';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { useAuthStore } from '@holiday-wheel/shared';
import { AppNavigator } from './src/navigation/AppNavigator';

function App(): React.JSX.Element {
  const [isLoading, setIsLoading] = useState(true);
  const setAuth = useAuthStore((state) => state.setAuth);

  useEffect(() => {
    // Check for stored auth on app start
    const checkAuth = async () => {
      try {
        const token = await AsyncStorage.getItem('@auth_token');
        const userStr = await AsyncStorage.getItem('@auth_user');

        if (token && userStr) {
          const user = JSON.parse(userStr);
          setAuth(user, token);
        }
      } catch (error) {
        console.error('Error checking auth:', error);
      } finally {
        setIsLoading(false);
      }
    };

    checkAuth();
  }, [setAuth]);

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
      <AppNavigator />
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
