import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  FlatList,
  RefreshControl,
} from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { useAuthStore, authService } from '@holiday-wheel/shared';
import type { RoomInfo } from '@holiday-wheel/shared';
import type { RootStackParamList } from '../navigation/AppNavigator';

type LobbyScreenProps = {
  navigation: NativeStackNavigationProp<RootStackParamList, 'Lobby'>;
};

const API_URL = 'http://10.0.2.2:5000';

export function LobbyScreen({ navigation }: LobbyScreenProps): React.JSX.Element {
  const [rooms, setRooms] = useState<RoomInfo[]>([]);
  const [customRoom, setCustomRoom] = useState('');
  const [refreshing, setRefreshing] = useState(false);

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
  }, [loadRooms]);

  const onRefresh = async () => {
    setRefreshing(true);
    await loadRooms();
    setRefreshing(false);
  };

  const handleLogout = async () => {
    await AsyncStorage.multiRemove(['@auth_token', '@auth_user']);
    clearAuth();
  };

  const joinRoom = (room: string, asController: boolean = false) => {
    const trimmedRoom = room.trim() || 'main';
    if (asController) {
      navigation.navigate('Controller', { room: trimmedRoom });
    } else {
      navigation.navigate('Game', { room: trimmedRoom });
    }
  };

  const renderRoomItem = ({ item }: { item: RoomInfo }) => (
    <View style={styles.roomCard}>
      <View style={styles.roomInfo}>
        <Text style={styles.roomName}>{item.name}</Text>
        <Text style={styles.roomPlayers}>{item.player_count} players</Text>
      </View>
      <View style={styles.roomActions}>
        <TouchableOpacity
          style={styles.joinButton}
          onPress={() => joinRoom(item.name)}
        >
          <Text style={styles.joinButtonText}>Play</Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={styles.controllerButton}
          onPress={() => joinRoom(item.name, true)}
        >
          <Text style={styles.controllerButtonText}>📱</Text>
        </TouchableOpacity>
      </View>
    </View>
  );

  return (
    <View style={styles.container}>
      {/* User info bar */}
      <View style={styles.userBar}>
        <Text style={styles.userName}>Welcome, {user?.display_name}!</Text>
        <TouchableOpacity onPress={handleLogout}>
          <Text style={styles.logoutText}>Logout</Text>
        </TouchableOpacity>
      </View>

      {/* Custom room input */}
      <View style={styles.customRoomContainer}>
        <TextInput
          style={styles.input}
          placeholder="Enter room name..."
          placeholderTextColor="#888"
          value={customRoom}
          onChangeText={setCustomRoom}
          autoCapitalize="none"
        />
        <TouchableOpacity
          style={styles.playButton}
          onPress={() => joinRoom(customRoom)}
        >
          <Text style={styles.playButtonText}>Play</Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={styles.controllerBtn}
          onPress={() => joinRoom(customRoom, true)}
        >
          <Text style={styles.controllerBtnText}>📱</Text>
        </TouchableOpacity>
      </View>

      {/* Mode explanation */}
      <View style={styles.modeHelp}>
        <Text style={styles.modeHelpText}>
          <Text style={styles.modeHelpBold}>Play:</Text> Full game on this device{'\n'}
          <Text style={styles.modeHelpBold}>📱:</Text> Use as controller (TV displays game)
        </Text>
      </View>

      {/* Room list */}
      <Text style={styles.sectionTitle}>Active Rooms</Text>
      <FlatList
        data={rooms}
        renderItem={renderRoomItem}
        keyExtractor={(item) => item.name}
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={onRefresh}
            tintColor="#d4af37"
          />
        }
        ListEmptyComponent={
          <Text style={styles.emptyText}>No active rooms. Create one above!</Text>
        }
        contentContainerStyle={rooms.length === 0 && styles.emptyList}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0d0628',
  },
  userBar: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 16,
    backgroundColor: '#1a0a3e',
    borderBottomWidth: 1,
    borderBottomColor: '#333',
  },
  userName: {
    color: '#ffffff',
    fontSize: 16,
  },
  logoutText: {
    color: '#d4af37',
    fontSize: 14,
  },
  customRoomContainer: {
    flexDirection: 'row',
    padding: 16,
    gap: 8,
  },
  input: {
    flex: 1,
    backgroundColor: '#1a0a3e',
    borderRadius: 8,
    padding: 12,
    color: '#ffffff',
    fontSize: 16,
    borderWidth: 1,
    borderColor: '#333',
  },
  playButton: {
    backgroundColor: '#d4af37',
    borderRadius: 8,
    paddingHorizontal: 20,
    justifyContent: 'center',
  },
  playButtonText: {
    color: '#1a0a3e',
    fontWeight: 'bold',
    fontSize: 16,
  },
  controllerBtn: {
    backgroundColor: '#1a0a3e',
    borderRadius: 8,
    paddingHorizontal: 16,
    justifyContent: 'center',
    borderWidth: 1,
    borderColor: '#d4af37',
  },
  controllerBtnText: {
    fontSize: 20,
  },
  modeHelp: {
    paddingHorizontal: 16,
    paddingBottom: 16,
  },
  modeHelpText: {
    color: '#888',
    fontSize: 12,
    lineHeight: 18,
  },
  modeHelpBold: {
    color: '#aaa',
    fontWeight: 'bold',
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#d4af37',
    paddingHorizontal: 16,
    paddingVertical: 8,
  },
  roomCard: {
    backgroundColor: '#1a0a3e',
    marginHorizontal: 16,
    marginVertical: 4,
    borderRadius: 8,
    padding: 16,
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  roomInfo: {
    flex: 1,
  },
  roomName: {
    color: '#ffffff',
    fontSize: 18,
    fontWeight: 'bold',
  },
  roomPlayers: {
    color: '#888',
    fontSize: 14,
    marginTop: 4,
  },
  roomActions: {
    flexDirection: 'row',
    gap: 8,
  },
  joinButton: {
    backgroundColor: '#d4af37',
    borderRadius: 6,
    paddingHorizontal: 16,
    paddingVertical: 8,
  },
  joinButtonText: {
    color: '#1a0a3e',
    fontWeight: 'bold',
  },
  controllerButton: {
    backgroundColor: '#1a0a3e',
    borderRadius: 6,
    paddingHorizontal: 12,
    paddingVertical: 8,
    borderWidth: 1,
    borderColor: '#d4af37',
  },
  controllerButtonText: {
    fontSize: 16,
  },
  emptyText: {
    color: '#888',
    textAlign: 'center',
    marginTop: 32,
    fontSize: 16,
  },
  emptyList: {
    flex: 1,
  },
});
