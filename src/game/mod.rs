pub mod collision;
pub mod config;
pub mod entity;
pub mod map;
pub mod math;
pub mod rules;
pub mod simulation;

pub use config::GameConfig;
pub use entity::{GameState, Player, PlayerId, Projectile, ProjectileId, Tank, TeamId};
pub use map::{GameMap, MapSize, Wall};
pub use math::Vec2;
pub use rules::{GameMode, GameRules};
pub use simulation::{GameSimulation, SimulationEvent, SimulationEventKind, TankInput};
