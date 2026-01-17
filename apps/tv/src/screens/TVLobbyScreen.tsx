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
import { useAuthStore, authService, configService } from '@holiday-wheel/shared';
import type { RoomInfo } from '@holiday-wheel/shared';
import type { TVStackParamList } from '../navigation/TVNavigator';
import { RoomQRCode } from '../components/RoomQRCode';

type TVLobbyScreenProps = {
  navigation: NativeStackNavigationProp<TVStackParamList, 'TVLobby'>;
};

const DEFAULT_API_URL = 'http://192.168.1.100:5000';

export function TVLobbyScreen({ navigation }: TVLobbyScreenProps): React.JSX.Element {
  const [rooms, setRooms] = useState<RoomInfo[]>([]);
  const [customRoom, setCustomRoom] = useState('main');
  const [focusedItem, setFocusedItem] = useState<string>('input');
  const [serverUrl, setServerUrl] = useState<string>(DEFAULT_API_URL);
  const [showQRCode, setShowQRCode] = useState<boolean>(false);

  const user = useAuthStore((state) => state.user);
  const token = useAuthStore((state) => state.token);
  const clearAuth = useAuthStore((state) => state.clearAuth);

  // Load server URL from config
  useEffect(() => {
    const loadServerUrl = async () => {
      const url = await configService.getServerUrl();
      setServerUrl(url);
    };
    loadServerUrl();
  }, []);

  const loadRooms = useCallback(async () => {
    if (!token) return;
    authService.setBaseUrl(serverUrl);
    const result = await authService.getRooms(token);
    setRooms(result.rooms);
  }, [token, serverUrl]);

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
        <View style={styles.mainContent}>
          {/* Left side - Room controls */}
          <View style={styles.leftPanel}>
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

          {/* Right side - Connection info */}
          <View style={styles.rightPanel}>
            <Text style={styles.sectionTitle}>Phone Connection</Text>
            <View style={styles.serverInfo}>
              <Text style={styles.serverLabel}>Server:</Text>
              <Text style={styles.serverUrl}>{serverUrl}</Text>
            </View>
            <View style={styles.serverInfo}>
              <Text style={styles.serverLabel}>Room:</Text>
              <Text style={styles.serverUrl}>{customRoom || 'main'}</Text>
            </View>

            <TouchableOpacity
              style={[
                styles.qrButton,
                focusedItem === 'qr' && styles.buttonFocused,
              ]}
              onPress={() => setShowQRCode(!showQRCode)}
              onFocus={() => setFocusedItem('qr')}
            >
              <Text style={styles.qrButtonText}>
                {showQRCode ? 'Hide QR Code' : 'Show QR Code'}
              </Text>
            </TouchableOpacity>

            {showQRCode && (
              <View style={styles.qrContainer}>
                <RoomQRCode
                  room={customRoom || 'main'}
                  serverUrl={serverUrl}
                  size={180}
                />
                <Text style={styles.qrHint}>
                  Scan with phone to join
                </Text>
              </View>
            )}

            <Text style={styles.manualHint}>
              Or enter server URL manually on phone
            </Text>
          </View>
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
  mainContent: {
    flex: 1,
    flexDirection: 'row',
    gap: 40,
  },
  leftPanel: {
    flex: 1,
  },
  rightPanel: {
    width: 320,
    backgroundColor: '#1a0a3e',
    borderRadius: 16,
    padding: 24,
    borderWidth: 2,
    borderColor: '#333',
  },
  serverInfo: {
    flexDirection: 'row',
    marginBottom: 12,
    flexWrap: 'wrap',
  },
  serverLabel: {
    color: '#888',
    fontSize: 18,
    marginRight: 8,
  },
  serverUrl: {
    color: '#d4af37',
    fontSize: 18,
    fontWeight: 'bold',
    flex: 1,
  },
  qrButton: {
    backgroundColor: '#2a1a4e',
    paddingVertical: 16,
    paddingHorizontal: 24,
    borderRadius: 8,
    marginTop: 16,
    alignItems: 'center',
    borderWidth: 2,
    borderColor: '#333',
  },
  qrButtonText: {
    color: '#d4af37',
    fontSize: 18,
    fontWeight: 'bold',
  },
  qrContainer: {
    alignItems: 'center',
    marginTop: 20,
    padding: 16,
    backgroundColor: '#fff',
    borderRadius: 12,
  },
  qrHint: {
    color: '#333',
    fontSize: 14,
    marginTop: 8,
    textAlign: 'center',
  },
  manualHint: {
    color: '#666',
    fontSize: 14,
    marginTop: 16,
    textAlign: 'center',
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
