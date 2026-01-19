import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  useTVEventHandler,
  BackHandler,
  Alert,
} from 'react-native';
import { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { RouteProp } from '@react-navigation/native';
import {
  useGameStore,
  useAuthStore,
  socketService,
  configService,
  ROW_WIDTHS,
  AnimatedWheel,
} from '@holiday-wheel/shared';
import type { WedgeValue } from '@holiday-wheel/shared';
import type { TVStackParamList } from '../navigation/TVNavigator';
import { HostControlPanel } from '../components/HostControlPanel';
import { RoomQRCode } from '../components/RoomQRCode';

type TVGameScreenProps = {
  navigation: NativeStackNavigationProp<TVStackParamList, 'TVGame'>;
  route: RouteProp<TVStackParamList, 'TVGame'>;
};

const HOST_CODE = 'holiday'; // Default host code

export function TVGameScreen({ route }: TVGameScreenProps): React.JSX.Element {
  const { room } = route.params;
  const [controlsVisible, setControlsVisible] = useState(false);
  const [showQRCode, setShowQRCode] = useState(true);
  const [serverUrl, setServerUrl] = useState<string | null>(null);

  const token = useAuthStore((state) => state.token);
  const connected = useGameStore((state) => state.connected);
  const phase = useGameStore((state) => state.phase);
  const puzzle = useGameStore((state) => state.puzzle);
  const revealed = useGameStore((state) => state.revealed);
  const players = useGameStore((state) => state.players);
  const activeIdx = useGameStore((state) => state.activeIdx);
  const currentWedge = useGameStore((state) => state.currentWedge);
  const wheelSlots = useGameStore((state) => state.wheelSlots);
  const lastSpinIndex = useGameStore((state) => state.lastSpinIndex);
  const isHost = useGameStore((state) => state.isHost);

  // Handle TV remote events (Menu/PlayPause toggles host controls)
  useTVEventHandler((evt: { eventType: string }) => {
    if (evt.eventType === 'menu' || evt.eventType === 'playPause') {
      setControlsVisible((prev) => !prev);
    }
  });

  // Handle back button
  useEffect(() => {
    const backHandler = BackHandler.addEventListener('hardwareBackPress', () => {
      if (controlsVisible) {
        setControlsVisible(false);
        return true;
      }
      return false;
    });
    return () => backHandler.remove();
  }, [controlsVisible]);

  // Load server URL on mount
  useEffect(() => {
    const loadServerUrl = async () => {
      const url = await configService.getServerUrl();
      setServerUrl(url);
    };
    loadServerUrl();
  }, []);

  // Connect to socket and auto-claim host
  useEffect(() => {
    if (!serverUrl) return;

    const logout = useAuthStore.getState().logout;

    socketService.connect(serverUrl, token || undefined);
    socketService.joinRoom(room);

    // Auto-claim host for TV display
    const claimTimer = setTimeout(() => {
      socketService.claimHost(room, HOST_CODE);
    }, 500);

    // Set up session invalidation handler
    socketService.setSessionInvalidatedCallback(() => {
      Alert.alert(
        'Session Expired',
        'You have been logged out because your account was accessed from another device.'
      );
      logout();
    });

    return () => {
      clearTimeout(claimTimer);
      socketService.disconnect();
    };
  }, [room, token, serverUrl]);

  // Hide QR code when players join
  useEffect(() => {
    if (players.length > 0) {
      setShowQRCode(false);
    }
  }, [players.length]);

  const formatWedge = (wedge: WedgeValue | null): string => {
    if (!wedge) return '--';
    if (typeof wedge === 'number') return `$${wedge}`;
    if (typeof wedge === 'object') return wedge.name;
    return wedge;
  };

  // Format number as cash with commas
  const formatCash = (amount: number) => {
    return '$' + amount.toLocaleString();
  };

  // Render puzzle board
  const renderPuzzleBoard = () => {
    const answer = puzzle.answer || '';
    const rows = layoutToRows(answer);

    return (
      <View style={styles.puzzleBoard}>
        {rows.map((row, rowIdx) => (
          <View key={rowIdx} style={styles.puzzleRow}>
            {row.map((cell, cellIdx) => {
              const isLetter = /[A-Z]/i.test(cell || '');
              const isRevealed = isLetter && revealed.has(cell!.toUpperCase());
              return (
                <View
                  key={cellIdx}
                  style={[
                    styles.puzzleCell,
                    !isLetter && styles.emptyCell,
                    isLetter && !isRevealed && styles.hiddenCell,
                  ]}
                >
                  {isRevealed && (
                    <Text style={styles.puzzleLetter}>{cell!.toUpperCase()}</Text>
                  )}
                </View>
              );
            })}
          </View>
        ))}
      </View>
    );
  };

  // Simple layout function
  const layoutToRows = (answer: string): (string | null)[][] => {
    const words = answer.split(' ');
    const rows: (string | null)[][] = [[], [], [], []];
    let rowIdx = 1; // Start in middle rows

    words.forEach((word, i) => {
      const chars = word.split('');
      if (rows[rowIdx].length + chars.length + (i > 0 ? 1 : 0) > ROW_WIDTHS[rowIdx]) {
        rowIdx = Math.min(rowIdx + 1, 3);
      }
      if (i > 0 && rows[rowIdx].length > 0) {
        rows[rowIdx].push(' ');
      }
      chars.forEach((c) => rows[rowIdx].push(c));
    });

    // Pad rows to width and center
    return rows.map((row, idx) => {
      const width = ROW_WIDTHS[idx];
      const padding = Math.floor((width - row.length) / 2);
      const padded: (string | null)[] = [];
      for (let i = 0; i < padding; i++) padded.push(null);
      padded.push(...row);
      while (padded.length < width) padded.push(null);
      return padded;
    });
  };

  return (
    <View style={styles.container}>
      {/* Main game display */}
      <View style={styles.main}>
        {/* Left: Wheel area */}
        <View style={styles.wheelSection}>
          {wheelSlots.length > 0 ? (
            <AnimatedWheel
              wheelSlots={wheelSlots}
              lastSpinIndex={lastSpinIndex}
              size={500}
            />
          ) : (
            <View style={styles.wheelPlaceholder}>
              <Text style={styles.wheelText}>🎡</Text>
            </View>
          )}
          <View style={styles.wedgeDisplay}>
            <Text style={styles.wedgeValue}>{formatWedge(currentWedge)}</Text>
          </View>
        </View>

        {/* Center: Puzzle */}
        <View style={styles.puzzleSection}>
          <Text style={styles.category}>{puzzle.category || 'Loading...'}</Text>
          {renderPuzzleBoard()}
          <View style={styles.phaseBadge}>
            <Text style={styles.phaseText}>{phase.toUpperCase()}</Text>
          </View>
        </View>

        {/* Right: QR Code for joining (when no players) */}
        {showQRCode && serverUrl && (
          <View style={styles.qrSection}>
            <RoomQRCode room={room} serverUrl={serverUrl} size={180} />
          </View>
        )}
      </View>

      {/* Bottom: Players bar */}
      <View style={styles.playersBar}>
        {players.length === 0 ? (
          <Text style={styles.waitingText}>Waiting for players to join...</Text>
        ) : (
          players.map((player, idx) => (
            <View
              key={player.id}
              style={[styles.playerCard, idx === activeIdx && styles.activePlayer]}
            >
              <Text style={styles.playerName}>{player.name}</Text>
              <Text style={styles.playerTotal}>{formatCash(player.total)}</Text>
              <Text style={styles.playerRound}>Round: {formatCash(player.round_bank)}</Text>
            </View>
          ))
        )}
      </View>

      {/* Host controls overlay */}
      <HostControlPanel
        room={room}
        visible={controlsVisible}
        onClose={() => setControlsVisible(false)}
      />

      {/* Connection status */}
      <View style={[styles.connStatus, connected ? styles.connGreen : styles.connRed]}>
        <Text style={styles.connText}>
          {connected ? '● Connected' : '○ Connecting...'}
          {isHost && ' (Host)'}
        </Text>
      </View>

      {/* Menu hint when controls hidden */}
      {!controlsVisible && (
        <View style={styles.menuHint}>
          <Text style={styles.menuHintText}>Press Menu for host controls</Text>
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0d0628',
  },
  main: {
    flex: 1,
    flexDirection: 'row',
    padding: 40,
    paddingBottom: 20,
  },
  wheelSection: {
    width: 520,
    alignItems: 'center',
    justifyContent: 'center',
  },
  wheelPlaceholder: {
    width: 500,
    height: 500,
    borderRadius: 250,
    backgroundColor: '#1a0a3e',
    borderWidth: 4,
    borderColor: '#d4af37',
    alignItems: 'center',
    justifyContent: 'center',
  },
  wheelText: {
    fontSize: 200,
  },
  wedgeDisplay: {
    marginTop: 24,
    backgroundColor: '#d4af37',
    paddingHorizontal: 48,
    paddingVertical: 20,
    borderRadius: 12,
  },
  wedgeValue: {
    fontSize: 40,
    fontWeight: 'bold',
    color: '#1a0a3e',
  },
  puzzleSection: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    paddingHorizontal: 40,
  },
  category: {
    fontSize: 42,
    fontWeight: 'bold',
    color: '#ffd700',
    marginBottom: 32,
  },
  puzzleBoard: {
    backgroundColor: '#1a5cb8',
    borderRadius: 16,
    padding: 24,
    borderWidth: 4,
    borderColor: '#d4af37',
  },
  puzzleRow: {
    flexDirection: 'row',
    justifyContent: 'center',
  },
  puzzleCell: {
    width: 56,
    height: 68,
    margin: 4,
    borderRadius: 6,
    backgroundColor: '#fff',
    alignItems: 'center',
    justifyContent: 'center',
  },
  emptyCell: {
    backgroundColor: '#228b22',
  },
  hiddenCell: {
    backgroundColor: '#fff',
  },
  puzzleLetter: {
    fontSize: 42,
    fontWeight: 'bold',
    color: '#1a1a2e',
  },
  phaseBadge: {
    marginTop: 32,
    backgroundColor: '#6c5ce7',
    paddingHorizontal: 32,
    paddingVertical: 16,
    borderRadius: 12,
  },
  phaseText: {
    color: '#fff',
    fontSize: 28,
    fontWeight: 'bold',
  },
  qrSection: {
    width: 240,
    alignItems: 'center',
    justifyContent: 'center',
  },
  playersBar: {
    flexDirection: 'row',
    justifyContent: 'center',
    gap: 24,
    paddingHorizontal: 40,
    paddingVertical: 24,
    borderTopWidth: 2,
    borderTopColor: '#333',
    minHeight: 140,
  },
  waitingText: {
    color: '#888',
    fontSize: 24,
  },
  playerCard: {
    backgroundColor: '#1a0a3e',
    borderRadius: 16,
    padding: 24,
    minWidth: 200,
    alignItems: 'center',
    borderWidth: 3,
    borderColor: 'transparent',
  },
  activePlayer: {
    borderColor: '#d4af37',
    backgroundColor: '#2a1a4e',
  },
  playerName: {
    color: '#fff',
    fontSize: 26,
    fontWeight: 'bold',
  },
  playerTotal: {
    color: '#d4af37',
    fontSize: 36,
    fontWeight: 'bold',
    marginTop: 8,
  },
  playerRound: {
    color: '#888',
    fontSize: 20,
    marginTop: 4,
  },
  connStatus: {
    position: 'absolute',
    top: 24,
    right: 24,
    paddingHorizontal: 20,
    paddingVertical: 10,
    borderRadius: 24,
  },
  connGreen: {
    backgroundColor: 'rgba(76, 175, 80, 0.3)',
  },
  connRed: {
    backgroundColor: 'rgba(244, 67, 54, 0.3)',
  },
  connText: {
    color: '#fff',
    fontSize: 18,
  },
  menuHint: {
    position: 'absolute',
    bottom: 24,
    left: 0,
    right: 0,
    alignItems: 'center',
  },
  menuHintText: {
    color: '#555',
    fontSize: 20,
  },
});
