#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameConfig {
    pub tank_speed: f32,
    pub projectile_speed: f32,
    pub projectile_lifetime_seconds: f32,
    pub max_active_projectiles_per_tank: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            tank_speed: 120.0,
            projectile_speed: 180.0,
            projectile_lifetime_seconds: 5.0,
            max_active_projectiles_per_tank: 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    }

    #[test]
    fn default_projectile_rules_match_design() {
        let config = GameConfig::default();
        assert_eq!(config.projectile_lifetime_seconds, 5.0);
        assert_eq!(config.max_active_projectiles_per_tank, 8);
    }
}
