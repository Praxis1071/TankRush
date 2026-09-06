from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    s = p.read_text()
    if old not in s:
        raise SystemExit(f"pattern not found in {path}: {old[:80]!r}")
    p.write_text(s.replace(old, new, 1))


# Visual theme: yellow hulls, black hardware, consistent AI/player presentation.
replace_once(
    "src/main.rs",
    """const PLAYER_COLORS: [(f64, f64, f64); 4] = [
    (0.25, 0.95, 0.35),
    (0.95, 0.20, 0.22),
    (0.20, 0.55, 1.0),
    (1.0, 0.82, 0.18),
];""",
    """const PLAYER_COLORS: [(f64, f64, f64); 4] = [
    (0.98, 0.78, 0.08),
    (0.98, 0.78, 0.08),
    (0.98, 0.78, 0.08),
    (0.98, 0.78, 0.08),
];""",
)
replace_once(
    "src/main.rs",
    """        let color = if runtime.ai_id == Some(tank.player_id) {
            (0.38, 0.39, 0.41)
        } else {
            PLAYER_COLORS[tank.player_id.0 as usize % PLAYER_COLORS.len()]
        };""",
    "        let color = PLAYER_COLORS[tank.player_id.0 as usize % PLAYER_COLORS.len()];",
)
replace_once(
    "src/main.rs",
    "context.set_source_rgb(0.70, 0.06, 0.07);",
    "context.set_source_rgb(1.0, 0.86, 0.10);",
)

# Tank Trouble 2-inspired baseline from the researched remake reference:
# 120 px/s tank speed, 4 rad/s turn rate, 300 px/s shell speed,
# 0.5 s fire cooldown, five active shells per tank.
replace_once("src/game/config.rs", "tank_turn_speed_radians: 3.6,", "tank_turn_speed_radians: 4.0,")
replace_once("src/game/config.rs", "max_active_projectiles_per_tank: 8,", "max_active_projectiles_per_tank: 5,")
replace_once(
    "src/game/config.rs",
    "pub max_active_projectiles_per_tank: usize,",
    "pub max_active_projectiles_per_tank: usize,\n    pub projectile_fire_cooldown_seconds: f32,",
)
replace_once(
    "src/game/config.rs",
    "max_active_projectiles_per_tank: 5,\n            fixed_timestep_seconds",
    "max_active_projectiles_per_tank: 5,\n            projectile_fire_cooldown_seconds: 0.5,\n            fixed_timestep_seconds",
)
replace_once(
    "src/game/config.rs",
    "assert!((config.tank_turn_speed_radians - 3.6).abs() < f32::EPSILON);",
    "assert!((config.tank_turn_speed_radians - 4.0).abs() < f32::EPSILON);\n        assert!((config.projectile_fire_cooldown_seconds - 0.5).abs() < f32::EPSILON);",
)

# Real fire cadence is enforced in the simulation, independent of frame rate.
replace_once(
    "src/game/entity.rs",
    """pub struct GameState {
    pub players: Vec<Player>,
    pub tanks: Vec<Tank>,
    pub projectiles: Vec<Projectile>,
    next_projectile_id: u32,
}""",
    """pub struct GameState {
    pub players: Vec<Player>,
    pub tanks: Vec<Tank>,
    pub projectiles: Vec<Projectile>,
    pub fire_cooldowns: Vec<f32>,
    next_projectile_id: u32,
}""",
)
replace_once(
    "src/game/entity.rs",
    """        self.players.push(Player {
            id,
            name: name.into(),
            team,
        });
        Some(id)""",
    """        self.players.push(Player {
            id,
            name: name.into(),
            team,
        });
        self.fire_cooldowns.push(0.0);
        Some(id)""",
)
replace_once(
    "src/game/entity.rs",
    """    pub fn fire(&mut self, player_id: PlayerId, config: &GameConfig) -> Option<ProjectileId> {
        let active_count = self
            .projectiles
            .iter()
            .filter(|p| p.owner == player_id)
            .count();""",
    """    pub fn tick_fire_cooldowns(&mut self, seconds: f32) {
        for cooldown in &mut self.fire_cooldowns {
            *cooldown = (*cooldown - seconds).max(0.0);
        }
    }

    pub fn fire(&mut self, player_id: PlayerId, config: &GameConfig) -> Option<ProjectileId> {
        let cooldown = *self.fire_cooldowns.get(player_id.0 as usize).unwrap_or(&0.0);
        if cooldown > 0.0 {
            return None;
        }
        let active_count = self
            .projectiles
            .iter()
            .filter(|p| p.owner == player_id)
            .count();""",
)
replace_once(
    "src/game/entity.rs",
    """        self.projectiles.push(Projectile::new(
            id,
            player_id,
            spawn_position,
            tank.direction(),
            config,
        ));
        Some(id)""",
    """        self.projectiles.push(Projectile::new(
            id,
            player_id,
            spawn_position,
            tank.direction(),
            config,
        ));
        if let Some(cooldown) = self.fire_cooldowns.get_mut(player_id.0 as usize) {
            *cooldown = config.projectile_fire_cooldown_seconds;
        }
        Some(id)""",
)
replace_once(
    "src/game/entity.rs",
    "for _ in 0..8 {",
    "for _ in 0..5 {",
)
replace_once(
    "src/game/entity.rs",
    "assert!(state.fire(player, &config).is_none());",
    "assert!(state.fire(player, &config).is_none());\n        state.tick_fire_cooldowns(0.5);\n        assert!(state.fire(player, &config).is_some());",
)
replace_once(
    "src/game/simulation.rs",
    """        let dt = self.config.fixed_timestep_seconds;
        let mut events = Vec::new();""",
    """        let dt = self.config.fixed_timestep_seconds;
        self.state.tick_fire_cooldowns(dt);
        let mut events = Vec::new();""",
)

# Add a mine power-up using the existing deterministic projectile/collision pipeline.
replace_once(
    "src/game/powerup.rs",
    """    GuidedMissile,
    Shrapnel,
}""",
    """    GuidedMissile,
    Shrapnel,
    Mine,
}""",
)
replace_once(
    "src/game/powerup.rs",
    """        Self::GuidedMissile,
        Self::Shrapnel,
    ];""",
    """        Self::GuidedMissile,
        Self::Shrapnel,
        Self::Mine,
    ];""",
)
replace_once(
    "src/game/powerup.rs",
    """            Self::GuidedMissile => \"GUIDED\",
            Self::Shrapnel => \"FRAG\",""",
    """            Self::GuidedMissile => \"GUIDED\",
            Self::Shrapnel => \"FRAG\",
            Self::Mine => \"MINE\",""",
)
replace_once(
    "src/main.rs",
    """            PowerUpKind::Shrapnel => {
                for _ in 0..5 {""",
    """            PowerUpKind::Mine => {
                if let Some(projectile) = self
                    .simulation
                    .state
                    .projectiles
                    .iter_mut()
                    .rev()
                    .find(|p| p.owner == owner)
                {
                    projectile.velocity = Vec2::ZERO;
                    projectile.remaining_lifetime = 5.0;
                }
            }
            PowerUpKind::Shrapnel => {
                for _ in 0..5 {""",
)
