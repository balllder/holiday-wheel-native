import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  ScrollView,
  Alert,
} from 'react-native';
import { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { RouteProp } from '@react-navigation/native';
import {
  useGameStore,
  useAuthStore,
  socketService,
  selectIsMyTurn,
  selectCanBuzz,
  VOWELS,
} from '@holiday-wheel/shared';
import type { RootStackParamList } from '../navigation/AppNavigator';

type GameScreenProps = {
  navigation: NativeStackNavigationProp<RootStackParamList, 'Game'>;
  route: RouteProp<RootStackParamList, 'Game'>;
};

const API_URL = 'http://10.0.2.2:5000';

export function GameScreen({ route, navigation }: GameScreenProps): React.JSX.Element {
  const { room } = route.params;
  const [letterInput, setLetterInput] = useState('');
  const [solveInput, setSolveInput] = useState('');

  const token = useAuthStore((state) => state.token);
  const connected = useGameStore((state) => state.connected);
  const phase = useGameStore((state) => state.phase);
  const puzzle = useGameStore((state) => state.puzzle);
  const revealed = useGameStore((state) => state.revealed);
  const players = useGameStore((state) => state.players);
  const activeIdx = useGameStore((state) => state.activeIdx);
  const myPlayerIdx = useGameStore((state) => state.myPlayerIdx);
  const currentWedge = useGameStore((state) => state.currentWedge);
  const isMyTurn = useGameStore(selectIsMyTurn);
  const canBuzz = useGameStore(selectCanBuzz);

  useEffect(() => {
    // Connect to socket
    socketService.connect(API_URL, token || undefined);
    socketService.joinRoom(room);

    // Set up toast handler
    socketService.setToastCallback((msg) => {
      Alert.alert('Notice', msg);
    });

    // Cleanup on unmount
    return () => {
      socketService.disconnect();
    };
  }, [room, token]);

  // Join game as player when connected
  useEffect(() => {
    if (connected && myPlayerIdx === null) {
      socketService.joinGame(room);
    }
  }, [connected, myPlayerIdx, room]);

  const handleSpin = () => {
    socketService.spin(room);
  };

  const handleGuess = () => {
    const letter = letterInput.toUpperCase().trim();
    if (letter.length === 1) {
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
      socketService.solve(room, solveInput.trim());
      setSolveInput('');
    }
  };

  const handleBuzz = () => {
    socketService.buzz(room);
  };

  // Build puzzle display
  const renderPuzzleBoard = () => {
    const answer = puzzle.answer || '';
    return (
      <View style={styles.puzzleBoard}>
        <Text style={styles.category}>{puzzle.category || 'Loading...'}</Text>
        <View style={styles.puzzleGrid}>
          {answer.split('').map((char, idx) => {
            const isLetter = /[A-Z]/i.test(char);
            const isRevealed = isLetter && revealed.has(char.toUpperCase());
            return (
              <View
                key={idx}
                style={[
                  styles.puzzleCell,
                  !isLetter && styles.emptyCell,
                  isLetter && !isRevealed && styles.hiddenCell,
                ]}
              >
                {isRevealed && <Text style={styles.puzzleLetter}>{char.toUpperCase()}</Text>}
              </View>
            );
          })}
        </View>
      </View>
    );
  };

  const renderPlayers = () => (
    <View style={styles.playersContainer}>
      {players.map((player, idx) => (
        <View
          key={player.id}
          style={[
            styles.playerCard,
            idx === activeIdx && styles.activePlayer,
            idx === myPlayerIdx && styles.myPlayer,
          ]}
        >
          <Text style={styles.playerName}>{player.name}</Text>
          <Text style={styles.playerScore}>${player.total}</Text>
          <Text style={styles.playerRound}>Round: ${player.round_bank}</Text>
          {idx === myPlayerIdx && <Text style={styles.youBadge}>You</Text>}
        </View>
      ))}
    </View>
  );

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      {/* Connection status */}
      <View style={[styles.statusBar, connected ? styles.connected : styles.disconnected]}>
        <Text style={styles.statusText}>
          {connected ? '● Connected' : '○ Connecting...'}
        </Text>
        <Text style={styles.phaseText}>{phase.toUpperCase()}</Text>
      </View>

      {/* Puzzle Board */}
      {renderPuzzleBoard()}

      {/* Current wedge */}
      {currentWedge && (
        <View style={styles.wedgeDisplay}>
          <Text style={styles.wedgeText}>
            {typeof currentWedge === 'number'
              ? `$${currentWedge}`
              : typeof currentWedge === 'object'
              ? currentWedge.name
              : currentWedge}
          </Text>
        </View>
      )}

      {/* Game Controls */}
      <View style={styles.controls}>
        {phase === 'normal' && (
          <>
            <TouchableOpacity
              style={[styles.button, styles.spinButton, !isMyTurn && styles.disabled]}
              onPress={handleSpin}
              disabled={!isMyTurn}
            >
              <Text style={styles.buttonText}>SPIN</Text>
            </TouchableOpacity>

            <View style={styles.inputRow}>
              <TextInput
                style={styles.letterInput}
                placeholder="Letter"
                placeholderTextColor="#888"
                value={letterInput}
                onChangeText={setLetterInput}
                maxLength={1}
                autoCapitalize="characters"
              />
              <TouchableOpacity
                style={[styles.button, !isMyTurn && styles.disabled]}
                onPress={handleGuess}
                disabled={!isMyTurn}
              >
                <Text style={styles.buttonText}>GUESS</Text>
              </TouchableOpacity>
            </View>

            <View style={styles.inputRow}>
              <TextInput
                style={styles.solveInput}
                placeholder="Solve the puzzle..."
                placeholderTextColor="#888"
                value={solveInput}
                onChangeText={setSolveInput}
              />
              <TouchableOpacity
                style={[styles.button, !isMyTurn && styles.disabled]}
                onPress={handleSolve}
                disabled={!isMyTurn}
              >
                <Text style={styles.buttonText}>SOLVE</Text>
              </TouchableOpacity>
            </View>
          </>
        )}

        {phase === 'tossup' && (
          <TouchableOpacity
            style={[styles.button, styles.buzzButton, !canBuzz && styles.disabled]}
            onPress={handleBuzz}
            disabled={!canBuzz}
          >
            <Text style={styles.buzzButtonText}>BUZZ!</Text>
          </TouchableOpacity>
        )}
      </View>

      {/* Players */}
      {renderPlayers()}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0d0628',
  },
  content: {
    padding: 16,
  },
  statusBar: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 8,
    borderRadius: 8,
    marginBottom: 16,
  },
  connected: {
    backgroundColor: 'rgba(76, 175, 80, 0.2)',
  },
  disconnected: {
    backgroundColor: 'rgba(244, 67, 54, 0.2)',
  },
  statusText: {
    color: '#fff',
    fontSize: 14,
  },
  phaseText: {
    color: '#d4af37',
    fontWeight: 'bold',
    fontSize: 14,
  },
  puzzleBoard: {
    backgroundColor: '#1a5cb8',
    borderRadius: 8,
    padding: 16,
    marginBottom: 16,
    borderWidth: 3,
    borderColor: '#d4af37',
  },
  category: {
    color: '#ffd700',
    fontSize: 18,
    fontWeight: 'bold',
    textAlign: 'center',
    marginBottom: 16,
  },
  puzzleGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'center',
  },
  puzzleCell: {
    width: 28,
    height: 34,
    margin: 2,
    borderRadius: 3,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#fff',
  },
  emptyCell: {
    backgroundColor: '#228b22',
  },
  hiddenCell: {
    backgroundColor: '#fff',
  },
  puzzleLetter: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#1a1a2e',
  },
  wedgeDisplay: {
    backgroundColor: '#d4af37',
    borderRadius: 8,
    padding: 16,
    marginBottom: 16,
    alignItems: 'center',
  },
  wedgeText: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#1a0a3e',
  },
  controls: {
    marginBottom: 16,
  },
  button: {
    backgroundColor: '#d4af37',
    borderRadius: 8,
    padding: 16,
    alignItems: 'center',
    marginVertical: 4,
  },
  spinButton: {
    backgroundColor: '#4caf50',
  },
  buzzButton: {
    backgroundColor: '#f44336',
    paddingVertical: 32,
  },
  disabled: {
    opacity: 0.5,
  },
  buttonText: {
    color: '#1a0a3e',
    fontSize: 18,
    fontWeight: 'bold',
  },
  buzzButtonText: {
    color: '#fff',
    fontSize: 24,
    fontWeight: 'bold',
  },
  inputRow: {
    flexDirection: 'row',
    gap: 8,
    marginVertical: 4,
  },
  letterInput: {
    width: 80,
    backgroundColor: '#1a0a3e',
    borderRadius: 8,
    padding: 16,
    color: '#fff',
    fontSize: 20,
    textAlign: 'center',
    borderWidth: 1,
    borderColor: '#333',
  },
  solveInput: {
    flex: 1,
    backgroundColor: '#1a0a3e',
    borderRadius: 8,
    padding: 16,
    color: '#fff',
    fontSize: 16,
    borderWidth: 1,
    borderColor: '#333',
  },
  playersContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'center',
    gap: 8,
  },
  playerCard: {
    backgroundColor: '#1a0a3e',
    borderRadius: 8,
    padding: 12,
    minWidth: 100,
    alignItems: 'center',
    borderWidth: 2,
    borderColor: 'transparent',
  },
  activePlayer: {
    borderColor: '#d4af37',
  },
  myPlayer: {
    backgroundColor: '#2a1a4e',
  },
  playerName: {
    color: '#fff',
    fontSize: 14,
    fontWeight: 'bold',
  },
  playerScore: {
    color: '#d4af37',
    fontSize: 18,
    fontWeight: 'bold',
    marginTop: 4,
  },
  playerRound: {
    color: '#888',
    fontSize: 12,
    marginTop: 2,
  },
  youBadge: {
    backgroundColor: '#d4af37',
    color: '#1a0a3e',
    fontSize: 10,
    fontWeight: 'bold',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 4,
    marginTop: 4,
  },
});
