import React from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { useAuthStore } from '@holiday-wheel/shared';

import { TVLoginScreen } from '../screens/TVLoginScreen';
import { TVLobbyScreen } from '../screens/TVLobbyScreen';
import { TVGameScreen } from '../screens/TVGameScreen';

export type TVStackParamList = {
  TVLogin: undefined;
  TVLobby: undefined;
  TVGame: { room: string };
};

const Stack = createNativeStackNavigator<TVStackParamList>();

export function TVNavigator(): React.JSX.Element {
  const user = useAuthStore((state) => state.user);

  return (
    <NavigationContainer>
      <Stack.Navigator
        screenOptions={{
          headerShown: false, // Full screen for TV
          contentStyle: {
            backgroundColor: '#0d0628',
          },
          animation: 'fade',
        }}
      >
        {!user ? (
          <Stack.Screen name="TVLogin" component={TVLoginScreen} />
        ) : (
          <>
            <Stack.Screen name="TVLobby" component={TVLobbyScreen} />
            <Stack.Screen name="TVGame" component={TVGameScreen} />
          </>
        )}
      </Stack.Navigator>
    </NavigationContainer>
  );
}
