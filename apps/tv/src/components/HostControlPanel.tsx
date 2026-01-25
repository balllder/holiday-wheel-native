import React, { useState, useCallback, useRef } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  ScrollView,
  TVFocusGuideView,
  Animated,
} from 'react-native';
import { useGameStore, socketService } from '@holiday-wheel/shared';

interface HostControlPanelProps {
  room: string;
  visible: boolean;
  onClose: () => void;
}

type TabType = 'game' | 'players' | 'settings';

// Custom focusable button component with visual feedback
interface FocusableButtonProps {
  onPress: () => void;
  style: object | object[];
  children: React.ReactNode;
  hasTVPreferredFocus?: boolean;
  onFocusChange?: (focused: boolean) => void;
  testID?: string;
}

function FocusableButton({
  onPress,
  style,
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
      toValue: 1.05,
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
        style={[style, isFocused && styles.buttonFocused]}
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

export function HostControlPanel({
  room,
  visible,
  onClose,
}: HostControlPanelProps): React.JSX.Element | null {
  const [activeTab, setActiveTab] = useState<TabType>('game');

  const players = useGameStore((state) => state.players);
  const phase = useGameStore((state) => state.phase);
  const activeIdx = useGameStore((state) => state.activeIdx);

  if (!visible) return null;

  const handleNewPuzzle = () => socketService.newPuzzle(room);
  const handleSpin = () => socketService.spin(room);
  const handleRevealAll = () => socketService.revealAll(room);
  const handleNewGame = () => socketService.newGame(room);
  const handleStartTossup = () => socketService.startTossup(room);
  const handleEndTossup = () => socketService.endTossup(room);
  const handleStartFinal = () => socketService.startFinal(room);
  const handleEndFinal = () => socketService.endFinal(room);
  const handleSetActivePlayer = (idx: number) => socketService.setActivePlayer(room, idx);

  const tabs: { key: TabType; label: string }[] = [
    { key: 'game', label: 'Game' },
    { key: 'players', label: 'Players' },
  ];

  const renderGameControls = () => (
    <TVFocusGuideView style={styles.controlsGrid} autoFocus>
      <FocusableButton
        style={[styles.controlButton, styles.primaryButton]}
        onPress={handleNewPuzzle}
        hasTVPreferredFocus={activeTab === 'game'}
        testID="btn-new-puzzle"
      >
        <Text style={styles.buttonIcon}>📝</Text>
        <Text style={styles.buttonLabel}>NEW PUZZLE</Text>
      </FocusableButton>

      <FocusableButton
        style={[styles.controlButton, styles.greenButton]}
        onPress={handleSpin}
        testID="btn-spin"
      >
        <Text style={styles.buttonIcon}>🎡</Text>
        <Text style={styles.buttonLabel}>SPIN</Text>
      </FocusableButton>

      <FocusableButton
        style={[styles.controlButton, styles.secondaryButton]}
        onPress={handleRevealAll}
        testID="btn-reveal-all"
      >
        <Text style={styles.buttonIcon}>👁️</Text>
        <Text style={styles.buttonLabel}>REVEAL ALL</Text>
      </FocusableButton>

      {phase === 'normal' && (
        <>
          <FocusableButton
            style={[styles.controlButton, styles.orangeButton]}
            onPress={handleStartTossup}
            testID="btn-start-tossup"
          >
            <Text style={styles.buttonIcon}>🔔</Text>
            <Text style={styles.buttonLabel}>START TOSS-UP</Text>
          </FocusableButton>

          <FocusableButton
            style={[styles.controlButton, styles.purpleButton]}
            onPress={handleStartFinal}
            testID="btn-start-final"
          >
            <Text style={styles.buttonIcon}>🏆</Text>
            <Text style={styles.buttonLabel}>START FINAL</Text>
          </FocusableButton>
        </>
      )}

      {phase === 'tossup' && (
        <FocusableButton
          style={[styles.controlButton, styles.redButton]}
          onPress={handleEndTossup}
          testID="btn-end-tossup"
        >
          <Text style={styles.buttonIcon}>⏹️</Text>
          <Text style={styles.buttonLabel}>END TOSS-UP</Text>
        </FocusableButton>
      )}

      {phase === 'final' && (
        <FocusableButton
          style={[styles.controlButton, styles.redButton]}
          onPress={handleEndFinal}
          testID="btn-end-final"
        >
          <Text style={styles.buttonIcon}>⏹️</Text>
          <Text style={styles.buttonLabel}>END FINAL</Text>
        </FocusableButton>
      )}

      <FocusableButton
        style={[styles.controlButton, styles.resetButton]}
        onPress={handleNewGame}
        testID="btn-new-game"
      >
        <Text style={styles.buttonIcon}>🔄</Text>
        <Text style={styles.buttonLabel}>NEW GAME</Text>
      </FocusableButton>
    </TVFocusGuideView>
  );

  const renderPlayersControls = () => (
    <TVFocusGuideView style={styles.playersList} autoFocus>
      <Text style={styles.sectionTitle}>Set Active Player</Text>
      {players.map((player, idx) => (
        <FocusableButton
          key={player.id}
          style={[
            styles.playerButton,
            idx === activeIdx && styles.activePlayerButton,
          ]}
          onPress={() => handleSetActivePlayer(idx)}
          hasTVPreferredFocus={activeTab === 'players' && idx === 0}
          testID={`btn-player-${idx}`}
        >
          <Text style={styles.playerName}>{player.name}</Text>
          <Text style={styles.playerScore}>${player.total}</Text>
          {idx === activeIdx && <Text style={styles.activeBadge}>ACTIVE</Text>}
        </FocusableButton>
      ))}
    </TVFocusGuideView>
  );

  return (
    <View style={styles.overlay}>
      <TVFocusGuideView style={styles.panel} autoFocus trapFocusLeft trapFocusRight>
        <View style={styles.header}>
          <Text style={styles.title}>Host Controls</Text>
          <FocusableButton
            style={styles.closeButton}
            onPress={onClose}
            testID="btn-close"
          >
            <Text style={styles.closeText}>✕</Text>
          </FocusableButton>
        </View>

        <TVFocusGuideView style={styles.tabs} trapFocusUp>
          {tabs.map((tab) => (
            <FocusableButton
              key={tab.key}
              style={[styles.tab, activeTab === tab.key && styles.activeTab]}
              onPress={() => setActiveTab(tab.key)}
              testID={`btn-tab-${tab.key}`}
            >
              <Text
                style={[
                  styles.tabText,
                  activeTab === tab.key && styles.activeTabText,
                ]}
              >
                {tab.label}
              </Text>
            </FocusableButton>
          ))}
        </TVFocusGuideView>

        <ScrollView style={styles.content}>
          {activeTab === 'game' && renderGameControls()}
          {activeTab === 'players' && renderPlayersControls()}
        </ScrollView>

        <View style={styles.phaseIndicator}>
          <Text style={styles.phaseLabel}>Current Phase:</Text>
          <Text style={styles.phaseValue}>{phase.toUpperCase()}</Text>
        </View>
      </TVFocusGuideView>
    </View>
  );
}

const styles = StyleSheet.create({
  overlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.8)',
    justifyContent: 'center',
    alignItems: 'center',
  },
  panel: {
    width: '80%',
    maxWidth: 800,
    maxHeight: '80%',
    backgroundColor: '#1a0a3e',
    borderRadius: 16,
    borderWidth: 3,
    borderColor: '#d4af37',
    overflow: 'hidden',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 20,
    borderBottomWidth: 2,
    borderBottomColor: '#333',
  },
  title: {
    color: '#d4af37',
    fontSize: 32,
    fontWeight: 'bold',
  },
  closeButton: {
    padding: 10,
  },
  closeText: {
    color: '#fff',
    fontSize: 28,
  },
  tabs: {
    flexDirection: 'row',
    borderBottomWidth: 2,
    borderBottomColor: '#333',
  },
  tab: {
    flex: 1,
    paddingVertical: 16,
    alignItems: 'center',
  },
  activeTab: {
    backgroundColor: '#2a1a4e',
    borderBottomWidth: 3,
    borderBottomColor: '#d4af37',
  },
  tabText: {
    color: '#888',
    fontSize: 20,
  },
  activeTabText: {
    color: '#d4af37',
    fontWeight: 'bold',
  },
  content: {
    padding: 20,
    maxHeight: 400,
  },
  controlsGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 16,
    justifyContent: 'center',
  },
  controlButton: {
    width: 180,
    height: 100,
    borderRadius: 12,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 12,
  },
  primaryButton: {
    backgroundColor: '#d4af37',
  },
  secondaryButton: {
    backgroundColor: '#4a4a6a',
  },
  greenButton: {
    backgroundColor: '#4caf50',
  },
  orangeButton: {
    backgroundColor: '#ff9800',
  },
  purpleButton: {
    backgroundColor: '#9c27b0',
  },
  redButton: {
    backgroundColor: '#f44336',
  },
  resetButton: {
    backgroundColor: '#607d8b',
  },
  buttonFocused: {
    borderWidth: 3,
    borderColor: '#fff',
    shadowColor: '#d4af37',
    shadowOffset: { width: 0, height: 0 },
    shadowOpacity: 0.8,
    shadowRadius: 8,
    elevation: 10,
  },
  buttonIcon: {
    fontSize: 32,
  },
  buttonLabel: {
    color: '#fff',
    fontSize: 16,
    fontWeight: 'bold',
    marginTop: 8,
    textAlign: 'center',
  },
  playersList: {
    gap: 12,
  },
  sectionTitle: {
    color: '#d4af37',
    fontSize: 24,
    fontWeight: 'bold',
    marginBottom: 12,
  },
  playerButton: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#2a1a4e',
    borderRadius: 12,
    padding: 16,
    borderWidth: 2,
    borderColor: 'transparent',
  },
  activePlayerButton: {
    borderColor: '#d4af37',
    backgroundColor: '#3a2a5e',
  },
  playerName: {
    color: '#fff',
    fontSize: 22,
    fontWeight: 'bold',
    flex: 1,
  },
  playerScore: {
    color: '#d4af37',
    fontSize: 22,
    fontWeight: 'bold',
    marginRight: 12,
  },
  activeBadge: {
    backgroundColor: '#d4af37',
    color: '#1a0a3e',
    fontSize: 14,
    fontWeight: 'bold',
    paddingHorizontal: 12,
    paddingVertical: 4,
    borderRadius: 8,
  },
  phaseIndicator: {
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 16,
    borderTopWidth: 2,
    borderTopColor: '#333',
    gap: 12,
  },
  phaseLabel: {
    color: '#888',
    fontSize: 18,
  },
  phaseValue: {
    color: '#d4af37',
    fontSize: 20,
    fontWeight: 'bold',
  },
});
