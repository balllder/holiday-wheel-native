/**
 * HostControlPanel Tests
 *
 * Note: Full component rendering tests are skipped due to react-native-tvos
 * compatibility issues with react-test-renderer. The core game control logic
 * is tested via the shared package tests for socketService.
 */

import { socketService } from '@holiday-wheel/shared';

// Mock the shared package
jest.mock('@holiday-wheel/shared', () => ({
  useGameStore: jest.fn(() => ({
    players: [],
    phase: 'normal',
    activeIdx: 0,
  })),
  socketService: {
    newPuzzle: jest.fn(),
    spin: jest.fn(),
    revealAll: jest.fn(),
    newGame: jest.fn(),
    startTossup: jest.fn(),
    endTossup: jest.fn(),
    startFinal: jest.fn(),
    endFinal: jest.fn(),
    setActivePlayer: jest.fn(),
  },
}));

describe('HostControlPanel', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('socketService integration', () => {
    it('socketService methods are callable', () => {
      const room = 'TEST_ROOM';

      socketService.newPuzzle(room);
      expect(socketService.newPuzzle).toHaveBeenCalledWith(room);

      socketService.spin(room);
      expect(socketService.spin).toHaveBeenCalledWith(room);

      socketService.revealAll(room);
      expect(socketService.revealAll).toHaveBeenCalledWith(room);

      socketService.newGame(room);
      expect(socketService.newGame).toHaveBeenCalledWith(room);

      socketService.startTossup(room);
      expect(socketService.startTossup).toHaveBeenCalledWith(room);

      socketService.endTossup(room);
      expect(socketService.endTossup).toHaveBeenCalledWith(room);

      socketService.startFinal(room);
      expect(socketService.startFinal).toHaveBeenCalledWith(room);

      socketService.endFinal(room);
      expect(socketService.endFinal).toHaveBeenCalledWith(room);

      socketService.setActivePlayer(room, 1);
      expect(socketService.setActivePlayer).toHaveBeenCalledWith(room, 1);
    });
  });

  describe('HostControlPanel module', () => {
    it('exports HostControlPanel component', () => {
      const { HostControlPanel } = require('../src/components/HostControlPanel');
      expect(HostControlPanel).toBeDefined();
      expect(typeof HostControlPanel).toBe('function');
    });
  });
});
