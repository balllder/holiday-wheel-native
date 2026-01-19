import React, { useEffect, useState, useRef } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  ScrollView,
  Alert,
} from 'react-native';
import Svg, { Path, Text as SvgText } from 'react-native-svg';
import { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { RouteProp } from '@react-navigation/native';
import {
  useGameStore,
  useAuthStore,
  socketService,
  configService,
  selectIsMyTurn,
  selectCanBuzz,
  VOWELS,
} from '@holiday-wheel/shared';
import type { RootStackParamList } from '../navigation/AppNavigator';

type GameScreenProps = {
  navigation: NativeStackNavigationProp<RootStackParamList, 'Game'>;
  route: RouteProp<RootStackParamList, 'Game'>;
};

export function GameScreen({ navigation, route }: GameScreenProps): React.JSX.Element {
  const { room } = route.params;
  const [letterInput, setLetterInput] = useState('');
  const [solveInput, setSolveInput] = useState('');
  const [serverUrl, setServerUrl] = useState<string | null>(null);

  const token = useAuthStore((state) => state.token);
  const connected = useGameStore((state) => state.connected);
  const phase = useGameStore((state) => state.phase);
  const puzzle = useGameStore((state) => state.puzzle);
  const revealed = useGameStore((state) => state.revealed);
  const players = useGameStore((state) => state.players);
  const activeIdx = useGameStore((state) => state.activeIdx);
  const myPlayerIdx = useGameStore((state) => state.myPlayerIdx);
  const currentWedge = useGameStore((state) => state.currentWedge);
  const wheelSlots = useGameStore((state) => state.wheelSlots);
  const lastSpinIdx = useGameStore((state) => state.lastSpinIndex);
  const isMyTurn = useGameStore(selectIsMyTurn);
  const canBuzz = useGameStore(selectCanBuzz);

  // Wheel rotation state
  const [wheelRotation, setWheelRotation] = useState(0);
  const prevSpinIdx = useRef<number | null>(null);
  const animationRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const letterInputRef = useRef<TextInput>(null);

  // Animate wheel when spin index changes
  useEffect(() => {
    if (lastSpinIdx !== null && lastSpinIdx !== prevSpinIdx.current && wheelSlots.length > 0) {
      prevSpinIdx.current = lastSpinIdx;

      // Calculate target rotation - center the slot under the pointer
      const anglePerSlot = 360 / wheelSlots.length;
      // Add half a slot width to center the slot under the pointer
      const targetSlotAngle = lastSpinIdx * anglePerSlot + anglePerSlot / 2;
      const spins = 3;
      const targetRotation = wheelRotation + spins * 360 + (360 - targetSlotAngle - (wheelRotation % 360));

      // Clear any existing animation
      if (animationRef.current) {
        clearInterval(animationRef.current);
      }

      // Animate using setInterval
      const startRotation = wheelRotation;
      const totalDelta = targetRotation - startRotation;
      const duration = 2500;
      const startTime = Date.now();

      animationRef.current = setInterval(() => {
        const elapsed = Date.now() - startTime;
        const progress = Math.min(elapsed / duration, 1);
        // Ease out cubic
        const eased = 1 - Math.pow(1 - progress, 3);
        const currentRotation = startRotation + totalDelta * eased;

        setWheelRotation(currentRotation);

        if (progress >= 1) {
          if (animationRef.current) {
            clearInterval(animationRef.current);
            animationRef.current = null;
          }
          // Focus the letter input so user is ready to guess
          letterInputRef.current?.focus();
        }
      }, 16);
    }

    return () => {
      if (animationRef.current) {
        clearInterval(animationRef.current);
      }
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [lastSpinIdx, wheelSlots.length]);

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

    const logout = useAuthStore.getState().logout;

    // Connect to socket
    socketService.connect(serverUrl, token || undefined);
    socketService.joinRoom(room);

    // Set up toast handler
    socketService.setToastCallback((msg) => {
      Alert.alert('Notice', msg);
    });

    // Set up session invalidation handler
    socketService.setSessionInvalidatedCallback(() => {
      Alert.alert(
        'Session Expired',
        'You have been logged out because your account was accessed from another device.',
        [
          {
            text: 'OK',
            onPress: () => {
              logout();
              navigation.reset({
                index: 0,
                routes: [{ name: 'Login' }],
              });
            },
          },
        ]
      );
    });

    // Cleanup on unmount
    return () => {
      socketService.disconnect();
    };
  }, [room, token, serverUrl, navigation]);

  // Join game as player when connected
  useEffect(() => {
    if (connected && myPlayerIdx === null) {
      socketService.joinGame(room);
    }
  }, [connected, myPlayerIdx, room]);

  const handleSpin = () => {
    socketService.spin(room);
  };

  const handleGuess = (inputLetter?: string) => {
    const letter = (inputLetter || letterInput).toUpperCase().trim();
    if (letter.length === 1) {
      if (VOWELS.includes(letter)) {
        socketService.buyVowel(room, letter);
      } else {
        socketService.guess(room, letter);
      }
      setLetterInput('');
    }
  };

  const handleLetterChange = (text: string) => {
    const letter = text.toUpperCase().trim();
    if (letter.length === 1 && /[A-Z]/.test(letter) && isMyTurn) {
      handleGuess(letter);
    } else {
      setLetterInput(text);
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

  // Format number as cash with commas
  const formatCash = (amount: number) => {
    return '$' + amount.toLocaleString();
  };

  // Wheel colors matching the TV show
  const WHEEL_COLORS = [
    '#c41e3a', '#0047ab', '#ff8c00', '#ffcc00', '#9932cc', '#ff1493',
    '#008b8b', '#dc143c', '#4169e1', '#ff4500', '#32cd32', '#9400d3',
    '#ff69b4', '#1e90ff', '#ffd700', '#00ced1', '#ff6347', '#8a2be2',
  ];

  const renderWheel = () => {
    if (wheelSlots.length === 0) return null;

    const size = 300;
    const radius = size / 2 - 5;
    const centerX = size / 2;
    const centerY = size / 2;

    const getWedgeLabel = (slot: any): string => {
      if (typeof slot === 'number') return `$${slot}`;
      if (typeof slot === 'string') return slot;
      if (slot?.type === 'PRIZE') return slot.name || 'PRIZE';
      if (slot?.type) return slot.type;
      return '';
    };

    const elements: React.ReactElement[] = [];

    wheelSlots.forEach((slot, idx) => {
      const anglePerSlot = 360 / wheelSlots.length;
      const startAngle = idx * anglePerSlot - 90;
      const endAngle = startAngle + anglePerSlot;
      const startRad = (startAngle * Math.PI) / 180;
      const endRad = (endAngle * Math.PI) / 180;

      const x1 = centerX + radius * Math.cos(startRad);
      const y1 = centerY + radius * Math.sin(startRad);
      const x2 = centerX + radius * Math.cos(endRad);
      const y2 = centerY + radius * Math.sin(endRad);

      const largeArc = anglePerSlot > 180 ? 1 : 0;
      const pathD = `M ${centerX} ${centerY} L ${x1} ${y1} A ${radius} ${radius} 0 ${largeArc} 1 ${x2} ${y2} Z`;

      const color = WHEEL_COLORS[idx % WHEEL_COLORS.length];
      const label = getWedgeLabel(slot);
      const isBankrupt = label === 'BANKRUPT';
      const isLoseTurn = label === 'LOSE A TURN';

      // Add wedge path
      elements.push(
        <Path
          key={`wedge-${idx}`}
          d={pathD}
          fill={isBankrupt ? '#000' : isLoseTurn ? '#fff' : color}
          stroke="#222"
          strokeWidth={1}
        />
      );

      // Add text label - position near outer edge where wedge is wider
      const midAngle = (startAngle + endAngle) / 2;
      const midRad = (midAngle * Math.PI) / 180;
      const textRadius = radius * 0.78;
      const textX = centerX + textRadius * Math.cos(midRad);
      const textY = centerY + textRadius * Math.sin(midRad);

      // Rotate text to read outward from center, flip if on left side of wheel
      // Normalize angle to 0-360 range
      const normalizedAngle = ((midAngle % 360) + 360) % 360;
      let rotation = midAngle;
      // Flip text on left side of wheel (90 to 270 degrees) so it's not upside down
      if (normalizedAngle > 90 && normalizedAngle < 270) {
        rotation = midAngle + 180;
      }

      // Use full labels with smaller font for longer text
      const displayLabel = label;
      const textColor = isBankrupt ? '#fff' : isLoseTurn ? '#000' : '#000';
      const fontSize = label.length > 8 ? 6 : label.length > 5 ? 7 : 9;

      elements.push(
        <SvgText
          key={`text-${idx}`}
          x={textX}
          y={textY}
          fill={textColor}
          fontSize={fontSize}
          fontWeight="bold"
          textAnchor="middle"
          alignmentBaseline="middle"
          transform={`rotate(${rotation}, ${textX}, ${textY})`}
        >
          {displayLabel}
        </SvgText>
      );
    });

    return (
      <View style={styles.wheelContainer}>
        <View style={styles.wheelPointer} />
        <View
          style={{
            width: size,
            height: size,
            transform: [{ rotate: `${wheelRotation}deg` }],
          }}
        >
          <Svg width={size} height={size}>
            {elements}
          </Svg>
        </View>
      </View>
    );
  };

  // Build puzzle display - split into rows like the real Wheel of Fortune board
  const renderPuzzleBoard = () => {
    const answer = (puzzle.answer || '').toUpperCase();
    const ROW_WIDTHS = [12, 14, 14, 12];
    const BOARD_ROWS = 4;

    // Word-wrap algorithm matching the web version
    const wrapWordsToLines = (text: string): string[] => {
      const words = text.split(/\s+/).filter(w => w);
      const lines: string[] = [];
      let lineIdx = 0;
      let cur = '';

      for (const w of words) {
        const maxW = ROW_WIDTHS[Math.min(lineIdx, BOARD_ROWS - 1)];
        if (!cur) {
          cur = w;
          continue;
        }
        if (cur.length + 1 + w.length <= maxW) {
          cur = cur + ' ' + w;
        } else {
          lines.push(cur);
          lineIdx++;
          cur = w;
        }
      }
      if (cur) lines.push(cur);

      // Ensure we have exactly BOARD_ROWS lines
      while (lines.length < BOARD_ROWS) lines.push('');
      while (lines.length > BOARD_ROWS) {
        const last = lines.pop()!;
        const maxW = ROW_WIDTHS[BOARD_ROWS - 1];
        lines[lines.length - 1] = (lines[lines.length - 1] + ' ' + last).slice(0, maxW);
      }

      return lines.map((l, i) => {
        const maxW = ROW_WIDTHS[i];
        return l.length > maxW ? l.slice(0, maxW) : l;
      });
    };

    // Layout lines into grid with centering
    const lines = wrapWordsToLines(answer);
    const grid: (string | null)[][] = [];

    for (let r = 0; r < BOARD_ROWS; r++) {
      const rowWidth = ROW_WIDTHS[r];
      const line = lines[r];
      const row: (string | null)[] = Array(rowWidth).fill(null);
      const padLeft = Math.max(0, Math.floor((rowWidth - line.length) / 2));
      for (let i = 0; i < line.length && padLeft + i < rowWidth; i++) {
        row[padLeft + i] = line[i];
      }
      grid.push(row);
    }

    return (
      <View style={styles.puzzleBoard}>
        <Text style={styles.category}>{puzzle.category || 'Loading...'}</Text>
        {grid.map((row, rowIdx) => (
          <View key={rowIdx} style={styles.puzzleRow}>
            {row.map((char, idx) => {
              const isLetter = char !== null && /[A-Z]/i.test(char);
              const isSpace = char === ' ';
              const isRevealed = isLetter && revealed.has(char!.toUpperCase());
              const isEmpty = char === null;

              return (
                <View
                  key={idx}
                  style={[
                    styles.puzzleCell,
                    (isEmpty || isSpace) && styles.emptyCell,
                    isLetter && !isRevealed && styles.hiddenCell,
                  ]}
                >
                  {isRevealed && <Text style={styles.puzzleLetter}>{char}</Text>}
                  {char && !isLetter && !isSpace && (
                    <Text style={styles.puzzleLetter}>{char}</Text>
                  )}
                </View>
              );
            })}
          </View>
        ))}
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
          <Text style={styles.playerScore}>{formatCash(player.total)}</Text>
          <Text style={styles.playerRound}>Round: {formatCash(player.round_bank)}</Text>
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

      {/* Wheel */}
      {renderWheel()}

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
                ref={letterInputRef}
                style={styles.letterInput}
                placeholder="Letter"
                placeholderTextColor="#888"
                value={letterInput}
                onChangeText={handleLetterChange}
                maxLength={1}
                autoCapitalize="characters"
              />
              <TouchableOpacity
                style={[styles.button, !isMyTurn && styles.disabled]}
                onPress={() => handleGuess()}
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
  wheelContainer: {
    alignItems: 'center',
    marginBottom: 16,
    position: 'relative',
  },
  wheelPointer: {
    position: 'absolute',
    top: -10,
    zIndex: 10,
    width: 0,
    height: 0,
    borderLeftWidth: 12,
    borderRightWidth: 12,
    borderTopWidth: 20,
    borderLeftColor: 'transparent',
    borderRightColor: 'transparent',
    borderTopColor: '#d4af37',
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
  puzzleRow: {
    flexDirection: 'row',
    justifyContent: 'center',
    marginVertical: 2,
  },
  puzzleCell: {
    width: 22,
    height: 28,
    marginHorizontal: 1,
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
    fontSize: 16,
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
