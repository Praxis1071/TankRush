from pathlib import Path

path = Path("src/main.rs")
text = path.read_text()

replacements = [
    (
        '''const PLAYER_COLORS: [(f64, f64, f64); 4] = [
    (0.25, 0.95, 0.35),
    (0.95, 0.20, 0.22),
    (0.20, 0.55, 1.0),
    (1.0, 0.82, 0.18),
];''',
        '''const PLAYER_COLORS: [(f64, f64, f64); 4] = [(0.98, 0.78, 0.08); 4];''',
    ),
    (
        '''        match kind {
            PowerUpKind::DoubleShot => {''',
        '''        match kind {
            PowerUpKind::DoubleShot => {''',
    ),
    (
        '''            PowerUpKind::Shrapnel => {
                for _ in 0..5 {
                    self.simulation.state.fire(owner, &self.simulation.config);
                }
                let mut angle = self
                    .simulation
                    .state
                    .tanks
                    .iter()
                    .find(|t| t.player_id == owner)
                    .map(|t| t.rotation_radians)
                    .unwrap_or(0.0)
                    - 0.65;
                for projectile in self
                    .simulation
                    .state
                    .projectiles
                    .iter_mut()
                    .rev()
                    .filter(|p| p.owner == owner)
                    .take(5)
                {
                    let speed = projectile.velocity.length();
                    projectile.velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
                    projectile.remaining_lifetime = 2.0;
                    angle += 0.26;
                }
            }
''',
        '''            PowerUpKind::Shrapnel => {
                for _ in 0..5 {
                    self.simulation.state.fire(owner, &self.simulation.config);
                }
                let mut angle = self
                    .simulation
                    .state
                    .tanks
                    .iter()
                    .find(|t| t.player_id == owner)
                    .map(|t| t.rotation_radians)
                    .unwrap_or(0.0)
                    - 0.65;
                for projectile in self
                    .simulation
                    .state
                    .projectiles
                    .iter_mut()
                    .rev()
                    .filter(|p| p.owner == owner)
                    .take(5)
                {
                    let speed = projectile.velocity.length();
                    projectile.velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
                    projectile.remaining_lifetime = 2.0;
                    angle += 0.26;
                }
            }
            PowerUpKind::Mine => {
                if let Some(projectile) = self
                    .simulation
                    .state
                    .projectiles
                    .iter_mut()
                    .rev()
                    .find(|p| p.owner == owner)
                {
                    projectile.velocity = Vec2::ZERO;
                    projectile.remaining_lifetime = 8.0;
                }
            }
''',
    ),
    (
        '''    for powerup in &runtime.powerups {
        context.set_source_rgb(0.95, 0.72, 0.20);
        context.arc(
            powerup.position.x as f64,
            powerup.position.y as f64,
            11.0,
            0.0,
            std::f64::consts::TAU,
        );
        context.fill().ok();
        context.set_source_rgb(0.08, 0.08, 0.08);
        context.set_font_size(7.0);
        context.move_to(
            (powerup.position.x - 15.0) as f64,
            (powerup.position.y + 3.0) as f64,
        );
        context.show_text(powerup.kind.label()).ok();
    }
''',
        '''    for powerup in &runtime.powerups {
        draw_weapon_icon(context, powerup.position, powerup.kind);
    }
''',
    ),
    (
        '''fn draw_game(context: &Context, width: i32, height: i32, runtime: &MatchRuntime) {''',
        '''fn draw_weapon_icon(context: &Context, position: Vec2, kind: PowerUpKind) {
    let x = position.x as f64;
    let y = position.y as f64;
    context.save().ok();
    context.set_source_rgb(0.08, 0.09, 0.10);
    context.arc(x, y, 14.0, 0.0, std::f64::consts::TAU);
    context.fill().ok();
    context.set_source_rgb(0.98, 0.78, 0.08);
    context.set_line_width(2.2);
    match kind {
        PowerUpKind::DoubleShot => {
            context.move_to(x - 7.0, y - 4.0);
            context.line_to(x + 7.0, y - 4.0);
            context.move_to(x - 7.0, y + 4.0);
            context.line_to(x + 7.0, y + 4.0);
            context.stroke().ok();
            context.arc(x + 8.0, y - 4.0, 2.5, 0.0, std::f64::consts::TAU);
            context.arc(x + 8.0, y + 4.0, 2.5, 0.0, std::f64::consts::TAU);
            context.fill().ok();
        }
        PowerUpKind::MachineGun => {
            for offset in [-6.0, 0.0, 6.0] {
                context.arc(x + offset, y, 2.5, 0.0, std::f64::consts::TAU);
                context.fill().ok();
            }
        }
        PowerUpKind::Laser => {
            context.move_to(x - 8.0, y + 7.0);
            context.line_to(x + 7.0, y - 7.0);
            context.stroke().ok();
            context.move_to(x - 1.0, y - 8.0);
            context.line_to(x + 8.0, y - 8.0);
            context.stroke().ok();
        }
        PowerUpKind::GuidedMissile => {
            context.move_to(x - 8.0, y);
            context.line_to(x + 5.0, y);
            context.stroke().ok();
            context.move_to(x + 4.0, y);
            context.line_to(x + 8.0, y - 4.0);
            context.line_to(x + 8.0, y + 4.0);
            context.close_path();
            context.fill().ok();
        }
        PowerUpKind::Shrapnel => {
            for angle in [0.0, 1.2566, 2.5133, 3.7699, 5.0265] {
                context.move_to(x, y);
                context.line_to(x + angle.cos() * 8.0, y + angle.sin() * 8.0);
            }
            context.stroke().ok();
            context.arc(x, y, 2.5, 0.0, std::f64::consts::TAU);
            context.fill().ok();
        }
        PowerUpKind::Mine => {
            context.arc(x, y, 6.0, 0.0, std::f64::consts::TAU);
            context.stroke().ok();
            for angle in [0.0, 1.5708, 3.1416, 4.7124] {
                context.move_to(x + angle.cos() * 6.0, y + angle.sin() * 6.0);
                context.line_to(x + angle.cos() * 10.0, y + angle.sin() * 10.0);
            }
            context.stroke().ok();
        }
    }
    context.set_source_rgb(0.08, 0.08, 0.08);
    context.set_font_size(7.0);
    context.move_to(x - 15.0, y + 24.0);
    context.show_text(kind.label()).ok();
    context.restore().ok();
}

fn draw_game(context: &Context, width: i32, height: i32, runtime: &MatchRuntime) {''',
    ),
    (
        '''        let color = if runtime.ai_id == Some(tank.player_id) {
            (0.38, 0.39, 0.41)
        } else {
            PLAYER_COLORS[tank.player_id.0 as usize % PLAYER_COLORS.len()]
        };''',
        '''        let color = PLAYER_COLORS[tank.player_id.0 as usize % PLAYER_COLORS.len()];''',
    ),
    (
        '''            // Laika gets subtle red sensor details.
            context.set_source_rgb(0.70, 0.06, 0.07);''',
        '''            // Laika keeps the same yellow-black silhouette with bright sensor details.
            context.set_source_rgb(1.0, 0.86, 0.10);''',
    ),
]

for old, new in replacements:
    if old not in text:
        raise SystemExit(f"required source pattern not found: {old[:80]!r}")
    text = text.replace(old, new, 1)

path.write_text(text)
