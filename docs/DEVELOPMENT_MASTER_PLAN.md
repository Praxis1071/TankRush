# TankRush — Development Master Plan

> **Purpose:** This is the long-term memory and execution plan for TankRush. Read it before substantial implementation work and update it after meaningful batches. The repository source code remains the final authority if this document becomes stale.

## 1. Product goal

TankRush will become a polished, fast, deterministic, Linux-first tank arena game built with Rust + GTK4.

The intended experience is:

- instant-to-understand arcade combat
- tanks that are visually yellow/black
- black cannon barrels and black standard projectiles
- one-hit kills
- fast bullets that ricochet from walls
- procedural maze arenas
- 1–4 local players
- LAN matches for up to 10 players
- Free For All and Team Battle
- AI single-player opponents
- optional tactical power-ups
- clean, keyboard-friendly GTK4 UI
- reliable settings and Linux/Flatpak distribution

The project should feel like a complete game, not merely a technical demo.

---

## 2. Current state snapshot — 2026-09-10

### Strong foundations already present

- Rust/Cargo project structure
- GTK4 desktop interface foundation
- fixed 60 Hz simulation
- tank movement and rotation
- projectile simulation and wall ricochet
- one-shot tank destruction
- five active projectiles per tank
- 120 px/s tank speed target
- 300 px/s projectile speed target
- 0.5 s fire cooldown target
- procedural map generation
- multiple map sizes
- wall collision
- spawn point generation
- local multiplayer foundation
- FFA and Team Battle foundations
- AI navigation/combat foundation
- power-up foundation
- UDP protocol with packet validation
- LAN lobby foundation
- authoritative host/session foundation
- player timeout/disconnect foundation
- CI for formatting, tests, build and Clippy
- initial Flatpak manifest/metadata

### Known incomplete areas

- Real end-to-end LAN gameplay UI/integration
- client-side network presentation/interpolation/reconciliation
- complete power-up implementation
- Mine weapon/entity and complete Mine lifecycle
- real audio playback system
- persistent audio/settings storage
- combat VFX/particles/feedback
- robust self-hit/friendly-fire rules
- tank-vs-tank collision and corner behavior polish
- spawn protection and combat edge cases
- gamepad support
- richer input rebinding/conflict handling
- AI difficulty levels and deeper tactics
- additional game modes
- full match/round UX
- statistics/results presentation
- polished lobby/network error UX
- accessibility pass
- release-grade Flatpak build/dependency handling
- release/versioning/screenshots/packaging validation

### Documentation drift to fix

The current README still contains claims that must be synchronized with the actual implementation. In particular, the documented projectile limit must match the five-projectile configuration, and persistent audio settings/LAN gameplay should not be described as complete until they really are.

---

## 3. Execution philosophy

### Rule A — finish end-to-end slices

Do not add a superficial version of ten systems. Complete one coherent gameplay slice before opening another large front.

### Rule B — preserve the engine

The existing game engine, map generator, collision system, AI foundation and network foundation are valuable. Extend them instead of rewriting them unless verification proves the architecture is blocking progress.

### Rule C — simulation first

Gameplay rules belong in engine modules. GTK should present state and collect input rather than become the owner of mutable game rules.

### Rule D — authoritative LAN

The host/server owns the authoritative match state. Clients send validated inputs and receive authoritative snapshots. Do not make client-side gameplay decisions authoritative.

### Rule E — verify continuously

Every meaningful batch should have:

- `cargo fmt --all -- --check`
- `cargo test --all-targets --all-features`
- `cargo build --all-targets`
- Clippy with the repository's current CI policy

When a feature affects networking, add protocol/session tests. When it affects packaging, validate the Flatpak manifest/build as far as the environment permits.

### Rule F — test by playing

The human owner is the primary gameplay tester. After each major gameplay milestone, produce a short test checklist describing exactly what should be played and what behavior should be reported.

---

# 4. Master roadmap

## Phase 0 — Baseline and engineering hygiene

**Status: mostly complete**

- [x] Core Rust/GTK4 structure
- [x] CI baseline
- [x] deterministic fixed timestep foundation
- [x] backup branch discipline
- [ ] remove stale documentation claims
- [ ] establish release/versioning convention
- [ ] establish a small regression checklist for every gameplay batch

**Exit condition:** documentation and CI accurately describe the current project.

---

## Phase 1 — Core combat completion

**Status: next priority**

Goal: make the local game mechanically complete before investing heavily in networking.

### 1.1 Power-up system

- [ ] complete Double Shot
- [ ] complete Machine Gun
- [ ] complete Laser
- [ ] complete Guided Missile
- [ ] complete Shrapnel/Frag
- [ ] add Mine
- [ ] define pickup duration/ammo rules per weapon
- [ ] define whether each weapon replaces, augments, or consumes the standard shot
- [ ] make every weapon deterministic
- [ ] add collision/rule tests for every weapon

### 1.2 Mine

Mine must be a complete gameplay entity, not just an enum:

- [ ] placement
- [ ] arming delay
- [ ] trigger radius
- [ ] owner tracking
- [ ] explosion
- [ ] one-hit destruction rule
- [ ] lifetime/cleanup
- [ ] self-hit/friendly-fire decision
- [ ] rendering state
- [ ] VFX/audio hooks
- [ ] AI awareness
- [ ] network serialization

### 1.3 Combat rules

- [ ] explicit self-hit rule
- [ ] explicit friendly-fire rule for FFA/team modes
- [ ] tank-vs-tank collision policy
- [ ] corner sliding / collision stability
- [ ] spawn safety/protection window
- [ ] projectile spawn safety
- [ ] round reset consistency
- [ ] score consistency

### Exit condition

A local 1–4 player match can be played repeatedly without obvious rule inconsistencies, and every power-up behaves correctly.

---

## Phase 2 — Combat feel: VFX + audio

**Status: not started as a complete system**

### VFX

- [ ] muzzle flash
- [ ] projectile trail where appropriate
- [ ] wall ricochet sparks
- [ ] tank explosion
- [ ] debris/impact particles
- [ ] pickup spawn effect
- [ ] pickup collection burst
- [ ] laser beam/impact effect
- [ ] missile trail/impact
- [ ] mine arming indicator
- [ ] mine explosion
- [ ] round-start feedback
- [ ] round-end feedback
- [ ] subtle screen shake

### Audio

- [ ] audio backend/manager
- [ ] music playback
- [ ] sound-effect playback
- [ ] master volume
- [ ] music volume
- [ ] SFX volume
- [ ] mute toggles
- [ ] fire sound
- [ ] ricochet sound
- [ ] explosion sound
- [ ] pickup sound
- [ ] weapon-specific sounds
- [ ] UI feedback sounds
- [ ] round/countdown sounds
- [ ] persistent audio settings

### Exit condition

The combat loop provides immediate visual and audio feedback for important actions without making the renderer or simulation architecture fragile.

---

## Phase 3 — Real LAN multiplayer v1

**Status: protocol/session foundation exists; integration incomplete**

The current UDP and authoritative-session foundations should be extended rather than replaced.

### Lobby/UI

- [ ] Host Game screen
- [ ] Join Game screen
- [ ] player name
- [ ] host address/IP field
- [ ] port field
- [ ] LAN discovery where practical
- [ ] connected-player list
- [ ] ready state
- [ ] host controls
- [ ] team selection
- [ ] team count
- [ ] map size/seed
- [ ] power-up toggle
- [ ] start match
- [ ] clear connection status
- [ ] useful error messages

### Authoritative gameplay

- [ ] authoritative player spawning
- [ ] authoritative weapon state
- [ ] authoritative power-up pickup
- [ ] authoritative round/match state
- [ ] synchronized scores
- [ ] synchronized timers/countdowns
- [ ] synchronized map/seed
- [ ] synchronized teams
- [ ] synchronized match settings

### Network quality

- [ ] snapshot interpolation
- [ ] input sequence numbers
- [ ] server tick included in snapshots
- [ ] stale/out-of-order input handling
- [ ] packet-loss tolerance
- [ ] duplicate packet handling
- [ ] ping measurement
- [ ] connection timeout UI
- [ ] graceful disconnect
- [ ] reconnect policy decision
- [ ] network diagnostics for testing

### Security/robustness

- [ ] validate player IDs against socket addresses
- [ ] validate lobby state transitions
- [ ] cap malformed input sizes
- [ ] reject impossible state transitions
- [ ] keep server authoritative for kills, pickups and scores

### Flatpak requirement

The runtime must have network sharing enabled for LAN gameplay. The manifest must be updated and tested accordingly.

### Exit condition

Two or more real machines on the same LAN can Host/Join, ready up, play a full match, finish a round, receive synchronized results, and recover cleanly from a normal disconnect.

---

## Phase 4 — Controls and accessibility

**Status: partial keyboard foundation**

- [ ] robust key rebinding
- [ ] duplicate/conflicting binding detection
- [ ] reset controls
- [ ] readable control editor
- [ ] gamepad detection
- [ ] gamepad movement
- [ ] gamepad fire
- [ ] gamepad rebinding
- [ ] optional vibration if backend permits
- [ ] keyboard navigation of menus
- [ ] visible focus states
- [ ] accessible labels/help text
- [ ] tooltips for non-obvious controls
- [ ] pause/fullscreen shortcuts

GTK4's standard controls provide accessibility support by default; custom controls should explicitly expose the information needed by assistive technology.

### Exit condition

A new player can understand and configure controls without reading source code, and the main menu/lobby/settings can be navigated with a keyboard.

---

## Phase 5 — AI 2.0

**Status: existing foundation; improve after combat rules stabilize**

Do not rewrite the AI until Mine and the weapon rules are stable.

- [ ] Easy difficulty
- [ ] Normal difficulty
- [ ] Hard difficulty
- [ ] Expert difficulty
- [ ] aim accuracy tuning
- [ ] projectile prediction
- [ ] stronger dodge behavior
- [ ] tactical retreat
- [ ] ambush behavior
- [ ] power-up selection
- [ ] Mine avoidance
- [ ] Mine placement
- [ ] weapon-specific tactics
- [ ] difficulty regression tests

### Exit condition

The AI should feel meaningfully different by difficulty, not merely faster or more accurate.

---

## Phase 6 — Match structure and game modes

**Status: basic FFA/Team Battle foundation exists**

### First priority

- [ ] Classic
- [ ] Team Battle
- [ ] Practice

### Later

- [ ] Score Attack
- [ ] Time Attack
- [ ] Elimination
- [ ] Sudden Death

### Match UX

- [ ] configurable first-to score (3/5/10)
- [ ] round countdown
- [ ] round result
- [ ] match result
- [ ] rematch
- [ ] return to lobby
- [ ] return to main menu

### Exit condition

A complete match has a clear beginning, round flow, winner, result screen and rematch path.

---

## Phase 7 — Statistics and local profile

**Status: missing**

Track locally:

- [ ] kills
- [ ] deaths
- [ ] shots fired
- [ ] shots hit
- [ ] accuracy
- [ ] ricochets
- [ ] power-ups collected
- [ ] rounds won
- [ ] matches won
- [ ] match history summary

Do not introduce accounts or cloud services for v1.0.

### Exit condition

The result screen provides useful statistics and local profile data survives application restarts.

---

## Phase 8 — Map 2.0

**Status: strong procedural foundation; polish later**

- [ ] map preview
- [ ] visible map seed
- [ ] optional custom seed
- [ ] stronger procedural validation
- [ ] map themes
- [ ] normal walls
- [ ] special/hazard walls if useful
- [ ] destructible walls only if they improve gameplay
- [ ] network synchronization of map seed/settings

Potential themes:

- Classic
- Industrial
- Cyber
- Desert
- Frozen

Do not add map complexity merely for visual variety if it harms readability or deterministic networking.

---

## Phase 9 — GTK4 UI/UX polish

**Status: foundation exists; major polish remains**

Main menu:

- [ ] Play
- [ ] Multiplayer
- [ ] Settings
- [ ] Controls
- [ ] About
- [ ] Quit

Multiplayer:

- [ ] Local
- [ ] Host
- [ ] Join

In-game:

- [ ] clean HUD
- [ ] player names
- [ ] score
- [ ] active weapon
- [ ] pause overlay
- [ ] countdown
- [ ] network status when applicable

Results:

- [ ] winner
- [ ] scoreboard
- [ ] statistics
- [ ] rematch
- [ ] menu/lobby actions

Settings:

- [ ] audio
- [ ] controls
- [ ] graphics/display
- [ ] gameplay
- [ ] network defaults

Keep the UI visually consistent with the TankRush yellow/black gameplay identity while retaining GTK4 usability.

---

## Phase 10 — Persistence

**Status: missing as a complete system**

Use a small human-readable configuration format under the user's standard Linux configuration directory.

Potential data:

- player name
- audio settings
- control bindings
- display preferences
- last selected map/settings
- gameplay preferences
- local statistics

Requirements:

- [ ] safe defaults
- [ ] load failure fallback
- [ ] atomic save where practical
- [ ] schema/version handling
- [ ] tests

---

## Phase 11 — Release and Flatpak

**Status: initial manifest only**

### Packaging

- [ ] update manifest for LAN network permission
- [ ] move toward reproducible source/dependency manifest
- [ ] ensure Cargo dependencies are available offline to Flatpak Builder
- [ ] validate AppStream metadata
- [ ] validate desktop file
- [ ] validate icon
- [ ] test sandboxed launch
- [ ] test sandboxed LAN
- [ ] test clean install/update

Flathub builds do not have general network access during the build, so Cargo dependencies must be supplied in the manifest/dependency sources for a release-quality submission.

### Release engineering

- [ ] semantic versioning decision
- [ ] changelog
- [ ] release notes
- [ ] Git tag
- [ ] GitHub Release
- [ ] screenshots
- [ ] gameplay GIF/video if useful
- [ ] installation instructions
- [ ] known issues
- [ ] final license/metadata review

### Exit condition

A fresh Linux machine can install the release, launch it, play local matches, and use LAN without undocumented manual steps.

---

# 5. Deferred ideas — do not start early

These are intentionally postponed until after v1.0 fundamentals:

- replay/demo system
- spectator mode
- online matchmaking
- public internet servers
- accounts
- cloud saves
- global leaderboard
- cross-platform ports
- Android/mobile version
- workshop/mod marketplace
- large content packs

They are not forbidden forever. They are simply not allowed to distract from the core release path.

---

# 6. Definition of TankRush v1.0

TankRush v1.0 is ready when all of the following are true:

1. Local gameplay is stable and fun.
2. Every shipped power-up has a complete implementation.
3. Combat has convincing VFX and audio.
4. LAN Host/Join works across real machines on the same LAN.
5. LAN is authoritative and robust against ordinary packet loss/disconnects.
6. Controls are configurable and gamepad support is usable if the chosen backend is stable.
7. AI has meaningful difficulty levels.
8. Matches have clear rounds, scores, winners and rematches.
9. Results include useful statistics.
10. Settings persist correctly.
11. GTK4 UI is keyboard-navigable and accessibility-conscious.
12. Flatpak can be built and launched in a sandbox with LAN support.
13. CI is green.
14. README and release metadata match actual behavior.
15. No known critical gameplay or networking bug remains.

---

# 7. Implementation protocol for future batches

For each batch:

### Before coding

1. Read this document.
2. Inspect the relevant source files on `main`.
3. Check the latest commit and CI state.
4. Verify/create a backup branch.
5. Define a small batch with a clear exit condition.

### During coding

1. Keep the diff focused.
2. Preserve existing architecture unless there is evidence it is insufficient.
3. Add tests alongside new engine/network behavior.
4. Avoid temporary scripts/workflows unless they are removed in the same completed batch.

### After coding

1. Format.
2. Test.
3. Build.
4. Clippy.
5. Run feature-specific tests.
6. Inspect the final diff.
7. Update this roadmap.
8. Update README when user-facing behavior changed.
9. Commit to `main` with a clear message.
10. Report exactly what was verified and what still requires human gameplay testing.

---

# 8. Research baseline

This plan is informed by current GTK4/Flatpak documentation and several open-source Tank Trouble implementations.

Useful observations:

- A current Tank Trouble implementation uses 120 px/s tank speed, 4.0 rad/s maximum turn rate, 300 px/s bullets, 0.5 s cooldown, five active bullets and BFS reachability validation. These values align closely with TankRush's current core configuration and therefore should be treated as a validated starting point rather than repeatedly re-tuned without gameplay evidence.
- Other Tank Trouble implementations emphasize procedural maps, power-ups, sound effects, statistics, custom controls and multiplayer room/server state.
- GTK4 provides standard accessibility interfaces for standard widgets and a structured input/shortcut system; TankRush should use those facilities rather than inventing parallel UI behavior.
- Flathub requires source builds and does not provide general network access during build, so release packaging must explicitly provide Cargo dependencies and cannot depend on downloading crates at build time.

Research references used for strategy:

- GTK4 Accessibility: https://docs.gtk.org/gtk4/section-accessibility.html
- GTK4 Input Handling: https://docs.gtk.org/gtk4/input-handling.html
- Flathub Requirements: https://docs.flathub.org/docs/for-app-authors/requirements
- Open-source Tank Trouble mechanics reference: https://github.com/fujiaze/TankTrouble2RemakeWithAINeuralNetwork
- Open-source Tank Trouble multiplayer/power-up reference: https://github.com/ethanbaker/tank-trouble
- Open-source Tank Trouble gameplay reference: https://github.com/pbsinclair42/TankTrouble

---

# 9. Decision log

## 2026-09-10 — Master roadmap established

Decision: stop treating every missing feature as an independent task. Develop TankRush through coherent end-to-end phases: core combat → combat feel → LAN → controls → AI → match structure → statistics → UI/persistence → release.

Reason: the repository already has meaningful engine and network foundations. The highest-value work is completing and integrating those systems rather than rewriting the project or adding peripheral features.

Decision: Mine is part of the core power-up completion gate.

Decision: real LAN gameplay is a major milestone and should use the existing authoritative-session architecture.

Decision: replay, spectator, public matchmaking, accounts and mobile are explicitly deferred until after v1.0.

Decision: this file plus `AGENTS.md` are persistent project memory. Future agents should read both before substantial work.

---

# 10. Current next batch

**Next implementation target: Phase 1 — Core combat completion.**

The first batch should inspect and then implement the smallest complete slice needed to finish the power-up system, beginning with the missing Mine path and any rule infrastructure it exposes. Do not start audio, LAN UI or map themes in the same batch.

The batch is complete only when:

- Mine is a real gameplay entity/behavior where required by the architecture.
- Power-up state transitions are coherent.
- Collision and destruction rules are tested.
- Existing weapons remain compatible.
- Rust formatting/tests/build/Clippy are verified.
- Human testing instructions are prepared.
- This roadmap is updated with the exact result.
