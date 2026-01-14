import React from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { useAuthStore } from '@holiday-wheel/shared';

import { LoginScreen } from '../screens/LoginScreen';
import { RegisterScreen } from '../screens/RegisterScreen';
import { LobbyScreen } from '../screens/LobbyScreen';
import { GameScreen } from '../screens/GameScreen';
import { ControllerScreen } from '../screens/ControllerScreen';

export type RootStackParamList = {
  Login: undefined;
  Register: undefined;
  Lobby: undefined;
  Game: { room: string };
  Controller: { room: string };
};

const Stack = createNativeStackNavigator<RootStackParamList>();

export function AppNavigator(): React.JSX.Element {
  const user = useAuthStore((state) => state.user);

  return (
    <NavigationContainer>
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
          </>
        )}
      </Stack.Navigator>
    </NavigationContainer>
  );
}
