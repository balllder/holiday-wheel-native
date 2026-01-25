import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  FlatList,
  TVFocusGuideView,
  Animated,
  useTVEventHandler,
} from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { useAuthStore, authService, configService } from '@holiday-wheel/shared';
import type { RoomInfo } from '@holiday-wheel/shared';
import type { TVStackParamList } from '../navigation/TVNavigator';
import { RoomQRCode } from '../components/RoomQRCode';

// Custom focusable button for lobby with animation
interface FocusableButtonProps {
  onPress: () => void;
  style: object | object[];
  focusedStyle?: object;
  children: React.ReactNode;
  hasTVPreferredFocus?: boolean;
  onFocusChange?: (focused: boolean) => void;
  testID?: string;
}

function FocusableButton({
  onPress,
  style,
  focusedStyle,
  children,
  hasTVPreferredFocus,
  onFocusChange,
  testID,
}: FocusableButtonProps): React.JSX.Element {
  const [isFocused, setIsFocused] = useState(false);
  const scaleAnim = useRef(new Animated.Value(1)).current;

  const handleFocus = useCallback(() => {
    setIsFocused(true);
    onFocusChange?.(true);
    Animated.spring(scaleAnim, {
      toValue: 1.03,
      useNativeDriver: true,
      tension: 100,
      friction: 8,
    }).start();
  }, [scaleAnim, onFocusChange]);

  const handleBlur = useCallback(() => {
    setIsFocused(false);
    onFocusChange?.(false);
    Animated.spring(scaleAnim, {
      toValue: 1,
      useNativeDriver: true,
      tension: 100,
      friction: 8,
    }).start();
  }, [scaleAnim, onFocusChange]);

  return (
    <Animated.View style={{ transform: [{ scale: scaleAnim }] }}>
      <TouchableOpacity
        style={[style, isFocused && (focusedStyle || styles.buttonFocused)]}
        onPress={onPress}
        onFocus={handleFocus}
        onBlur={handleBlur}
        hasTVPreferredFocus={hasTVPreferredFocus}
        testID={testID}
        activeOpacity={0.8}
      >
        {children}
      </TouchableOpacity>
    </Animated.View>
  );
}

type TVLobbyScreenProps = {
  navigation: NativeStackNavigationProp<TVStackParamList, 'TVLobby'>;
};

const DEFAULT_API_URL = 'http://192.168.1.100:5000';

// Focus regions for navigation
type FocusRegion = 'header' | 'serverConfig' | 'joinSection' | 'rooms' | 'rightPanel';

export function TVLobbyScreen({ navigation }: TVLobbyScreenProps): React.JSX.Element {
  const [rooms, setRooms] = useState<RoomInfo[]>([]);
  const [customRoom, setCustomRoom] = useState('main');
  const [focusedItem, setFocusedItem] = useState<string>('input');
  const [focusRegion, setFocusRegion] = useState<FocusRegion>('joinSection');
  const [serverUrl, setServerUrl] = useState<string>(DEFAULT_API_URL);
  const [serverUrlInput, setServerUrlInput] = useState<string>(DEFAULT_API_URL);
  const [showQRCode, setShowQRCode] = useState<boolean>(false);
  const [showServerConfig, setShowServerConfig] = useState<boolean>(false);
  const [serverSaved, setServerSaved] = useState<boolean>(false);

  const user = useAuthStore((state) => state.user);
  const token = useAuthStore((state) => state.token);
  const clearAuth = useAuthStore((state) => state.clearAuth);

  // Handle TV remote D-pad events for region navigation
  useTVEventHandler((evt: { eventType: string }) => {
    // Update focus region tracking based on D-pad
    if (evt.eventType === 'up') {
      if (focusRegion === 'rooms') {
        setFocusRegion('joinSection');
      } else if (focusRegion === 'joinSection' && showServerConfig) {
        setFocusRegion('serverConfig');
      } else if (focusRegion === 'joinSection' || focusRegion === 'serverConfig') {
        setFocusRegion('header');
      }
    } else if (evt.eventType === 'down') {
      if (focusRegion === 'header') {
        setFocusRegion(showServerConfig ? 'serverConfig' : 'joinSection');
      } else if (focusRegion === 'serverConfig') {
        setFocusRegion('joinSection');
      } else if (focusRegion === 'joinSection') {
        setFocusRegion('rooms');
      }
    } else if (evt.eventType === 'right') {
      if (focusRegion !== 'rightPanel') {
        setFocusRegion('rightPanel');
      }
    } else if (evt.eventType === 'left') {
      if (focusRegion === 'rightPanel') {
        setFocusRegion('joinSection');
      }
    }
  });

  // Load server URL from config
  useEffect(() => {
    const loadServerUrl = async () => {
      const url = await configService.getServerUrl();
      setServerUrl(url);
      setServerUrlInput(url);
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
    const interval = setInterval(loadRooms, 10000); // Refresh every 10s
    return () => clearInterval(interval);
  }, [loadRooms]);

  const handleLogout = async () => {
    await AsyncStorage.multiRemove(['@auth_token', '@auth_user']);
    clearAuth();
  };

  const handleSaveServer = async () => {
    let url = serverUrlInput.trim();
    // Ensure URL has protocol
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      url = 'http://' + url;
      setServerUrlInput(url);
    }
    // Remove trailing slash
    if (url.endsWith('/')) {
      url = url.slice(0, -1);
      setServerUrlInput(url);
    }

    await configService.setServerUrl(url);
    setServerUrl(url);
    authService.setBaseUrl(url);
    setServerSaved(true);
    setTimeout(() => setServerSaved(false), 2000);
    loadRooms();
  };

  const joinRoom = (room: string) => {
    navigation.navigate('TVGame', { room: room.trim() || 'main' });
  };

  const renderRoomItem = ({ item, index }: { item: RoomInfo; index: number }) => (
    <FocusableButton
      style={styles.roomCard}
      focusedStyle={styles.roomCardFocused}
      onPress={() => joinRoom(item.name)}
      onFocusChange={(focused) => {
        if (focused) {
          setFocusedItem(`room-${index}`);
          setFocusRegion('rooms');
        }
      }}
      hasTVPreferredFocus={index === 0 && rooms.length > 0 && focusRegion === 'rooms'}
      testID={`room-${index}`}
    >
      <Text style={styles.roomName}>{item.name}</Text>
      <Text style={styles.roomPlayers}>{item.player_count} players</Text>
    </FocusableButton>
  );

  return (
    <View style={styles.container}>
      {/* Header */}
      <TVFocusGuideView style={styles.header} trapFocusUp>
        <View>
          <Text style={styles.title}>🎡 Holiday Wheel</Text>
          <Text style={styles.welcome}>Welcome, {user?.display_name}!</Text>
        </View>
        <View style={styles.headerButtons}>
          <FocusableButton
            style={styles.settingsButton}
            onPress={() => setShowServerConfig(!showServerConfig)}
            onFocusChange={(focused) => {
              if (focused) {
                setFocusedItem('settings');
                setFocusRegion('header');
              }
            }}
            testID="btn-settings"
          >
            <Text style={styles.settingsText}>⚙️ Server</Text>
          </FocusableButton>
          <FocusableButton
            style={styles.logoutButton}
            onPress={handleLogout}
            onFocusChange={(focused) => {
              if (focused) {
                setFocusedItem('logout');
                setFocusRegion('header');
              }
            }}
            testID="btn-logout"
          >
            <Text style={styles.logoutText}>Logout</Text>
          </FocusableButton>
        </View>
      </TVFocusGuideView>

      {/* Server Configuration Panel */}
      {showServerConfig && (
        <TVFocusGuideView style={styles.serverConfigPanel}>
          <Text style={styles.serverConfigTitle}>Server Configuration</Text>
          <View style={styles.serverConfigRow}>
            <TextInput
              style={[
                styles.serverInput,
                focusedItem === 'serverInput' && styles.inputFocused,
              ]}
              placeholder="http://192.168.1.100:5000"
              placeholderTextColor="#666"
              value={serverUrlInput}
              onChangeText={setServerUrlInput}
              onFocus={() => {
                setFocusedItem('serverInput');
                setFocusRegion('serverConfig');
              }}
              onSubmitEditing={handleSaveServer}
              autoCapitalize="none"
              autoCorrect={false}
              hasTVPreferredFocus={showServerConfig && focusRegion === 'serverConfig'}
            />
            <FocusableButton
              style={[
                styles.saveButton,
                serverSaved && styles.saveButtonSaved,
              ]}
              onPress={handleSaveServer}
              onFocusChange={(focused) => {
                if (focused) {
                  setFocusedItem('saveServer');
                  setFocusRegion('serverConfig');
                }
              }}
              testID="btn-save-server"
            >
              <Text style={styles.saveButtonText}>
                {serverSaved ? '✓ Saved' : 'Save'}
              </Text>
            </FocusableButton>
          </View>
          <Text style={styles.serverHint}>
            Enter the IP address of the computer running the backend server
          </Text>
        </TVFocusGuideView>
      )}

      <View style={styles.content}>
        <View style={styles.mainContent}>
          {/* Left side - Room controls */}
          <TVFocusGuideView style={styles.leftPanel} trapFocusLeft>
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
                  onFocus={() => {
                    setFocusedItem('input');
                    setFocusRegion('joinSection');
                  }}
                  onSubmitEditing={() => joinRoom(customRoom)}
                  hasTVPreferredFocus={!showServerConfig && focusRegion === 'joinSection'}
                />
                <FocusableButton
                  style={styles.joinButton}
                  onPress={() => joinRoom(customRoom)}
                  onFocusChange={(focused) => {
                    if (focused) {
                      setFocusedItem('join');
                      setFocusRegion('joinSection');
                    }
                  }}
                  testID="btn-join"
                >
                  <Text style={styles.joinButtonText}>JOIN</Text>
                </FocusableButton>
              </View>
            </TVFocusGuideView>

            {/* Active Rooms */}
            <TVFocusGuideView style={styles.roomsSection}>
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
            </TVFocusGuideView>
          </TVFocusGuideView>

          {/* Right side - Connection info (QR code area - focusable but not a trap) */}
          <TVFocusGuideView style={styles.rightPanel} trapFocusRight>
            <Text style={styles.sectionTitle}>Phone Connection</Text>
            <View style={styles.serverInfo}>
              <Text style={styles.serverLabel}>Server:</Text>
              <Text style={styles.serverUrlText}>{serverUrl.replace(/^https?:\/\//, '')}</Text>
            </View>
            <View style={styles.serverInfo}>
              <Text style={styles.serverLabel}>Room:</Text>
              <Text style={styles.serverUrlText}>{customRoom || 'main'}</Text>
            </View>

            <FocusableButton
              style={styles.qrButton}
              onPress={() => setShowQRCode(!showQRCode)}
              onFocusChange={(focused) => {
                if (focused) {
                  setFocusedItem('qr');
                  setFocusRegion('rightPanel');
                }
              }}
              hasTVPreferredFocus={focusRegion === 'rightPanel'}
              testID="btn-qr"
            >
              <Text style={styles.qrButtonText}>
                {showQRCode ? 'Hide QR Code' : 'Show QR Code'}
              </Text>
            </FocusableButton>

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
          </TVFocusGuideView>
        </View>
      </View>

      {/* Instructions */}
      <View style={styles.footer}>
        <Text style={styles.footerText}>
          ← → Navigate • Select to join • Menu for settings
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
  headerButtons: {
    flexDirection: 'row',
    gap: 16,
  },
  settingsButton: {
    backgroundColor: '#1a0a3e',
    paddingHorizontal: 24,
    paddingVertical: 16,
    borderRadius: 8,
    borderWidth: 2,
    borderColor: '#333',
  },
  settingsText: {
    color: '#888',
    fontSize: 20,
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
  serverConfigPanel: {
    backgroundColor: '#1a0a3e',
    padding: 24,
    marginHorizontal: 40,
    marginTop: 20,
    borderRadius: 12,
    borderWidth: 2,
    borderColor: '#d4af37',
  },
  serverConfigTitle: {
    color: '#d4af37',
    fontSize: 24,
    fontWeight: 'bold',
    marginBottom: 16,
  },
  serverConfigRow: {
    flexDirection: 'row',
    gap: 16,
  },
  serverInput: {
    flex: 1,
    backgroundColor: '#0d0628',
    borderRadius: 8,
    padding: 16,
    color: '#ffffff',
    fontSize: 22,
    borderWidth: 2,
    borderColor: '#333',
  },
  saveButton: {
    backgroundColor: '#4caf50',
    paddingHorizontal: 32,
    paddingVertical: 16,
    borderRadius: 8,
    justifyContent: 'center',
    borderWidth: 2,
    borderColor: 'transparent',
  },
  saveButtonSaved: {
    backgroundColor: '#2e7d32',
  },
  saveButtonText: {
    color: '#fff',
    fontSize: 20,
    fontWeight: 'bold',
  },
  serverHint: {
    color: '#666',
    fontSize: 16,
    marginTop: 12,
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
  serverUrlText: {
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
