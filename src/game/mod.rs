pub mod config;
pub mod entity;
pub mod math;
pub mod rules;

pub use config::GameConfig;
pub use entity::{GameState, Player, PlayerId, Projectile, ProjectileId, Tank, TeamId};
pub use math::Vec2;
pub use rules::GameRules;
