use super::{collision, GameMap, GameState, PlayerId, Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mine {
    pub id: u32,
    pub owner: PlayerId,
    pub position: Vec2,
    pub arm_remaining: f32,
    pub remaining_lifetime: f32,
    pub trigger_radius: f32,
}

impl Mine {
    pub const ARM_TIME: f32 = 0.35;
    pub const LIFETIME: f32 = 12.0;
    pub const TRIGGER_RADIUS: f32 = 22.0;

    pub fn new(id: u32, owner: PlayerId, position: Vec2) -> Self {
        Self {
            id,
            owner,
            position,
            arm_remaining: Self::ARM_TIME,
            remaining_lifetime: Self::LIFETIME,
            trigger_radius: Self::TRIGGER_RADIUS,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.arm_remaining = (self.arm_remaining - dt).max(0.0);
        self.remaining_lifetime -= dt;
    }

    pub fn armed(&self) -> bool {
        self.arm_remaining <= 0.0
    }

    pub fn expired(&self) -> bool {
        self.remaining_lifetime <= 0.0
    }

    pub fn triggered_by(&self, state: &GameState) -> Option<PlayerId> {
        if !self.armed() {
            return None;
        }
        state
            .tanks
            .iter()
            .find(|tank| {
                tank.alive
                    && tank.player_id != self.owner
                    && (tank.position - self.position).length() <= self.trigger_radius
            })
            .map(|tank| tank.player_id)
    }

    pub fn valid_position(position: Vec2, map: &GameMap, tank_radius: f32) -> bool {
        !map.walls
            .iter()
            .any(|wall| collision::circle_intersects_wall(position, tank_radius, *wall))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{MapSize, Tank};

    #[test]
    fn mine_starts_unarmed_and_then_arms() {
        let mut mine = Mine::new(1, PlayerId(0), Vec2::new(100.0, 100.0));
        assert!(!mine.armed());
        mine.update(Mine::ARM_TIME);
        assert!(mine.armed());
    }

    #[test]
    fn armed_mine_triggers_on_enemy_not_owner() {
        let mut state = GameState::default();
        let owner = state.add_player("Owner", None).unwrap();
        let enemy = state.add_player("Enemy", None).unwrap();
        state.tanks.push(Tank::new(owner, Vec2::ZERO));
        state.tanks.push(Tank::new(enemy, Vec2::new(10.0, 0.0)));
        let mut mine = Mine::new(1, owner, Vec2::ZERO);
        mine.update(Mine::ARM_TIME);
        assert_eq!(mine.triggered_by(&state), Some(enemy));
    }

    #[test]
    fn mine_expires_after_lifetime() {
        let mut mine = Mine::new(1, PlayerId(0), Vec2::ZERO);
        mine.update(Mine::LIFETIME);
        assert!(mine.expired());
    }

    #[test]
    fn generated_map_spawn_point_is_valid_for_mine() {
        let map = GameMap::generate(MapSize::Small);
        assert!(Mine::valid_position(map.spawn_points()[0], &map, 14.0));
    }
}
