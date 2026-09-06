#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameConfig {
    pub tank_speed: f32,
    pub tank_turn_speed_radians: f32,
    pub tank_radius: f32,
    pub projectile_speed: f32,
    pub projectile_lifetime_seconds: f32,
    pub projectile_radius: f32,
    pub max_active_projectiles_per_tank: usize,
    pub fire_cooldown_seconds: f32,
    pub fixed_timestep_seconds: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            tank_speed: 120.0,
            tank_turn_speed_radians: 4.0,
            tank_radius: 14.0,
            projectile_speed: 300.0,
            projectile_lifetime_seconds: 4.5,
            projectile_radius: 3.0,
            max_active_projectiles_per_tank: 5,
            fire_cooldown_seconds: 0.5,
            fixed_timestep_seconds: 1.0 / 60.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioSettings {
    pub music_enabled: bool,
    pub sound_effects_enabled: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            music_enabled: true,
            sound_effects_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_projectile_is_faster_than_tank() {
        let config = GameConfig::default();
        assert!(config.projectile_speed > config.tank_speed);
        assert!((config.projectile_speed - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn default_movement_and_projectile_rules_match_design() {
        let config = GameConfig::default();
        assert!((config.tank_speed - 120.0).abs() < f32::EPSILON);
        assert!((config.tank_turn_speed_radians - 4.0).abs() < f32::EPSILON);
        assert!((config.projectile_lifetime_seconds - 4.5).abs() < f32::EPSILON);
        assert_eq!(config.max_active_projectiles_per_tank, 5);
        assert!((config.fire_cooldown_seconds - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn fixed_timestep_is_sixty_hz() {
        assert!((GameConfig::default().fixed_timestep_seconds - 1.0 / 60.0).abs() < f32::EPSILON);
    }
}
