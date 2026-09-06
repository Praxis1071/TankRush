pub mod collision;
pub mod config;
pub mod entity;
pub mod map;
pub mod math;
pub mod rules;
pub mod simulation;

pub use config::GameConfig;
#[allow(unused_imports)]
pub use entity::{GameState, Player, PlayerId, Projectile, ProjectileId, Tank, TeamId};
#[allow(unused_imports)]
pub use map::{GameMap, MapSize, Wall};
pub use math::Vec2;
#[allow(unused_imports)]
pub use rules::{GameMode, GameRules};
#[allow(unused_imports)]
pub use simulation::{GameSimulation, SimulationEvent, SimulationEventKind, TankInput};
