import {
  soundService,
  SOUND_FILES,
  AudioProvider,
  SoundType,
} from '../soundService';

describe('soundService', () => {
  let mockProvider: jest.Mocked<AudioProvider>;

  beforeEach(() => {
    soundService.reset();
    mockProvider = {
      play: jest.fn().mockResolvedValue(undefined),
      stop: jest.fn().mockResolvedValue(undefined),
      stopAll: jest.fn().mockResolvedValue(undefined),
      setVolume: jest.fn(),
      setMuted: jest.fn(),
      preload: jest.fn().mockResolvedValue(undefined),
    };
  });

  describe('setAudioProvider', () => {
    it('sets the audio provider', () => {
      soundService.setAudioProvider(mockProvider);
      expect(soundService.getAudioProvider()).toBe(mockProvider);
    });

    it('initializes provider with current volume and mute state', () => {
      soundService.setVolume(0.5);
      soundService.setMuted(true);
      soundService.setAudioProvider(mockProvider);

      expect(mockProvider.setVolume).toHaveBeenCalledWith(0.5);
      expect(mockProvider.setMuted).toHaveBeenCalledWith(true);
    });
  });

  describe('play', () => {
    beforeEach(() => {
      soundService.setAudioProvider(mockProvider);
    });

    it('plays a sound', async () => {
      await soundService.play('wheelSpin');
      expect(mockProvider.play).toHaveBeenCalledWith('wheelSpin', {
        volume: 1.0,
      });
    });

    it('plays a sound with custom config', async () => {
      await soundService.play('wheelTick', { volume: 0.5, loop: true });
      expect(mockProvider.play).toHaveBeenCalledWith('wheelTick', {
        volume: 0.5,
        loop: true,
      });
    });

    it('applies master volume to sound', async () => {
      soundService.setVolume(0.5);
      await soundService.play('correctLetter', { volume: 0.8 });
      expect(mockProvider.play).toHaveBeenCalledWith('correctLetter', {
        volume: 0.4, // 0.8 * 0.5
      });
    });

    it('does not play when muted', async () => {
      soundService.setMuted(true);
      await soundService.play('wrongLetter');
      expect(mockProvider.play).not.toHaveBeenCalled();
    });

    it('does not play when disabled', async () => {
      soundService.setEnabled(false);
      await soundService.play('bankrupt');
      expect(mockProvider.play).not.toHaveBeenCalled();
    });

    it('handles play errors gracefully', async () => {
      mockProvider.play.mockRejectedValueOnce(new Error('Audio error'));
      const consoleSpy = jest.spyOn(console, 'warn').mockImplementation();

      await soundService.play('loseTurn');

      expect(consoleSpy).toHaveBeenCalledWith(
        '[SoundService] Failed to play loseTurn:',
        expect.any(Error)
      );
      consoleSpy.mockRestore();
    });
  });

  describe('stop', () => {
    beforeEach(() => {
      soundService.setAudioProvider(mockProvider);
    });

    it('stops a specific sound', async () => {
      await soundService.stop('wheelSpin');
      expect(mockProvider.stop).toHaveBeenCalledWith('wheelSpin');
    });

    it('handles stop errors gracefully', async () => {
      mockProvider.stop.mockRejectedValueOnce(new Error('Stop error'));
      const consoleSpy = jest.spyOn(console, 'warn').mockImplementation();

      await soundService.stop('wheelSpin');

      expect(consoleSpy).toHaveBeenCalled();
      consoleSpy.mockRestore();
    });
  });

  describe('stopAll', () => {
    beforeEach(() => {
      soundService.setAudioProvider(mockProvider);
    });

    it('stops all sounds', async () => {
      await soundService.stopAll();
      expect(mockProvider.stopAll).toHaveBeenCalled();
    });

    it('handles errors gracefully', async () => {
      mockProvider.stopAll.mockRejectedValueOnce(new Error('Stop all error'));
      const consoleSpy = jest.spyOn(console, 'warn').mockImplementation();

      await soundService.stopAll();

      expect(consoleSpy).toHaveBeenCalled();
      consoleSpy.mockRestore();
    });
  });

  describe('volume', () => {
    beforeEach(() => {
      soundService.setAudioProvider(mockProvider);
    });

    it('sets and gets volume', () => {
      soundService.setVolume(0.7);
      expect(soundService.getVolume()).toBe(0.7);
      expect(mockProvider.setVolume).toHaveBeenCalledWith(0.7);
    });

    it('clamps volume to 0-1 range', () => {
      soundService.setVolume(-0.5);
      expect(soundService.getVolume()).toBe(0);

      soundService.setVolume(1.5);
      expect(soundService.getVolume()).toBe(1);
    });
  });

  describe('mute', () => {
    beforeEach(() => {
      soundService.setAudioProvider(mockProvider);
    });

    it('sets and gets mute state', () => {
      soundService.setMuted(true);
      expect(soundService.isMuted()).toBe(true);
      expect(mockProvider.setMuted).toHaveBeenCalledWith(true);

      soundService.setMuted(false);
      expect(soundService.isMuted()).toBe(false);
      expect(mockProvider.setMuted).toHaveBeenCalledWith(false);
    });
  });

  describe('enabled', () => {
    beforeEach(() => {
      soundService.setAudioProvider(mockProvider);
    });

    it('sets and gets enabled state', () => {
      soundService.setEnabled(false);
      expect(soundService.isEnabled()).toBe(false);

      soundService.setEnabled(true);
      expect(soundService.isEnabled()).toBe(true);
    });

    it('stops all sounds when disabled', () => {
      soundService.setEnabled(false);
      expect(mockProvider.stopAll).toHaveBeenCalled();
    });
  });

  describe('preload', () => {
    beforeEach(() => {
      soundService.setAudioProvider(mockProvider);
    });

    it('preloads all sounds by default', async () => {
      await soundService.preload();
      expect(mockProvider.preload).toHaveBeenCalledWith([
        'wheelSpin',
        'wheelTick',
        'wheelStop',
        'correctLetter',
        'wrongLetter',
        'bankrupt',
        'loseTurn',
        'solveFanfare',
        'buzzIn',
        'timerTick',
        'letterReveal',
        'victory',
        'buttonPress',
      ]);
    });

    it('preloads specific sounds when provided', async () => {
      await soundService.preload(['wheelSpin', 'wheelStop']);
      expect(mockProvider.preload).toHaveBeenCalledWith([
        'wheelSpin',
        'wheelStop',
      ]);
    });

    it('handles preload errors gracefully', async () => {
      mockProvider.preload.mockRejectedValueOnce(new Error('Preload error'));
      const consoleSpy = jest.spyOn(console, 'warn').mockImplementation();

      await soundService.preload();

      expect(consoleSpy).toHaveBeenCalled();
      consoleSpy.mockRestore();
    });
  });

  describe('reset', () => {
    it('resets all state to defaults', async () => {
      soundService.setAudioProvider(mockProvider);
      soundService.setVolume(0.3);
      soundService.setMuted(true);
      soundService.setEnabled(false);

      soundService.reset();

      expect(soundService.getVolume()).toBe(1.0);
      expect(soundService.isMuted()).toBe(false);
      expect(soundService.isEnabled()).toBe(true);
      // Should use noop provider after reset
      await soundService.play('wheelSpin');
      // mockProvider should not have been called since we reset
    });
  });

  describe('noop provider', () => {
    it('works without a provider set', async () => {
      // Should not throw
      await soundService.play('wheelSpin');
      await soundService.stop('wheelSpin');
      await soundService.stopAll();
      await soundService.preload();
      soundService.setVolume(0.5);
      soundService.setMuted(true);
    });
  });

  describe('SOUND_FILES', () => {
    it('has all sound types mapped', () => {
      const soundTypes: SoundType[] = [
        'wheelSpin',
        'wheelTick',
        'wheelStop',
        'correctLetter',
        'wrongLetter',
        'bankrupt',
        'loseTurn',
        'solveFanfare',
        'buzzIn',
        'timerTick',
        'letterReveal',
        'victory',
        'buttonPress',
      ];

      soundTypes.forEach((sound) => {
        expect(SOUND_FILES[sound]).toBeDefined();
        expect(SOUND_FILES[sound]).toMatch(/\.mp3$/);
      });
    });
  });
});
