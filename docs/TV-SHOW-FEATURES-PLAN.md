# Holiday Wheel - TV Show Features & UI Enhancement Plan

## Overview

Transform Holiday Wheel into a more authentic Wheel of Fortune experience with sound effects, animations, modern UI, and additional game mechanics. Features are organized from **quickest wins** to **most complex**.

---

## Priority Tiers

| Tier | Focus | Effort | Impact |
|------|-------|--------|--------|
| **Tier 1** | Quick Wins | 1-2 hours each | High |
| **Tier 2** | Visual Polish | 2-4 hours each | High |
| **Tier 3** | Game Mechanics | 4-8 hours each | Medium-High |
| **Tier 4** | Major Features | 1-2 days each | High |

---

## Tier 1: Quick Wins (Highest Impact, Lowest Effort)

### 1.1 Sound Effects Library
**Effort:** 2 hours | **Impact:** Very High

Add audio feedback for game events using `expo-av` or `react-native-sound`.

| Sound | Trigger |
|-------|---------|
| Wheel spin | On spin start |
| Wheel tick | During spin (click per wedge) |
| Wheel stop | On spin complete |
| Correct letter | Letter found in puzzle |
| Wrong letter | Letter not in puzzle |
| Bankrupt | Land on bankrupt |
| Lose a turn | Land on lose a turn |
| Solve fanfare | Puzzle solved |
| Buzz in | Toss-up buzz |
| Timer tick | Final round countdown |

**Files to modify:**
- `packages/shared/src/services/soundService.ts` (NEW)
- `packages/shared/src/stores/gameStore.ts` (add sound triggers)
- `apps/phone/src/screens/GameScreen.tsx`
- `apps/tv/src/screens/TVGameScreen.tsx`

**Assets needed:** 10-12 short audio files (can use free game sound libraries)

---

### 1.2 Score Change Animations
**Effort:** 1-2 hours | **Impact:** High

Animate score updates with floating "+$500" text.

```typescript
// Floating score animation component
<AnimatedScoreChange value={+500} position="above-player" />
```

**Implementation:**
- Use `react-native-reanimated` for smooth animations
- Fade in, float up, fade out over 1.5 seconds
- Gold color for gains, red for losses (bankrupt)

**Files to modify:**
- `packages/shared/src/components/AnimatedScore.tsx` (NEW)
- Player score displays in GameScreen/TVGameScreen

---

### 1.3 Button Press Feedback
**Effort:** 1 hour | **Impact:** Medium

Enhance button interactions with scale + glow effects.

```typescript
// Pressable with animation
<AnimatedButton
  onPress={handleSpin}
  scaleOnPress={0.95}
  glowColor="#d4af37"
/>
```

**Implementation:**
- Scale down to 95% on press
- Optional glow/shadow effect
- Haptic feedback on mobile

---

### 1.4 Wedge Landing Highlight
**Effort:** 1 hour | **Impact:** High

When wheel stops, highlight/pulse the winning wedge.

**Implementation:**
- Flash the wedge 2-3 times
- Add glow effect around winning segment
- Sync with "wheel stop" sound

---

## Tier 2: Visual Polish (Medium Effort, High Impact)

### 2.1 Letter Reveal Animation
**Effort:** 3-4 hours | **Impact:** Very High

Animate letters flipping/popping into view like the TV show.

**Options:**
1. **Flip animation** - Letter tile rotates 180° on Y-axis
2. **Pop animation** - Scale from 0 to 1.1 to 1.0 with bounce
3. **Slide-in** - Letter slides down from top of cell

**Implementation:**
- Stagger reveals (100-200ms apart)
- Add subtle sound per letter reveal
- White flash/glow on reveal

**Files to modify:**
- `packages/shared/src/components/PuzzleBoard.tsx`
- Add letter state tracking (hidden → revealing → revealed)

---

### 2.2 Modern UI Theme Overhaul
**Effort:** 4-6 hours | **Impact:** High

Refresh the visual design with modern styling.

**Theme Updates:**

| Element | Current | New |
|---------|---------|-----|
| Background | Solid `#0d0628` | Gradient purple-to-blue |
| Cards | Solid `#1a0a3e` | Glassmorphism (blur + transparency) |
| Buttons | Flat colors | Gradients + shadows + glow |
| Text | Plain white | Subtle shadows, better hierarchy |
| Borders | Solid lines | Gradient borders or none |

**Color Palette Refresh:**
```typescript
const theme = {
  background: 'linear-gradient(180deg, #1a0a3e 0%, #0d0628 100%)',
  card: 'rgba(26, 10, 62, 0.8)',
  cardBlur: 10,
  gold: '#d4af37',
  goldGlow: 'rgba(212, 175, 55, 0.4)',
  accent: '#6366f1', // Indigo accent
  success: '#22c55e',
  danger: '#ef4444',
};
```

**Files to modify:**
- `packages/shared/src/constants/index.ts` (theme constants)
- All screen components (apply new styles)

---

### 2.3 Celebration Effects (Confetti)
**Effort:** 2-3 hours | **Impact:** High

Add confetti/particles when puzzle is solved.

**Options:**
1. `react-native-confetti-cannon` - Simple, works well
2. `lottie-react-native` - Use pre-made Lottie animations
3. Custom SVG particles with reanimated

**Triggers:**
- Puzzle solved correctly
- Round won
- Game won (extra dramatic)

---

### 2.4 Phase Transition Animations
**Effort:** 2 hours | **Impact:** Medium

Animate transitions between game phases.

| Transition | Animation |
|------------|-----------|
| Normal → Toss-up | Slide-in banner, dramatic sound |
| Normal → Final | Fade to black, reveal new board |
| Puzzle solved | Flash + confetti + fanfare |

---

## Tier 3: Game Mechanics (Higher Effort)

### 3.1 Mystery Wedge
**Effort:** 4-6 hours | **Impact:** High

Add the exciting mystery wedge from the TV show.

**How it works:**
1. Player lands on Mystery wedge
2. Must guess correct consonant first
3. Then chooses: Keep $1,000 OR flip for $10,000/Bankrupt

**Backend changes:**
- Add `Mystery` to `WedgeValue` enum
- Add `flip_mystery` socket event
- Track mystery wedge state

**Frontend changes:**
- Show mystery choice modal
- Dramatic reveal animation

---

### 3.2 Express Wedge
**Effort:** 4-6 hours | **Impact:** Medium-High

Allow rapid-fire consonant guessing without spinning.

**How it works:**
1. Player lands on Express wedge
2. Can keep guessing consonants at $1,000 each
3. Buy vowels for $250
4. One wrong guess = Bankrupt

**Implementation:**
- Add `Express` to `WedgeValue` enum
- Add express mode state tracking
- Special UI showing express mode active

---

### 3.3 Wild Card Token
**Effort:** 3-4 hours | **Impact:** Medium

Collectible token that acts as extra consonant.

**How it works:**
1. Win Wild Card wedge by guessing correctly
2. Card saved to player inventory
3. Can use later to call any consonant (even if already called)

**Implementation:**
- Track wild cards per player
- Add "Use Wild Card" button
- Show wild card count in player panel

---

### 3.4 Multi-Round Game Structure
**Effort:** 6-8 hours | **Impact:** High

Proper round structure like the TV show.

**Structure:**
- Round 1: Normal puzzle
- Round 2: Normal puzzle
- Round 3: Normal puzzle (higher values)
- Bonus Round: Final puzzle with jackpot

**Implementation:**
- Track round number in game state
- Different wheel configurations per round
- Automatic round advancement
- Cumulative scoring display

---

### 3.5 Enhanced Toss-Up Mode
**Effort:** 4-6 hours | **Impact:** Medium

More TV-authentic toss-up puzzles.

**Improvements:**
- Letters reveal one at a time (200-500ms intervals)
- Triple Toss-Up (3 consecutive puzzles)
- Increasing values ($1000, $2000, $3000)
- Speed Round variant

---

## Tier 4: Major Features (Highest Effort)

### 4.1 3D Wheel with Physics
**Effort:** 1-2 days | **Impact:** Very High

Replace SVG wheel with 3D rendered wheel.

**Options:**
1. `three.js` / `react-three-fiber` - Full 3D
2. `react-native-skia` - 2.5D with shaders
3. Enhanced SVG with perspective transforms

**Features:**
- Realistic spin physics (acceleration, deceleration)
- Wheel tilt perspective
- Reflections and lighting
- Pointer flapper animation

---

### 4.2 Vanna-Style Letter Touch
**Effort:** 1-2 days | **Impact:** High

Animated character that "touches" letters to reveal them.

**Implementation:**
- Simple character sprite/animation
- Moves to each letter position
- Touch triggers reveal animation
- Could be optional/toggle

---

### 4.3 Full Audio System
**Effort:** 1 day | **Impact:** Very High

Complete audio experience with music and effects.

**Components:**
- Background music (lobby, game, final round)
- Sound effect layers
- Volume controls
- Mute toggle
- Audio ducking (lower music during effects)

---

### 4.4 Player Podiums/Avatars
**Effort:** 1 day | **Impact:** Medium

Visual player representations like TV show.

**Features:**
- Player avatar selection
- Podium-style score displays
- Name plates
- Winner spotlight

---

## Implementation Order (Recommended)

### Sprint 1: Core Polish
1. Sound Effects Library (1.1)
2. Score Change Animations (1.2)
3. Button Press Feedback (1.3)
4. Wedge Landing Highlight (1.4)

### Sprint 2: Visual Upgrade
5. Letter Reveal Animation (2.1)
6. Celebration Effects (2.3)
7. Modern UI Theme (2.2)
8. Phase Transitions (2.4)

### Sprint 3: New Mechanics
9. Mystery Wedge (3.1)
10. Express Wedge (3.2)
11. Wild Card Token (3.3)

### Sprint 4: Advanced Features
12. Multi-Round Structure (3.4)
13. Enhanced Toss-Up (3.5)
14. 3D Wheel (4.1)
15. Full Audio System (4.3)
16. Player Podiums (4.4)
17. Vanna Character (4.2)

---

## Files to Create

| File | Purpose |
|------|---------|
| `packages/shared/src/services/soundService.ts` | Audio playback |
| `packages/shared/src/services/animationService.ts` | Animation orchestration |
| `packages/shared/src/components/AnimatedScore.tsx` | Score animations |
| `packages/shared/src/components/AnimatedButton.tsx` | Button effects |
| `packages/shared/src/components/Confetti.tsx` | Celebration effects |
| `packages/shared/src/components/LetterCell.tsx` | Animated letter reveal |
| `packages/shared/src/constants/theme.ts` | New theme system |
| `packages/shared/src/constants/sounds.ts` | Sound file mappings |

## Files to Modify

| File | Changes |
|------|---------|
| `packages/shared/src/components/PuzzleBoard.tsx` | Letter animations |
| `packages/shared/src/components/AnimatedWheel.tsx` | Landing highlight, sounds |
| `packages/shared/src/stores/gameStore.ts` | Animation/sound triggers |
| `apps/phone/src/screens/GameScreen.tsx` | New theme, animations |
| `apps/tv/src/screens/TVGameScreen.tsx` | New theme, animations |
| `apps/backend-rust/src/game/wheel.rs` | Mystery/Express wedges |
| `apps/backend-rust/src/game/state.rs` | New game mechanics |
| `apps/backend-rust/src/game/handlers.rs` | New socket events |

---

## Dependencies to Add

```json
// packages/shared/package.json
{
  "expo-av": "^14.0.0",           // Audio playback
  "react-native-reanimated": "^3.x", // Already likely present
  "react-native-confetti-cannon": "^1.5.0"  // Confetti effects
}
```

---

## Verification Plan

### Sound Effects
- [ ] Play each sound effect manually
- [ ] Verify sounds trigger on correct game events
- [ ] Test volume controls

### Animations
- [ ] Letter reveal looks smooth on phone and TV
- [ ] Score animations don't overlap
- [ ] Confetti doesn't cause performance issues

### Game Mechanics
- [ ] Mystery wedge flow works correctly
- [ ] Express mode tracks state properly
- [ ] Wild cards persist across turns

### Theme
- [ ] UI looks good on both phone and TV
- [ ] No accessibility issues (contrast, readability)
- [ ] Dark mode only (no light mode needed)
