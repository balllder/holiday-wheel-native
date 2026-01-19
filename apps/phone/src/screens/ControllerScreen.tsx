import React, { useEffect, useState, useRef } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  Alert,
  Vibration,
} from 'react-native';
import { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { RouteProp } from '@react-navigation/native';
import {
  useGameStore,
  useAuthStore,
  socketService,
  configService,
  selectIsMyTurn,
  selectCanBuzz,
  selectMyPlayer,
  VOWELS,
} from '@holiday-wheel/shared';
import type { RootStackParamList } from '../navigation/AppNavigator';

type ControllerScreenProps = {
  navigation: NativeStackNavigationProp<RootStackParamList, 'Controller'>;
  route: RouteProp<RootStackParamList, 'Controller'>;
};

export function ControllerScreen({ route }: ControllerScreenProps): React.JSX.Element {
  const { room } = route.params;
  const [letterInput, setLetterInput] = useState('');
  const [solveInput, setSolveInput] = useState('');
  const [serverUrl, setServerUrl] = useState<string | null>(null);

  const token = useAuthStore((state) => state.token);
  const connected = useGameStore((state) => state.connected);
  const phase = useGameStore((state) => state.phase);
  const puzzle = useGameStore((state) => state.puzzle);
  const myPlayerIdx = useGameStore((state) => state.myPlayerIdx);
  const currentWedge = useGameStore((state) => state.currentWedge);
  const isMyTurn = useGameStore(selectIsMyTurn);
  const canBuzz = useGameStore(selectCanBuzz);
  const myPlayer = useGameStore(selectMyPlayer);

  // Ref for auto-focusing letter input after spin
  const letterInputRef = useRef<TextInput>(null);
  const prevWedgeRef = useRef(currentWedge);

  // Load server URL on mount
  useEffect(() => {
    const loadServerUrl = async () => {
      const url = await configService.getServerUrl();
      setServerUrl(url);
    };
    loadServerUrl();
  }, []);

  // Connect to socket when server URL is available
  useEffect(() => {
    if (!serverUrl) return;

    // Connect to socket
    socketService.connect(serverUrl, token || undefined);
    socketService.joinRoom(room);

    // Set up toast handler with vibration
    socketService.setToastCallback((msg) => {
      Vibration.vibrate(100);
      Alert.alert('Notice', msg);
    });

    return () => {
      socketService.disconnect();
    };
  }, [room, token, serverUrl]);

  // Auto-join game when connected
  useEffect(() => {
    if (connected && myPlayerIdx === null) {
      socketService.joinGame(room);
    }
  }, [connected, myPlayerIdx, room]);

  // Focus letter input when spin result comes back
  useEffect(() => {
    if (currentWedge !== null && currentWedge !== prevWedgeRef.current) {
      prevWedgeRef.current = currentWedge;
      // Small delay to ensure UI has updated
      setTimeout(() => {
        letterInputRef.current?.focus();
      }, 100);
    }
  }, [currentWedge]);

  const handleSpin = () => {
    Vibration.vibrate(50);
    socketService.spin(room);
  };

  const handleGuess = () => {
    const letter = letterInput.toUpperCase().trim();
    if (letter.length === 1) {
      Vibration.vibrate(50);
      if (VOWELS.includes(letter)) {
        socketService.buyVowel(room, letter);
      } else {
        socketService.guess(room, letter);
      }
      setLetterInput('');
    }
  };

  const handleSolve = () => {
    if (solveInput.trim()) {
      Vibration.vibrate(50);
      socketService.solve(room, solveInput.trim());
      setSolveInput('');
    }
  };

  const handleBuzz = () => {
    Vibration.vibrate([0, 100, 50, 100]); // Double vibration for buzz
    socketService.buzz(room);
  };

  return (
    <View style={styles.container}>
      {/* Status bar */}
      <View style={[styles.statusBar, connected ? styles.connected : styles.disconnected]}>
        <View style={styles.statusLeft}>
          <Text style={styles.statusDot}>{connected ? '●' : '○'}</Text>
          <Text style={styles.roomText}>{room}</Text>
        </View>
        <Text style={styles.phaseText}>{phase.toUpperCase()}</Text>
      </View>

      {/* Minimal info display */}
      <View style={styles.infoSection}>
        <Text style={styles.categoryLabel}>Category</Text>
        <Text style={styles.category}>{puzzle.category || '...'}</Text>
        {myPlayer && (
          <View style={styles.scoreBox}>
            <Text style={styles.scoreName}>{myPlayer.name}</Text>
            <Text style={styles.scoreValue}>${myPlayer.round_bank}</Text>
          </View>
        )}
      </View>

      {/* Main controls */}
      <View style={styles.controlsSection}>
        {phase === 'tossup' ? (
          // Toss-up: Big buzz button
          <TouchableOpacity
            style={[styles.buzzButton, !canBuzz && styles.disabled]}
            onPress={handleBuzz}
            disabled={!canBuzz}
            activeOpacity={0.7}
          >
            <Text style={styles.buzzText}>BUZZ!</Text>
            <Text style={styles.buzzSubtext}>Tap to buzz in</Text>
          </TouchableOpacity>
        ) : (
          // Normal round controls
          <>
            <TouchableOpacity
              style={[styles.spinButton, !isMyTurn && styles.disabled]}
              onPress={handleSpin}
              disabled={!isMyTurn}
              activeOpacity={0.7}
            >
              <Text style={styles.spinText}>SPIN</Text>
            </TouchableOpacity>

            <View style={styles.letterRow}>
              <TextInput
                ref={letterInputRef}
                style={styles.letterInput}
                placeholder="?"
                placeholderTextColor="#666"
                value={letterInput}
                onChangeText={setLetterInput}
                maxLength={1}
                autoCapitalize="characters"
                keyboardType="default"
              />
              <TouchableOpacity
                style={[styles.guessButton, !isMyTurn && styles.disabled]}
                onPress={handleGuess}
                disabled={!isMyTurn}
              >
                <Text style={styles.guessText}>GUESS</Text>
              </TouchableOpacity>
            </View>

            <View style={styles.solveRow}>
              <TextInput
                style={styles.solveInput}
                placeholder="Solve the puzzle..."
                placeholderTextColor="#666"
                value={solveInput}
                onChangeText={setSolveInput}
                autoCapitalize="characters"
              />
              <TouchableOpacity
                style={[styles.solveButton, !isMyTurn && styles.disabled]}
                onPress={handleSolve}
                disabled={!isMyTurn}
              >
                <Text style={styles.solveText}>SOLVE</Text>
              </TouchableOpacity>
            </View>
          </>
        )}
      </View>

      {/* Turn indicator */}
      <View style={styles.turnIndicator}>
        {isMyTurn ? (
          <Text style={styles.yourTurn}>YOUR TURN!</Text>
        ) : (
          <Text style={styles.waiting}>Waiting for your turn...</Text>
        )}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0d0628',
  },
  statusBar: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 12,
  },
  connected: {
    backgroundColor: 'rgba(76, 175, 80, 0.2)',
  },
  disconnected: {
    backgroundColor: 'rgba(244, 67, 54, 0.2)',
  },
  statusLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  statusDot: {
    color: '#4caf50',
    fontSize: 16,
  },
  roomText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: 'bold',
  },
  phaseText: {
    color: '#d4af37',
    fontWeight: 'bold',
    fontSize: 14,
  },
  infoSection: {
    padding: 16,
    alignItems: 'center',
  },
  categoryLabel: {
    color: '#888',
    fontSize: 12,
    textTransform: 'uppercase',
  },
  category: {
    color: '#ffd700',
    fontSize: 24,
    fontWeight: 'bold',
    textAlign: 'center',
    marginTop: 4,
  },
  scoreBox: {
    backgroundColor: '#1a0a3e',
    borderRadius: 8,
    padding: 12,
    marginTop: 16,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#d4af37',
  },
  scoreName: {
    color: '#fff',
    fontSize: 14,
  },
  scoreValue: {
    color: '#d4af37',
    fontSize: 28,
    fontWeight: 'bold',
  },
  controlsSection: {
    flex: 1,
    padding: 16,
    justifyContent: 'center',
  },
  buzzButton: {
    backgroundColor: '#f44336',
    borderRadius: 16,
    paddingVertical: 64,
    alignItems: 'center',
    justifyContent: 'center',
  },
  buzzText: {
    color: '#fff',
    fontSize: 48,
    fontWeight: 'bold',
  },
  buzzSubtext: {
    color: 'rgba(255,255,255,0.7)',
    fontSize: 16,
    marginTop: 8,
  },
  spinButton: {
    backgroundColor: '#4caf50',
    borderRadius: 16,
    paddingVertical: 48,
    alignItems: 'center',
    marginBottom: 16,
  },
  spinText: {
    color: '#fff',
    fontSize: 36,
    fontWeight: 'bold',
  },
  letterRow: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 12,
  },
  letterInput: {
    width: 80,
    backgroundColor: '#1a0a3e',
    borderRadius: 12,
    padding: 20,
    color: '#fff',
    fontSize: 32,
    textAlign: 'center',
    fontWeight: 'bold',
    borderWidth: 2,
    borderColor: '#333',
  },
  guessButton: {
    flex: 1,
    backgroundColor: '#d4af37',
    borderRadius: 12,
    alignItems: 'center',
    justifyContent: 'center',
  },
  guessText: {
    color: '#1a0a3e',
    fontSize: 24,
    fontWeight: 'bold',
  },
  solveRow: {
    flexDirection: 'row',
    gap: 12,
  },
  solveInput: {
    flex: 1,
    backgroundColor: '#1a0a3e',
    borderRadius: 12,
    padding: 16,
    color: '#fff',
    fontSize: 18,
    borderWidth: 2,
    borderColor: '#333',
  },
  solveButton: {
    backgroundColor: '#9c27b0',
    borderRadius: 12,
    paddingHorizontal: 24,
    alignItems: 'center',
    justifyContent: 'center',
  },
  solveText: {
    color: '#fff',
    fontSize: 18,
    fontWeight: 'bold',
  },
  disabled: {
    opacity: 0.4,
  },
  turnIndicator: {
    padding: 16,
    alignItems: 'center',
  },
  yourTurn: {
    color: '#4caf50',
    fontSize: 24,
    fontWeight: 'bold',
  },
  waiting: {
    color: '#888',
    fontSize: 16,
  },
});
