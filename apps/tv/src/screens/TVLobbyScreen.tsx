import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  FlatList,
  TVFocusGuideView,
} from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { useAuthStore, authService } from '@holiday-wheel/shared';
import type { RoomInfo } from '@holiday-wheel/shared';
import type { TVStackParamList } from '../navigation/TVNavigator';

type TVLobbyScreenProps = {
  navigation: NativeStackNavigationProp<TVStackParamList, 'TVLobby'>;
};

const API_URL = 'http://192.168.1.100:5000';

export function TVLobbyScreen({ navigation }: TVLobbyScreenProps): React.JSX.Element {
  const [rooms, setRooms] = useState<RoomInfo[]>([]);
  const [customRoom, setCustomRoom] = useState('main');
  const [focusedItem, setFocusedItem] = useState<string>('input');

  const user = useAuthStore((state) => state.user);
  const token = useAuthStore((state) => state.token);
  const clearAuth = useAuthStore((state) => state.clearAuth);

  const loadRooms = useCallback(async () => {
    if (!token) return;
    authService.setBaseUrl(API_URL);
    const result = await authService.getRooms(token);
    setRooms(result.rooms);
  }, [token]);

  useEffect(() => {
    loadRooms();
    const interval = setInterval(loadRooms, 10000); // Refresh every 10s
    return () => clearInterval(interval);
  }, [loadRooms]);

  const handleLogout = async () => {
    await AsyncStorage.multiRemove(['@auth_token', '@auth_user']);
    clearAuth();
  };

  const joinRoom = (room: string) => {
    navigation.navigate('TVGame', { room: room.trim() || 'main' });
  };

  const renderRoomItem = ({ item, index }: { item: RoomInfo; index: number }) => (
    <TouchableOpacity
      style={[
        styles.roomCard,
        focusedItem === `room-${index}` && styles.roomCardFocused,
      ]}
      onPress={() => joinRoom(item.name)}
      onFocus={() => setFocusedItem(`room-${index}`)}
      activeOpacity={0.8}
    >
      <Text style={styles.roomName}>{item.name}</Text>
      <Text style={styles.roomPlayers}>{item.player_count} players</Text>
    </TouchableOpacity>
  );

  return (
    <View style={styles.container}>
      {/* Header */}
      <View style={styles.header}>
        <View>
          <Text style={styles.title}>🎡 Holiday Wheel</Text>
          <Text style={styles.welcome}>Welcome, {user?.display_name}!</Text>
        </View>
        <TouchableOpacity
          style={[
            styles.logoutButton,
            focusedItem === 'logout' && styles.buttonFocused,
          ]}
          onPress={handleLogout}
          onFocus={() => setFocusedItem('logout')}
        >
          <Text style={styles.logoutText}>Logout</Text>
        </TouchableOpacity>
      </View>

      <View style={styles.content}>
        {/* Join Room Section */}
        <TVFocusGuideView style={styles.joinSection} autoFocus>
          <Text style={styles.sectionTitle}>Join a Room</Text>
          <View style={styles.joinRow}>
            <TextInput
              style={[
                styles.input,
                focusedItem === 'input' && styles.inputFocused,
              ]}
              placeholder="Room name"
              placeholderTextColor="#666"
              value={customRoom}
              onChangeText={setCustomRoom}
              onFocus={() => setFocusedItem('input')}
              onSubmitEditing={() => joinRoom(customRoom)}
              hasTVPreferredFocus={true}
            />
            <TouchableOpacity
              style={[
                styles.joinButton,
                focusedItem === 'join' && styles.buttonFocused,
              ]}
              onPress={() => joinRoom(customRoom)}
              onFocus={() => setFocusedItem('join')}
            >
              <Text style={styles.joinButtonText}>JOIN</Text>
            </TouchableOpacity>
          </View>
        </TVFocusGuideView>

        {/* Active Rooms */}
        <View style={styles.roomsSection}>
          <Text style={styles.sectionTitle}>Active Rooms</Text>
          {rooms.length > 0 ? (
            <FlatList
              data={rooms}
              renderItem={renderRoomItem}
              keyExtractor={(item) => item.name}
              horizontal
              showsHorizontalScrollIndicator={false}
              contentContainerStyle={styles.roomsList}
            />
          ) : (
            <Text style={styles.emptyText}>No active rooms. Join one above!</Text>
          )}
        </View>
      </View>

      {/* Instructions */}
      <View style={styles.footer}>
        <Text style={styles.footerText}>
          ← → Navigate rooms • ↑ ↓ Move between sections • Select to join
        </Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0d0628',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 40,
    borderBottomWidth: 1,
    borderBottomColor: '#333',
  },
  title: {
    fontSize: 42,
    fontWeight: 'bold',
    color: '#d4af37',
  },
  welcome: {
    fontSize: 24,
    color: '#888',
    marginTop: 8,
  },
  logoutButton: {
    backgroundColor: '#1a0a3e',
    paddingHorizontal: 32,
    paddingVertical: 16,
    borderRadius: 8,
    borderWidth: 2,
    borderColor: '#333',
  },
  logoutText: {
    color: '#d4af37',
    fontSize: 20,
    fontWeight: 'bold',
  },
  content: {
    flex: 1,
    padding: 40,
  },
  joinSection: {
    marginBottom: 48,
  },
  sectionTitle: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#fff',
    marginBottom: 20,
  },
  joinRow: {
    flexDirection: 'row',
    gap: 20,
  },
  input: {
    flex: 1,
    backgroundColor: '#1a0a3e',
    borderRadius: 12,
    padding: 24,
    color: '#ffffff',
    fontSize: 28,
    borderWidth: 3,
    borderColor: '#333',
  },
  inputFocused: {
    borderColor: '#d4af37',
    backgroundColor: '#2a1a4e',
  },
  joinButton: {
    backgroundColor: '#d4af37',
    paddingHorizontal: 48,
    borderRadius: 12,
    justifyContent: 'center',
    borderWidth: 3,
    borderColor: 'transparent',
  },
  joinButtonText: {
    color: '#1a0a3e',
    fontSize: 28,
    fontWeight: 'bold',
  },
  buttonFocused: {
    borderColor: '#fff',
    transform: [{ scale: 1.02 }],
  },
  roomsSection: {
    flex: 1,
  },
  roomsList: {
    gap: 20,
  },
  roomCard: {
    backgroundColor: '#1a0a3e',
    borderRadius: 16,
    padding: 32,
    minWidth: 280,
    borderWidth: 3,
    borderColor: '#333',
  },
  roomCardFocused: {
    borderColor: '#d4af37',
    backgroundColor: '#2a1a4e',
    transform: [{ scale: 1.05 }],
  },
  roomName: {
    color: '#fff',
    fontSize: 32,
    fontWeight: 'bold',
  },
  roomPlayers: {
    color: '#888',
    fontSize: 22,
    marginTop: 8,
  },
  emptyText: {
    color: '#666',
    fontSize: 24,
    textAlign: 'center',
    marginTop: 40,
  },
  footer: {
    padding: 24,
    alignItems: 'center',
    borderTopWidth: 1,
    borderTopColor: '#333',
  },
  footerText: {
    color: '#555',
    fontSize: 18,
  },
});
