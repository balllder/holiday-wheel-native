import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  FlatList,
  RefreshControl,
  Platform,
} from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { useAuthStore, authService, configService } from '@holiday-wheel/shared';
import type { RoomInfo } from '@holiday-wheel/shared';
import type { RootStackParamList } from '../navigation/AppNavigator';

type LobbyScreenProps = {
  navigation: NativeStackNavigationProp<RootStackParamList, 'Lobby'>;
};

// Default URL based on platform
const getDefaultUrl = () => {
  if (Platform.OS === 'android') {
    return 'http://10.0.2.2:5000';
  }
  return 'http://localhost:5000';
};

export function LobbyScreen({ navigation }: LobbyScreenProps): React.JSX.Element {
  const [rooms, setRooms] = useState<RoomInfo[]>([]);
  const [customRoom, setCustomRoom] = useState('');
  const [refreshing, setRefreshing] = useState(false);
  const [serverUrl, setServerUrl] = useState(getDefaultUrl());
  const [showServerConfig, setShowServerConfig] = useState(false);
  const [serverSaved, setServerSaved] = useState(false);

  const user = useAuthStore((state) => state.user);
  const token = useAuthStore((state) => state.token);
  const clearAuth = useAuthStore((state) => state.clearAuth);

  // Load saved server URL on mount
  useEffect(() => {
    const loadServerUrl = async () => {
      const savedUrl = await configService.getServerUrl();
      if (savedUrl) {
        setServerUrl(savedUrl);
      }
    };
    loadServerUrl();
  }, []);

  const loadRooms = useCallback(async () => {
    if (!token) return;

    try {
      authService.setBaseUrl(serverUrl);
      const result = await authService.getRooms(token);
      setRooms(result.rooms);
    } catch (error) {
      console.error('Failed to load rooms:', error);
      setRooms([]);
    }
  }, [token, serverUrl]);

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

  const handleSaveServer = async () => {
    let url = serverUrl.trim();
    // Ensure URL has protocol
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      url = 'http://' + url;
      setServerUrl(url);
    }
    // Remove trailing slash
    if (url.endsWith('/')) {
      url = url.slice(0, -1);
      setServerUrl(url);
    }

    await configService.setServerUrl(url);
    authService.setBaseUrl(url);
    setServerSaved(true);
    setTimeout(() => setServerSaved(false), 2000);
    loadRooms();
  };

  const joinRoom = (room: string, asController: boolean = false) => {
    const trimmedRoom = room.trim() || 'main';
    // Ensure auth service has the current server URL
    authService.setBaseUrl(serverUrl);

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

      {/* Server configuration */}
      <TouchableOpacity
        style={styles.serverToggle}
        onPress={() => setShowServerConfig(!showServerConfig)}
      >
        <Text style={styles.serverToggleText}>
          ⚙️ Server: {serverUrl.replace(/^https?:\/\//, '')}
        </Text>
        <Text style={styles.serverToggleArrow}>
          {showServerConfig ? '▲' : '▼'}
        </Text>
      </TouchableOpacity>

      {showServerConfig && (
        <View style={styles.serverConfig}>
          <Text style={styles.serverLabel}>Server URL</Text>
          <View style={styles.serverInputRow}>
            <TextInput
              style={styles.serverInput}
              placeholder="http://192.168.1.100:5000"
              placeholderTextColor="#666"
              value={serverUrl}
              onChangeText={setServerUrl}
              autoCapitalize="none"
              autoCorrect={false}
              keyboardType="url"
            />
            <TouchableOpacity
              style={[styles.saveButton, serverSaved && styles.saveButtonSaved]}
              onPress={handleSaveServer}
            >
              <Text style={styles.saveButtonText}>
                {serverSaved ? '✓' : 'Save'}
              </Text>
            </TouchableOpacity>
          </View>
          <Text style={styles.serverHint}>
            Enter the IP address of the computer running the backend server.
            {Platform.OS === 'android' && ' For Android emulator, use 10.0.2.2'}
          </Text>
        </View>
      )}

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

      {/* Scan QR Code button */}
      <TouchableOpacity
        style={styles.qrButton}
        onPress={() => navigation.navigate('QRScan')}
      >
        <Text style={styles.qrButtonText}>📷 Scan QR Code to Join TV Game</Text>
      </TouchableOpacity>

      {/* Mode explanation */}
      <View style={styles.modeHelp}>
        <Text style={styles.modeHelpText}>
          <Text style={styles.modeHelpBold}>Play:</Text> Full game on this device{'\n'}
          <Text style={styles.modeHelpBold}>📱:</Text> Use as controller (TV displays game){'\n'}
          <Text style={styles.modeHelpBold}>📷:</Text> Scan QR from Apple TV to join
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
  serverToggle: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    backgroundColor: '#150833',
    borderBottomWidth: 1,
    borderBottomColor: '#333',
  },
  serverToggleText: {
    color: '#888',
    fontSize: 14,
  },
  serverToggleArrow: {
    color: '#888',
    fontSize: 12,
  },
  serverConfig: {
    backgroundColor: '#1a0a3e',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#333',
  },
  serverLabel: {
    color: '#d4af37',
    fontSize: 14,
    fontWeight: 'bold',
    marginBottom: 8,
  },
  serverInputRow: {
    flexDirection: 'row',
    gap: 8,
  },
  serverInput: {
    flex: 1,
    backgroundColor: '#0d0628',
    borderRadius: 8,
    padding: 12,
    color: '#ffffff',
    fontSize: 16,
    borderWidth: 1,
    borderColor: '#333',
  },
  saveButton: {
    backgroundColor: '#4caf50',
    borderRadius: 8,
    paddingHorizontal: 20,
    justifyContent: 'center',
  },
  saveButtonSaved: {
    backgroundColor: '#2e7d32',
  },
  saveButtonText: {
    color: '#fff',
    fontWeight: 'bold',
    fontSize: 16,
  },
  serverHint: {
    color: '#666',
    fontSize: 12,
    marginTop: 8,
    lineHeight: 18,
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
  qrButton: {
    backgroundColor: '#4caf50',
    marginHorizontal: 16,
    borderRadius: 12,
    padding: 16,
    alignItems: 'center',
  },
  qrButtonText: {
    color: '#fff',
    fontSize: 18,
    fontWeight: 'bold',
  },
  modeHelp: {
    paddingHorizontal: 16,
    paddingVertical: 12,
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
