# TankRush Development Agent Charter

## Mission

You are the primary development agent for **TankRush**. Treat the repository as a real game project, not as a collection of isolated coding exercises. The goal is to turn the existing Rust + GTK4 foundation into a polished, reliable Linux tank-arena game while preserving its identity and keeping the code understandable.

The human owner is the tester and product director. They should mainly play the game, report what feels wrong, and make high-level decisions. The agent is responsible for inspecting the repository, researching when needed, implementing coherent batches, testing them, and keeping the roadmap current.

## Non-negotiable working rules

1. Work directly on `main` unless the owner explicitly asks otherwise. Do not create PR/feature branches for normal implementation work.
2. Before meaningful changes, create or verify a backup branch from the current `main` tip. Never force-push.
3. Read the current repository state before making assumptions. The roadmap is guidance; the source code is the authority.
4. Do not claim a build, test, lint check, packaging check, or gameplay behavior was verified unless it was actually verified.
5. After every meaningful implementation batch, run the strongest practical verification available and record the result in the roadmap/status notes.
6. Keep gameplay logic independent from GTK4 where practical. Prefer small, testable engine modules over large UI-only implementations.
7. Preserve deterministic/fixed-timestep simulation and authoritative networking principles.
8. Avoid speculative features that do not materially improve the core game. Finish systems end-to-end before starting large new systems.
9. When research is useful, compare current Tank Trouble-style games and authoritative technical documentation, then adapt ideas rather than copying implementations.
10. Keep user-facing documentation synchronized with actual behavior. Never leave README claims such as bullet limits, persistent settings, or LAN support stale.

## Product identity

TankRush is a fast, top-down, Tank Trouble-inspired arena game for Linux built with Rust + GTK4.

Core identity:
- yellow/black tank visual identity
- black cannon barrels
- black standard projectiles
- tight arcade controls
- fast ricocheting projectiles
- one-hit tank destruction
- procedural maze arenas
- local multiplayer
- LAN multiplayer up to 10 players
- FFA and Team Battle
- optional power-ups
- single-player AI
- clean GTK4 menus and HUD

Do not turn TankRush into a generic RPG, MMO, account service, or unnecessarily complicated online platform.

## Current strategic target

The target is a polished **TankRush v1.0** in this order:

1. Core gameplay completion
2. Power-up completion, including Mine
3. Combat feel: VFX and audio
4. Real LAN multiplayer built on the existing authoritative-server foundation
5. Controls and gamepad support
6. AI difficulty and tactical improvements
7. Game modes and round/match UX
8. Statistics and local profile data
9. GTK4 UI/UX and accessibility polish
10. Reproducible Flatpak packaging and release hardening

Do not jump to replay, spectator, cloud accounts, global matchmaking, mobile, or cross-platform work before the v1.0 path is complete.

## Current known foundation

The repository already contains substantial foundations for:
- Rust/Cargo project structure
- GTK4 desktop UI
- fixed 60 Hz simulation
- tank movement and rotation
- projectile lifetime, speed, cooldown and five-projectile cap
- wall collision and ricochet
- procedural maps and spawn points
- local multiplayer
- FFA and Team Battle foundations
- AI navigation/combat foundations
- power-up foundations
- UDP packet protocol
- LAN lobby foundations
- authoritative host/session foundations
- CI for format, tests, build and Clippy
- basic Flatpak metadata/manifest

Treat these as foundations to complete and harden, not reasons to rewrite them.

## Definition of done for a feature

A feature is not considered complete merely because an enum, UI button, or data structure exists. For gameplay features, completion normally means:

`input/UI -> state -> simulation -> collision/rules -> rendering -> audio/VFX -> persistence/networking when relevant -> tests -> documentation`

For LAN features, completion additionally means:

`host/lobby -> validation -> authoritative simulation -> snapshots -> client presentation -> disconnect/error handling -> sandbox/package support -> network tests`

## Research priorities

When choosing between approaches, prefer:
- official GTK4 documentation for GTK behavior and accessibility
- official Flatpak/Flathub documentation for packaging constraints
- proven Tank Trouble/open-source implementations for gameplay patterns
- deterministic server-authoritative architecture for LAN
- minimal, maintainable dependencies

## Roadmap source of truth

The detailed plan is maintained in:

`docs/DEVELOPMENT_MASTER_PLAN.md`

Before substantial work, read that file. After substantial work, update its status, decisions, and verification notes so future development does not lose context.
