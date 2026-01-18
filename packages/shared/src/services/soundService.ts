/**
 * Sound Service - Manages game audio playback
 *
 * This service provides a platform-agnostic interface for playing game sounds.
 * The actual audio implementation is injected via setAudioProvider().
 *
 * Usage:
 *   // In app initialization (phone/TV)
 *   import { Audio } from 'expo-av';
 *   soundService.setAudioProvider(createExpoAudioProvider(Audio));
 *
 *   // In game code
 *   soundService.play('wheelSpin');
 */

// Sound types available in the game
export type SoundType =
  | 'wheelSpin' // Wheel starts spinning
  | 'wheelTick' // Click sound per wedge during spin
  | 'wheelStop' // Wheel stops on a wedge
  | 'correctLetter' // Letter found in puzzle
  | 'wrongLetter' // Letter not in puzzle (buzzer)
  | 'bankrupt' // Landed on bankrupt
  | 'loseTurn' // Landed on lose a turn
  | 'solveFanfare' // Puzzle solved correctly
  | 'buzzIn' // Toss-up buzz in
  | 'timerTick' // Final round countdown tick
  | 'letterReveal' // Single letter reveal animation
  | 'victory' // Game/round victory
  | 'buttonPress'; // UI button press feedback

// Sound configuration
export interface SoundConfig {
  volume?: number; // 0.0 to 1.0
  loop?: boolean;
  playbackRate?: number; // Speed multiplier
}

// Audio provider interface - implement this for each platform
export interface AudioProvider {
  play(sound: SoundType, config?: SoundConfig): Promise<void>;
  stop(sound: SoundType): Promise<void>;
  stopAll(): Promise<void>;
  setVolume(volume: number): void;
  setMuted(muted: boolean): void;
  preload(sounds: SoundType[]): Promise<void>;
}

// Default no-op provider (for testing or when audio is unavailable)
const noopProvider: AudioProvider = {
  play: async () => {},
  stop: async () => {},
  stopAll: async () => {},
  setVolume: () => {},
  setMuted: () => {},
  preload: async () => {},
};

// Sound service state
let audioProvider: AudioProvider = noopProvider;
let masterVolume = 1.0;
let isMuted = false;
let isEnabled = true;

/**
 * Sound Service API
 */
export const soundService = {
  /**
   * Set the audio provider implementation
   * Call this during app initialization
   */
  setAudioProvider(provider: AudioProvider): void {
    audioProvider = provider;
    audioProvider.setVolume(masterVolume);
    audioProvider.setMuted(isMuted);
  },

  /**
   * Get the current audio provider
   */
  getAudioProvider(): AudioProvider {
    return audioProvider;
  },

  /**
   * Play a sound effect
   */
  async play(sound: SoundType, config?: SoundConfig): Promise<void> {
    if (!isEnabled || isMuted) return;
    try {
      const adjustedConfig: SoundConfig = {
        ...config,
        volume: (config?.volume ?? 1.0) * masterVolume,
      };
      await audioProvider.play(sound, adjustedConfig);
    } catch (error) {
      // Silently fail - audio shouldn't break the game
      console.warn(`[SoundService] Failed to play ${sound}:`, error);
    }
  },

  /**
   * Stop a specific sound
   */
  async stop(sound: SoundType): Promise<void> {
    try {
      await audioProvider.stop(sound);
    } catch (error) {
      console.warn(`[SoundService] Failed to stop ${sound}:`, error);
    }
  },

  /**
   * Stop all sounds
   */
  async stopAll(): Promise<void> {
    try {
      await audioProvider.stopAll();
    } catch (error) {
      console.warn('[SoundService] Failed to stop all sounds:', error);
    }
  },

  /**
   * Set master volume (0.0 to 1.0)
   */
  setVolume(volume: number): void {
    masterVolume = Math.max(0, Math.min(1, volume));
    audioProvider.setVolume(masterVolume);
  },

  /**
   * Get current master volume
   */
  getVolume(): number {
    return masterVolume;
  },

  /**
   * Mute/unmute all sounds
   */
  setMuted(muted: boolean): void {
    isMuted = muted;
    audioProvider.setMuted(muted);
  },

  /**
   * Check if muted
   */
  isMuted(): boolean {
    return isMuted;
  },

  /**
   * Enable/disable sound system entirely
   */
  setEnabled(enabled: boolean): void {
    isEnabled = enabled;
    if (!enabled) {
      audioProvider.stopAll();
    }
  },

  /**
   * Check if sound system is enabled
   */
  isEnabled(): boolean {
    return isEnabled;
  },

  /**
   * Preload sounds for faster playback
   * Call during app load or game start
   */
  async preload(sounds?: SoundType[]): Promise<void> {
    const allSounds: SoundType[] = sounds ?? [
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
    try {
      await audioProvider.preload(allSounds);
    } catch (error) {
      console.warn('[SoundService] Failed to preload sounds:', error);
    }
  },

  /**
   * Reset to default state (for testing)
   */
  reset(): void {
    audioProvider = noopProvider;
    masterVolume = 1.0;
    isMuted = false;
    isEnabled = true;
  },
};

// Sound file mappings (to be used by audio providers)
export const SOUND_FILES: Record<SoundType, string> = {
  wheelSpin: 'wheel_spin.mp3',
  wheelTick: 'wheel_tick.mp3',
  wheelStop: 'wheel_stop.mp3',
  correctLetter: 'correct_letter.mp3',
  wrongLetter: 'wrong_letter.mp3',
  bankrupt: 'bankrupt.mp3',
  loseTurn: 'lose_turn.mp3',
  solveFanfare: 'solve_fanfare.mp3',
  buzzIn: 'buzz_in.mp3',
  timerTick: 'timer_tick.mp3',
  letterReveal: 'letter_reveal.mp3',
  victory: 'victory.mp3',
  buttonPress: 'button_press.mp3',
};
