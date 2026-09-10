pub mod ai;
pub mod collision;
pub mod config;
pub mod entity;
pub mod map;
pub mod math;
pub mod mine;
pub mod powerup;
pub mod round;
pub mod rules;
pub mod simulation;

pub use ai::AiController;
pub use config::GameConfig;
#[allow(unused_imports)]
pub use entity::{GameState, Player, PlayerId, Projectile, ProjectileId, Tank, TeamId};
#[allow(unused_imports)]
pub use map::{GameMap, MapSize, Wall};
pub use math::Vec2;
pub use mine::Mine;
pub use powerup::{PowerUp, PowerUpKind};
pub use round::RoundState;
#[allow(unused_imports)]
pub use rules::{GameMode, GameRules};
#[allow(unused_imports)]
pub use simulation::{GameSimulation, SimulationEvent, SimulationEventKind, TankInput};
