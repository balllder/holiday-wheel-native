import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  TVFocusGuideView,
  useTVEventHandler,
  BackHandler,
} from 'react-native';
import { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { RouteProp } from '@react-navigation/native';
import {
  useGameStore,
  useAuthStore,
  socketService,
  ROW_WIDTHS,
} from '@holiday-wheel/shared';
import type { WedgeValue } from '@holiday-wheel/shared';
import type { TVStackParamList } from '../navigation/TVNavigator';

type TVGameScreenProps = {
  navigation: NativeStackNavigationProp<TVStackParamList, 'TVGame'>;
  route: RouteProp<TVStackParamList, 'TVGame'>;
};

const API_URL = 'http://192.168.1.100:5000';
const HOST_CODE = 'holiday'; // Default host code

export function TVGameScreen({ route }: TVGameScreenProps): React.JSX.Element {
  const { room } = route.params;
  const [controlsVisible, setControlsVisible] = useState(false);
  const [focusedControl, setFocusedControl] = useState(0);

  const token = useAuthStore((state) => state.token);
  const connected = useGameStore((state) => state.connected);
  const phase = useGameStore((state) => state.phase);
  const puzzle = useGameStore((state) => state.puzzle);
  const revealed = useGameStore((state) => state.revealed);
  const players = useGameStore((state) => state.players);
  const activeIdx = useGameStore((state) => state.activeIdx);
  const currentWedge = useGameStore((state) => state.currentWedge);
  const isHost = useGameStore((state) => state.isHost);

  // Handle TV remote events
  useTVEventHandler((evt) => {
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

  // Connect to socket and auto-claim host
  useEffect(() => {
    socketService.connect(API_URL, token || undefined);
    socketService.joinRoom(room);

    // Auto-claim host for TV display
    const claimTimer = setTimeout(() => {
      socketService.claimHost(room, HOST_CODE);
    }, 500);

    return () => {
      clearTimeout(claimTimer);
      socketService.disconnect();
    };
  }, [room, token]);

  const controls = [
    { label: 'SPIN', action: () => socketService.spin(room) },
    { label: 'NEW PUZZLE', action: () => socketService.newPuzzle(room) },
    { label: 'TOSS-UP', action: () => socketService.startTossup(room) },
    { label: 'END TOSS-UP', action: () => socketService.endTossup(room) },
    { label: 'FINAL', action: () => socketService.startFinal(room) },
    { label: 'END FINAL', action: () => socketService.endFinal(room) },
    { label: 'NEW GAME', action: () => socketService.newGame(room) },
    { label: 'REVEAL', action: () => socketService.revealAll(room) },
  ];

  const formatWedge = (wedge: WedgeValue | null): string => {
    if (!wedge) return '--';
    if (typeof wedge === 'number') return `$${wedge}`;
    if (typeof wedge === 'object') return wedge.name;
    return wedge;
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
          <View style={styles.wheelPlaceholder}>
            <Text style={styles.wheelText}>🎡</Text>
          </View>
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
      </View>

      {/* Bottom: Players bar */}
      <View style={styles.playersBar}>
        {players.map((player, idx) => (
          <View
            key={player.id}
            style={[styles.playerCard, idx === activeIdx && styles.activePlayer]}
          >
            <Text style={styles.playerName}>{player.name}</Text>
            <Text style={styles.playerTotal}>${player.total}</Text>
            <Text style={styles.playerRound}>Round: ${player.round_bank}</Text>
          </View>
        ))}
      </View>

      {/* Host controls overlay */}
      {controlsVisible && (
        <View style={styles.controlsOverlay}>
          <TVFocusGuideView style={styles.controlsBar} autoFocus>
            {controls.map((ctrl, idx) => (
              <TouchableOpacity
                key={ctrl.label}
                style={[
                  styles.controlButton,
                  focusedControl === idx && styles.controlButtonFocused,
                ]}
                onPress={ctrl.action}
                onFocus={() => setFocusedControl(idx)}
                hasTVPreferredFocus={idx === 0}
              >
                <Text style={styles.controlButtonText}>{ctrl.label}</Text>
              </TouchableOpacity>
            ))}
          </TVFocusGuideView>
          <Text style={styles.controlsHint}>
            Press Menu to hide controls
          </Text>
        </View>
      )}

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
          <Text style={styles.menuHintText}>Press Menu for controls</Text>
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
    width: 320,
    alignItems: 'center',
    justifyContent: 'center',
  },
  wheelPlaceholder: {
    width: 280,
    height: 280,
    borderRadius: 140,
    backgroundColor: '#1a0a3e',
    borderWidth: 4,
    borderColor: '#d4af37',
    alignItems: 'center',
    justifyContent: 'center',
  },
  wheelText: {
    fontSize: 120,
  },
  wedgeDisplay: {
    marginTop: 24,
    backgroundColor: '#d4af37',
    paddingHorizontal: 32,
    paddingVertical: 16,
    borderRadius: 8,
  },
  wedgeValue: {
    fontSize: 32,
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
    fontSize: 36,
    fontWeight: 'bold',
    color: '#ffd700',
    marginBottom: 24,
  },
  puzzleBoard: {
    backgroundColor: '#1a5cb8',
    borderRadius: 12,
    padding: 20,
    borderWidth: 4,
    borderColor: '#d4af37',
  },
  puzzleRow: {
    flexDirection: 'row',
    justifyContent: 'center',
  },
  puzzleCell: {
    width: 52,
    height: 62,
    margin: 3,
    borderRadius: 4,
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
    fontSize: 36,
    fontWeight: 'bold',
    color: '#1a1a2e',
  },
  phaseBadge: {
    marginTop: 24,
    backgroundColor: '#6c5ce7',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  phaseText: {
    color: '#fff',
    fontSize: 24,
    fontWeight: 'bold',
  },
  playersBar: {
    flexDirection: 'row',
    justifyContent: 'center',
    gap: 20,
    paddingHorizontal: 40,
    paddingVertical: 20,
    borderTopWidth: 2,
    borderTopColor: '#333',
  },
  playerCard: {
    backgroundColor: '#1a0a3e',
    borderRadius: 12,
    padding: 20,
    minWidth: 180,
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
    fontSize: 22,
    fontWeight: 'bold',
  },
  playerTotal: {
    color: '#d4af37',
    fontSize: 32,
    fontWeight: 'bold',
    marginTop: 8,
  },
  playerRound: {
    color: '#888',
    fontSize: 18,
    marginTop: 4,
  },
  controlsOverlay: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    backgroundColor: 'rgba(13, 6, 40, 0.95)',
    padding: 24,
    borderTopWidth: 2,
    borderTopColor: '#d4af37',
  },
  controlsBar: {
    flexDirection: 'row',
    justifyContent: 'center',
    flexWrap: 'wrap',
    gap: 16,
  },
  controlButton: {
    backgroundColor: '#d4af37',
    paddingHorizontal: 28,
    paddingVertical: 18,
    borderRadius: 10,
    borderWidth: 3,
    borderColor: 'transparent',
  },
  controlButtonFocused: {
    borderColor: '#fff',
    transform: [{ scale: 1.05 }],
  },
  controlButtonText: {
    color: '#1a0a3e',
    fontSize: 22,
    fontWeight: 'bold',
  },
  controlsHint: {
    color: '#888',
    fontSize: 18,
    textAlign: 'center',
    marginTop: 16,
  },
  connStatus: {
    position: 'absolute',
    top: 20,
    right: 20,
    paddingHorizontal: 16,
    paddingVertical: 8,
    borderRadius: 20,
  },
  connGreen: {
    backgroundColor: 'rgba(76, 175, 80, 0.3)',
  },
  connRed: {
    backgroundColor: 'rgba(244, 67, 54, 0.3)',
  },
  connText: {
    color: '#fff',
    fontSize: 16,
  },
  menuHint: {
    position: 'absolute',
    bottom: 20,
    left: 0,
    right: 0,
    alignItems: 'center',
  },
  menuHintText: {
    color: '#555',
    fontSize: 18,
  },
});
