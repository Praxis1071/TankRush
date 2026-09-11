# TankRush

A fast-paced tank arena game built with **Rust + GTK4** for Linux.

TankRush is designed around a clean separation between the game engine and the GTK4 interface, with deterministic simulation, local multiplayer, and LAN multiplayer as core goals.

## Features

- 🎮 Single-player and local multiplayer support
- 👥 1–4 players on one machine
- 🌐 LAN multiplayer foundation for up to 10 players
- ⚔️ Free For All and Team Battle rule foundations
- 🗺️ Multiple arena sizes: Small, Medium, Large and Very Large
- 🧱 Seeded maze-style arenas and wall collision
- 💥 Fast projectiles with wall ricochet
- 🎯 One-shot tank destruction
- 🔫 Maximum 5 active projectiles per tank
- ⚡ Power-up weapon system with Double Shot, Machine Gun, Laser, Guided Missile, Shrapnel and Mine foundations
- 💣 Placeable mines with arming time, trigger radius and lifetime
- 🤖 Local AI opponent with navigation and combat tactics
- ⏱️ Fixed 60 Hz game simulation
- 🖥️ GTK4 desktop interface
- 🔊 Audio settings UI foundation
- 🧪 Automated Rust tests, formatting, build and Clippy checks

## Tech Stack

- **Language:** Rust
- **GUI:** GTK4
- **Build system:** Cargo
- **Platform:** Linux
- **Networking:** UDP / LAN architecture
- **License:** MIT

## Project Architecture

The project is intentionally divided into independent layers:

```text
TankRush
├── Game Engine
│   ├── Entities
│   ├── Simulation
│   ├── Collision
│   ├── Maps
│   ├── Match Rules
│   ├── Power-ups / Mines
│   └── AI
├── GTK4 UI
│   ├── Main Menu
│   ├── Match Setup
│   ├── Settings
│   └── Game View
└── LAN Multiplayer
    ├── Protocol
    ├── Host-authoritative session foundation
    ├── Lobby
    └── Network Gameplay
```

The engine does not depend on GTK4. This keeps gameplay logic testable and makes the networking layer independent from the graphical interface.

## Building

Install Rust and the GTK4 development packages for your Linux distribution, then:

```bash
git clone https://github.com/Praxis1071/TankRush.git
cd TankRush
cargo build
```

## Running

```bash
cargo run
```

## Testing

Run the complete test suite with:

```bash
cargo test
```

For formatting and static analysis:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

GitHub Actions also runs the project's formatting, test, build and Clippy checks.

## Development Roadmap

TankRush is being developed incrementally toward v1.0:

- [x] Foundation and project structure
- [x] Core game engine
- [x] Tank movement and rotation
- [x] Projectiles and ricochet primitives
- [x] Maps and collision system
- [x] Game modes and match-rule foundations
- [x] GTK4 game interface foundation
- [x] Rendering and fixed timestep integration
- [x] LAN protocol/session foundation
- [x] LAN lobby foundation
- [x] Power-up and Mine gameplay foundations
- [ ] Finish and harden all power-up behaviors
- [ ] Combat VFX and real audio
- [ ] Authoritative LAN gameplay on real machines
- [ ] Controls/accessibility and gamepad support
- [ ] AI difficulty improvements
- [ ] Complete match UX, statistics and persistence
- [ ] Flatpak packaging and release validation
- [ ] Final testing and hardening

## Controls

Tank controls are configurable per player. The current local game supports separate keyboard controls for up to four players.

## Contributing

Issues, bug reports and improvement ideas are welcome. Please keep gameplay logic independent from GTK4 where possible and add tests for new engine or networking behavior.

## Author

**Mehmet Boztepe**

## License

TankRush is free and open-source software distributed under the [MIT License](LICENSE).

## Development process

The v1.0 roadmap is implemented directly on `main` in meaningful, backed-up batches. Rust verification is performed in GitHub Actions after changes.
