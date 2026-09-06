use super::{GameConfig, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectileId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TeamId(pub u8);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub team: Option<TeamId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tank {
    pub player_id: PlayerId,
    pub position: Vec2,
    pub rotation_radians: f32,
    pub alive: bool,
}

impl Tank {
    pub fn new(player_id: PlayerId, position: Vec2) -> Self {
        Self {
            player_id,
            position,
            rotation_radians: 0.0,
            alive: true,
        }
    }

    pub fn direction(&self) -> Vec2 {
        Vec2::new(self.rotation_radians.cos(), self.rotation_radians.sin())
    }

    pub fn move_forward(&mut self, seconds: f32, config: &GameConfig) {
        self.move_signed(1.0, seconds, config);
    }

    pub fn move_backward(&mut self, seconds: f32, config: &GameConfig) {
        self.move_signed(-1.0, seconds, config);
    }

    fn move_signed(&mut self, direction: f32, seconds: f32, config: &GameConfig) {
        if self.alive {
            self.position =
                self.position + self.direction() * (config.tank_speed * direction * seconds);
        }
    }

    pub fn turn_left(&mut self, seconds: f32, config: &GameConfig) {
        if self.alive {
            self.rotation_radians -= config.tank_turn_speed_radians * seconds;
        }
    }

    pub fn turn_right(&mut self, seconds: f32, config: &GameConfig) {
        if self.alive {
            self.rotation_radians += config.tank_turn_speed_radians * seconds;
        }
    }

    pub fn destroy(&mut self) {
        self.alive = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projectile {
    pub id: ProjectileId,
    pub owner: PlayerId,
    pub position: Vec2,
    pub velocity: Vec2,
    pub remaining_lifetime: f32,
}

impl Projectile {
    pub fn new(
        id: ProjectileId,
        owner: PlayerId,
        position: Vec2,
        direction: Vec2,
        config: &GameConfig,
    ) -> Self {
        Self {
            id,
            owner,
            position,
            velocity: direction.normalized() * config.projectile_speed,
            remaining_lifetime: config.projectile_lifetime_seconds,
        }
    }

    pub fn update(&mut self, seconds: f32) {
        self.position = self.position + self.velocity * seconds;
        self.remaining_lifetime -= seconds;
    }

    pub fn bounce(&mut self, normal: Vec2) {
        self.velocity = self.velocity.reflect(normal.normalized());
    }

    pub fn expired(&self) -> bool {
        self.remaining_lifetime <= 0.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct GameState {
    pub players: Vec<Player>,
    pub tanks: Vec<Tank>,
    pub projectiles: Vec<Projectile>,
    next_projectile_id: u32,
}

impl GameState {
    pub fn add_player(
        &mut self,
        name: impl Into<String>,
        team: Option<TeamId>,
    ) -> Option<PlayerId> {
        if self.players.len() >= 10 {
            return None;
        }
        let id = PlayerId(self.players.len() as u8);
        self.players.push(Player {
            id,
            name: name.into(),
            team,
        });
        Some(id)
    }

    pub fn add_tank(&mut self, player_id: PlayerId, position: Vec2) {
        self.tanks.push(Tank::new(player_id, position));
    }

    pub fn fire(&mut self, player_id: PlayerId, config: &GameConfig) -> Option<ProjectileId> {
        let active_count = self
            .projectiles
            .iter()
            .filter(|p| p.owner == player_id)
            .count();
        if active_count >= config.max_active_projectiles_per_tank {
            return None;
        }
        let tank = self
            .tanks
            .iter()
            .find(|t| t.player_id == player_id && t.alive)?;
        let id = ProjectileId(self.next_projectile_id);
        self.next_projectile_id = self.next_projectile_id.wrapping_add(1);
        let spawn_position = tank.position
            + tank.direction() * (config.tank_radius + config.projectile_radius + 1.0);
        self.projectiles.push(Projectile::new(
            id,
            player_id,
            spawn_position,
            tank.direction(),
            config,
        ));
        Some(id)
    }

    pub fn update_projectiles(&mut self, seconds: f32) {
        for projectile in &mut self.projectiles {
            projectile.update(seconds);
        }
        self.projectiles.retain(|projectile| !projectile.expired());
    }

    pub fn destroy_tank(&mut self, player_id: PlayerId) -> bool {
        if let Some(tank) = self
            .tanks
            .iter_mut()
            .find(|tank| tank.player_id == player_id && tank.alive)
        {
            tank.destroy();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_limit_is_ten() {
        let mut state = GameState::default();
        for index in 0..10 {
            assert!(state.add_player(format!("P{index}"), None).is_some());
        }
        assert!(state.add_player("P10", None).is_none());
    }

    #[test]
    fn tank_moves_at_fixed_speed() {
        let config = GameConfig::default();
        let mut tank = Tank::new(PlayerId(0), Vec2::ZERO);
        tank.move_forward(1.0, &config);
        assert!((tank.position.x - config.tank_speed).abs() < 0.001);
    }

    #[test]
    fn tank_rotation_changes_direction() {
        let config = GameConfig::default();
        let mut tank = Tank::new(PlayerId(0), Vec2::ZERO);
        tank.turn_right(0.5, &config);
        assert!(tank.direction().y > 0.0);
    }

    #[test]
    fn projectile_limit_is_eight_per_tank() {
        let config = GameConfig::default();
        let mut state = GameState::default();
        let player = state.add_player("P1", None).unwrap();
        state.add_tank(player, Vec2::ZERO);
        for _ in 0..8 {
            assert!(state.fire(player, &config).is_some());
        }
        assert!(state.fire(player, &config).is_none());
    }

    #[test]
    fn expired_projectiles_are_removed() {
        let config = GameConfig::default();
        let mut state = GameState::default();
        let player = state.add_player("P1", None).unwrap();
        state.add_tank(player, Vec2::ZERO);
        state.fire(player, &config);
        state.update_projectiles(5.0);
        assert!(state.projectiles.is_empty());
    }

    #[test]
    fn direct_hit_destroys_tank() {
        let mut state = GameState::default();
        let player = state.add_player("P1", None).unwrap();
        state.add_tank(player, Vec2::ZERO);
        assert!(state.destroy_tank(player));
        assert!(!state.tanks[0].alive);
    }
}
