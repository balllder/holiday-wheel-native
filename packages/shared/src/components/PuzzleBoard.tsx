import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { View, Text, StyleSheet, ViewStyle } from 'react-native';
import { LetterCell, LetterState } from './LetterCell';
import { ROW_WIDTHS } from '../constants';

export interface PuzzleBoardProps {
  /** The puzzle answer text */
  answer: string;
  /** The puzzle category */
  category: string;
  /** Set of revealed letters (uppercase) */
  revealed: Set<string>;
  /** Size variant for different displays */
  size?: 'phone' | 'tv';
  /** Animation type for letter reveal */
  animationType?: 'pop' | 'flip' | 'fade';
  /** Base duration for reveal animation in ms */
  animationDuration?: number;
  /** Delay between each letter reveal in ms (for staggering) */
  staggerDelay?: number;
  /** Callback when a letter starts revealing */
  onLetterRevealStart?: (letter: string, index: number) => void;
  /** Callback when a letter finishes revealing */
  onLetterRevealComplete?: (letter: string, index: number) => void;
  /** Callback when all letters in a batch finish revealing */
  onAllRevealsComplete?: () => void;
  /** Custom container style */
  style?: ViewStyle;
  /** Test ID for testing */
  testID?: string;
}

interface CellData {
  char: string | null;
  rowIdx: number;
  colIdx: number;
  globalIdx: number;
}

const BOARD_ROWS = 4;

/**
 * PuzzleBoard - Wheel of Fortune style puzzle display with animated letter reveals
 *
 * Features:
 * - 4-row layout matching the TV show board
 * - Word wrapping to fit row widths
 * - Animated letter reveals with staggering
 * - Phone and TV size variants
 *
 * @example
 * <PuzzleBoard
 *   answer="WHEEL OF FORTUNE"
 *   category="TV SHOW"
 *   revealed={revealedSet}
 *   animationType="pop"
 *   staggerDelay={100}
 * />
 */
export function PuzzleBoard({
  answer,
  category,
  revealed,
  size = 'phone',
  animationType = 'pop',
  animationDuration = 300,
  staggerDelay = 100,
  onLetterRevealStart,
  onLetterRevealComplete,
  onAllRevealsComplete,
  style,
  testID,
}: PuzzleBoardProps): React.JSX.Element {
  // Track letter states for animation
  const [letterStates, setLetterStates] = useState<Map<number, LetterState>>(new Map());
  const prevRevealed = useRef<Set<string>>(new Set());
  const revealingCount = useRef(0);

  // Layout puzzle into rows
  const grid = useMemo(() => layoutPuzzle(answer.toUpperCase()), [answer]);

  // Flatten grid for easier indexing
  const flatCells = useMemo(() => {
    const cells: CellData[] = [];
    grid.forEach((row, rowIdx) => {
      row.forEach((char, colIdx) => {
        cells.push({ char, rowIdx, colIdx, globalIdx: cells.length });
      });
    });
    return cells;
  }, [grid]);

  // Detect newly revealed letters and trigger animations
  useEffect(() => {
    const newlyRevealed: { letter: string; indices: number[] }[] = [];

    // Find letters that are newly revealed
    revealed.forEach((letter) => {
      if (!prevRevealed.current.has(letter)) {
        // Find all cells with this letter
        const indices: number[] = [];
        flatCells.forEach((cell, idx) => {
          if (cell.char?.toUpperCase() === letter) {
            indices.push(idx);
          }
        });
        if (indices.length > 0) {
          newlyRevealed.push({ letter, indices });
        }
      }
    });

    // Update previous revealed set
    prevRevealed.current = new Set(revealed);

    if (newlyRevealed.length === 0) return;

    // Start revealing animations with stagger
    let delay = 0;
    const allIndices: number[] = [];

    newlyRevealed.forEach(({ letter, indices }) => {
      indices.forEach((idx, i) => {
        allIndices.push(idx);
        const cellDelay = delay + i * staggerDelay;

        // Set to revealing state with delay
        setTimeout(() => {
          onLetterRevealStart?.(letter, idx);
          setLetterStates((prev) => {
            const next = new Map(prev);
            next.set(idx, 'revealing');
            return next;
          });
          revealingCount.current++;
        }, cellDelay);
      });
      delay += indices.length * staggerDelay;
    });
  }, [revealed, flatCells, staggerDelay, onLetterRevealStart]);

  // Handle individual letter animation complete
  const handleAnimationComplete = useCallback(
    (idx: number, letter: string) => {
      setLetterStates((prev) => {
        const next = new Map(prev);
        next.set(idx, 'revealed');
        return next;
      });

      onLetterRevealComplete?.(letter, idx);

      revealingCount.current--;
      if (revealingCount.current === 0) {
        onAllRevealsComplete?.();
      }
    },
    [onLetterRevealComplete, onAllRevealsComplete]
  );

  // Determine letter state for a cell
  const getLetterState = useCallback(
    (cell: CellData, idx: number): LetterState => {
      const char = cell.char;

      // Empty or space
      if (char === null) return 'empty';
      if (char === ' ') return 'space';

      // Non-letter characters (punctuation) are always shown
      if (!/[A-Z]/i.test(char)) return 'revealed';

      // Check animation state first
      const animState = letterStates.get(idx);
      if (animState) return animState;

      // Check if letter is in revealed set
      if (revealed.has(char.toUpperCase())) {
        return 'revealed';
      }

      return 'hidden';
    },
    [revealed, letterStates]
  );

  const cellSize = size === 'tv' ? 'large' : 'small';
  const categoryStyle = size === 'tv' ? styles.categoryTV : styles.categoryPhone;
  const rowStyle = size === 'tv' ? styles.puzzleRowTV : styles.puzzleRowPhone;
  const boardStyle = size === 'tv' ? styles.puzzleBoardTV : styles.puzzleBoardPhone;

  return (
    <View style={[boardStyle, style]} testID={testID}>
      <Text style={categoryStyle}>{category || 'Loading...'}</Text>
      {grid.map((row, rowIdx) => (
        <View key={rowIdx} style={rowStyle}>
          {row.map((char, colIdx) => {
            const _idx = rowIdx * ROW_WIDTHS[rowIdx] + colIdx;
            // Recalculate the global index correctly
            let globalIdx = 0;
            for (let r = 0; r < rowIdx; r++) {
              globalIdx += ROW_WIDTHS[r];
            }
            globalIdx += colIdx;

            const cell = flatCells[globalIdx] || { char, rowIdx, colIdx, globalIdx };
            const state = getLetterState(cell, globalIdx);

            return (
              <LetterCell
                key={`${rowIdx}-${colIdx}`}
                char={char}
                state={state}
                size={cellSize}
                animationType={animationType}
                animationDuration={animationDuration}
                onAnimationComplete={
                  state === 'revealing'
                    ? () => handleAnimationComplete(globalIdx, char || '')
                    : undefined
                }
                testID={`cell-${rowIdx}-${colIdx}`}
              />
            );
          })}
        </View>
      ))}
    </View>
  );
}

/**
 * Layout puzzle text into the 4-row board format
 */
function layoutPuzzle(text: string): (string | null)[][] {
  const words = text.split(/\s+/).filter((w) => w);
  const lines: string[] = [];
  let lineIdx = 0;
  let cur = '';

  // Word wrap algorithm
  for (const word of words) {
    const maxW = ROW_WIDTHS[Math.min(lineIdx, BOARD_ROWS - 1)];
    if (!cur) {
      cur = word;
      continue;
    }
    if (cur.length + 1 + word.length <= maxW) {
      cur = cur + ' ' + word;
    } else {
      lines.push(cur);
      lineIdx++;
      cur = word;
    }
  }
  if (cur) lines.push(cur);

  // Ensure exactly BOARD_ROWS lines
  while (lines.length < BOARD_ROWS) lines.push('');
  while (lines.length > BOARD_ROWS) {
    const last = lines.pop()!;
    const maxW = ROW_WIDTHS[BOARD_ROWS - 1];
    lines[lines.length - 1] = (lines[lines.length - 1] + ' ' + last).slice(0, maxW);
  }

  // Build grid with centering
  const grid: (string | null)[][] = [];
  for (let r = 0; r < BOARD_ROWS; r++) {
    const rowWidth = ROW_WIDTHS[r];
    const line = lines[r].length > rowWidth ? lines[r].slice(0, rowWidth) : lines[r];
    const row: (string | null)[] = Array(rowWidth).fill(null);
    const padLeft = Math.max(0, Math.floor((rowWidth - line.length) / 2));
    for (let i = 0; i < line.length && padLeft + i < rowWidth; i++) {
      row[padLeft + i] = line[i];
    }
    grid.push(row);
  }

  return grid;
}

const styles = StyleSheet.create({
  // Phone styles
  puzzleBoardPhone: {
    backgroundColor: '#1a5cb8',
    borderRadius: 8,
    padding: 16,
    borderWidth: 3,
    borderColor: '#d4af37',
  },
  categoryPhone: {
    color: '#ffd700',
    fontSize: 18,
    fontWeight: 'bold',
    textAlign: 'center',
    marginBottom: 16,
  },
  puzzleRowPhone: {
    flexDirection: 'row',
    justifyContent: 'center',
    marginVertical: 1,
  },

  // TV styles
  puzzleBoardTV: {
    backgroundColor: '#1a5cb8',
    borderRadius: 16,
    padding: 24,
    borderWidth: 4,
    borderColor: '#d4af37',
  },
  categoryTV: {
    color: '#ffd700',
    fontSize: 42,
    fontWeight: 'bold',
    textAlign: 'center',
    marginBottom: 32,
  },
  puzzleRowTV: {
    flexDirection: 'row',
    justifyContent: 'center',
  },
});

export default PuzzleBoard;
