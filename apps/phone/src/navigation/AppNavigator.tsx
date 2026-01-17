import React from 'react';
import {
  NavigationContainer,
  NavigationContainerRef,
  LinkingOptions,
} from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { useAuthStore } from '@holiday-wheel/shared';

import { LoginScreen } from '../screens/LoginScreen';
import { RegisterScreen } from '../screens/RegisterScreen';
import { LobbyScreen } from '../screens/LobbyScreen';
import { GameScreen } from '../screens/GameScreen';
import { ControllerScreen } from '../screens/ControllerScreen';
import { QRScanScreen } from '../screens/QRScanScreen';

export type RootStackParamList = {
  Login: undefined;
  Register: undefined;
  Lobby: undefined;
  Game: { room: string };
  Controller: { room: string };
  QRScan: undefined;
};

const Stack = createNativeStackNavigator<RootStackParamList>();

interface AppNavigatorProps {
  navigationRef?: React.RefObject<NavigationContainerRef<RootStackParamList> | null>;
  linking?: LinkingOptions<RootStackParamList>;
}

export function AppNavigator({
  navigationRef,
  linking,
}: AppNavigatorProps): React.JSX.Element {
  const user = useAuthStore((state) => state.user);

  return (
    <NavigationContainer ref={navigationRef} linking={linking}>
      <Stack.Navigator
        screenOptions={{
          headerStyle: {
            backgroundColor: '#0d0628',
          },
          headerTintColor: '#d4af37',
          headerTitleStyle: {
            fontWeight: 'bold',
          },
          contentStyle: {
            backgroundColor: '#0d0628',
          },
        }}
      >
        {!user ? (
          // Auth screens
          <>
            <Stack.Screen
              name="Login"
              component={LoginScreen}
              options={{ title: 'Holiday Wheel' }}
            />
            <Stack.Screen
              name="Register"
              component={RegisterScreen}
              options={{ title: 'Create Account' }}
            />
          </>
        ) : (
          // Main app screens
          <>
            <Stack.Screen
              name="Lobby"
              component={LobbyScreen}
              options={{ title: 'Game Rooms' }}
            />
            <Stack.Screen
              name="Game"
              component={GameScreen}
              options={({ route }) => ({ title: `Room: ${route.params.room}` })}
            />
            <Stack.Screen
              name="Controller"
              component={ControllerScreen}
              options={({ route }) => ({ title: `Controller: ${route.params.room}` })}
            />
            <Stack.Screen
              name="QRScan"
              component={QRScanScreen}
              options={{
                title: 'Scan QR Code',
                headerShown: false,
                presentation: 'fullScreenModal',
              }}
            />
          </>
        )}
      </Stack.Navigator>
    </NavigationContainer>
  );
}
