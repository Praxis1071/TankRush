# TankRush

A fast-paced tank arena game built with **Rust + GTK4** for Linux.

TankRush is designed around a clean separation between the game engine and the GTK4 interface, with deterministic simulation, local multiplayer, and LAN multiplayer as core goals.

## Features

- 🎮 Single-player and local multiplayer support
- 👥 1–4 players on one machine
- 🌐 LAN multiplayer architecture for up to 10 players
- ⚔️ Free For All and Team Battle modes
- 🗺️ Multiple arena sizes: Small, Medium, Large and Very Large
- 🧱 Maze-style arenas and wall collision
- 💥 Fast projectiles with wall ricochet
- 🎯 One-shot tank destruction
- 🔫 Up to 8 active projectiles per tank
- ⏱️ Fixed 60 Hz game simulation
- 🖥️ GTK4 desktop interface
- 🔊 Persistent music and sound-effect settings
- 🧪 Automated Rust tests, formatting, build and Clippy checks

## Tech Stack

- **Language:** Rust
- **GUI:** GTK4
- **Build system:** Cargo
- **Platform:** Linux
- **Networking:** LAN / UDP architecture
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
│   └── Match Rules
├── GTK4 UI
│   ├── Main Menu
│   ├── Match Setup
│   ├── Settings
│   └── Game View
└── LAN Multiplayer
    ├── Protocol
    ├── Host
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

TankRush is being developed incrementally:

- [x] Foundation and project structure
- [x] Core game engine
- [x] Tank movement and rotation
- [x] Projectiles and ricochet primitives
- [x] Maps and collision system
- [x] Game modes and match rules
- [x] GTK4 game interface foundation
- [x] Rendering and fixed timestep integration
- [x] LAN networking foundation
- [x] LAN lobby foundation
- [ ] Authoritative LAN gameplay
- [ ] Gameplay polish and effects
- [ ] Persistent settings completion
- [ ] Final testing and hardening
- [ ] Linux packaging and distribution

## Controls

Tank controls are configurable per player. The project is designed to support separate keyboard controls for up to four local players.

## Contributing

Issues, bug reports and improvement ideas are welcome. Please keep gameplay logic independent from GTK4 where possible and add tests for new engine or networking behavior.

## Author

**Mehmet Boztepe**

## License

TankRush is free and open-source software distributed under the [MIT License](LICENSE).

## Roadmap batch

The full gameplay roadmap implementation is being applied directly on `main`, with Rust verification in GitHub Actions after each meaningful batch.
