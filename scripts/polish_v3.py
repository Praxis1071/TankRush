from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f'{label}: expected 1 match, found {count}')
    return text.replace(old, new, 1)


main_path = Path('src/main.rs')
s = main_path.read_text()

s = replace_once(
    s,
    '    paused: bool,\n}',
    '    paused: bool,\n    guided_projectiles: Vec<game::ProjectileId>,\n    powerups_enabled: bool,\n}',
    'runtime fields',
)
s = replace_once(
    s,
    '    fn new(human_count: usize, map_size: MapSize, names: Vec<String>, seed: u32) -> Self {',
    '    fn new(human_count: usize, map_size: MapSize, names: Vec<String>, seed: u32, powerups_enabled: bool) -> Self {',
    'runtime constructor',
)
s = replace_once(
    s,
    '            paused: false,\n        };',
    '            paused: false,\n            guided_projectiles: Vec::new(),\n            powerups_enabled,\n        };',
    'runtime initialization',
)
s = replace_once(
    s,
    '        self.paused = false;\n        let spawns',
    '        self.paused = false;\n        self.guided_projectiles.clear();\n        let spawns',
    'round reset',
)

old_powerups = '''        self.powerups.clear();
        while self.powerups.len() < 5 {
            self.powerups.push(PowerUp::generate(
                &self.simulation.map,
                &mut self.powerup_seed,
            ));
        }'''
new_powerups = '''        self.powerups.clear();
        if self.powerups_enabled {
            while self.powerups.len() < 5 {
                self.powerups.push(PowerUp::generate(
                    &self.simulation.map,
                    &mut self.powerup_seed,
                ));
            }
        }'''
s = replace_once(s, old_powerups, new_powerups, 'powerup generation')

s = replace_once(
    s,
    '                    &self.simulation.config,\n                    ai_id,\n                    dt,',
    '                    &self.simulation.config,\n                    &self.powerups,\n                    ai_id,\n                    dt,',
    'AI powerup context',
)

s = replace_once(
    s,
    '                    projectile.velocity = projectile.velocity * 0.92;\n                    projectile.remaining_lifetime = 5.0;\n                }',
    '                    projectile.velocity = projectile.velocity * 0.92;\n                    projectile.remaining_lifetime = 5.0;\n                    self.guided_projectiles.push(projectile.id);\n                }',
    'guided projectile registration',
)

old_guide = '''    fn guide_projectiles(&mut self) {
        for projectile in &mut self.simulation.state.projectiles {
            let Some(owner_tank) = self
                .simulation
                .state
                .tanks
                .iter()
                .find(|t| t.player_id == projectile.owner)
            else {
                continue;
            };
            let Some(target) = self
                .simulation
                .state
                .tanks
                .iter()
                .filter(|t| t.alive && t.player_id != projectile.owner)
                .min_by(|a, b| {
                    ((a.position - projectile.position).length())
                        .total_cmp(&((b.position - projectile.position).length()))
                })
            else {
                continue;
            };
            if self.weapons[owner_tank.player_id.0 as usize] == Some(PowerUpKind::GuidedMissile) {
                let desired = (target.position - projectile.position).normalized();
                let speed = projectile.velocity.length();
                projectile.velocity =
                    (projectile.velocity.normalized() * 0.88 + desired * 0.12).normalized() * speed;
            }
        }
    }'''
new_guide = '''    fn guide_projectiles(&mut self) {
        self.guided_projectiles.retain(|id| {
            self.simulation.state.projectiles.iter().any(|projectile| projectile.id == *id)
        });
        let targets = self
            .simulation
            .state
            .tanks
            .iter()
            .filter(|tank| tank.alive)
            .map(|tank| (tank.player_id, tank.position))
            .collect::<Vec<_>>();
        for projectile in &mut self.simulation.state.projectiles {
            if !self.guided_projectiles.contains(&projectile.id) {
                continue;
            }
            let Some((_, target_position)) = targets
                .iter()
                .filter(|(id, _)| *id != projectile.owner)
                .min_by(|a, b| {
                    ((a.1 - projectile.position).length())
                        .total_cmp(&((b.1 - projectile.position).length()))
                })
            else {
                continue;
            };
            let desired = (*target_position - projectile.position).normalized();
            let speed = projectile.velocity.length();
            projectile.velocity =
                (projectile.velocity.normalized() * 0.84 + desired * 0.16).normalized() * speed;
        }
    }'''
s = replace_once(s, old_guide, new_guide, 'guided projectile steering')

s = replace_once(
    s,
    '    fn collect_powerups(&mut self) {\n',
    '    fn collect_powerups(&mut self) {\n        if !self.powerups_enabled {\n            return;\n        }\n',
    'powerup collection toggle',
)

old_tanks = '''    for tank in &simulation.state.tanks {
        if !tank.alive {
            continue;
        }
        let color = if runtime.ai_id == Some(tank.player_id) {
            (0.36, 0.37, 0.39)
        } else {
            PLAYER_COLORS[tank.player_id.0 as usize % PLAYER_COLORS.len()]
        };
        context.set_source_rgb(color.0, color.1, color.2);
        context.save().ok();
        context.translate(tank.position.x as f64, tank.position.y as f64);
        context.rotate(tank.rotation_radians as f64);
        context.rectangle(-16.0, -12.0, 32.0, 24.0);
        context.fill().ok();
        context.restore().ok();
        let direction = tank.direction();
        context.set_line_width(7.0);
        context.set_line_cap(gtk4::cairo::LineCap::Round);
        context.move_to(tank.position.x as f64, tank.position.y as f64);
        context.line_to(
            (tank.position.x + direction.x * 22.0) as f64,
            (tank.position.y + direction.y * 22.0) as f64,
        );
        context.stroke().ok();
        context.set_source_rgb(0.92, 0.94, 0.96);
        context.set_font_size(11.0);
        context.move_to(
            (tank.position.x - 22.0) as f64,
            (tank.position.y - 21.0) as f64,
        );
        context.show_text(&runtime.player_name(tank.player_id)).ok();
    }'''
new_tanks = '''    for tank in &simulation.state.tanks {
        if !tank.alive {
            context.set_source_rgba(0.10, 0.10, 0.10, 0.24);
            context.arc(tank.position.x as f64, tank.position.y as f64, 18.0, 0.0, std::f64::consts::TAU);
            context.fill().ok();
            continue;
        }
        let color = if runtime.ai_id == Some(tank.player_id) {
            (0.38, 0.39, 0.41)
        } else {
            PLAYER_COLORS[tank.player_id.0 as usize % PLAYER_COLORS.len()]
        };
        context.save().ok();
        context.translate(tank.position.x as f64, tank.position.y as f64);
        context.rotate(tank.rotation_radians as f64);

        // Ground shadow and heavy tracks.
        context.set_source_rgba(0.02, 0.03, 0.04, 0.28);
        context.rectangle(-19.0, -14.0, 38.0, 28.0);
        context.fill().ok();
        context.set_source_rgb(0.08, 0.09, 0.10);
        context.rectangle(-18.0, -14.0, 36.0, 5.0);
        context.rectangle(-18.0, 9.0, 36.0, 5.0);
        context.fill().ok();

        // Armored hull and color identification band.
        context.set_source_rgb(color.0 * 0.78, color.1 * 0.78, color.2 * 0.78);
        context.rectangle(-15.0, -10.0, 30.0, 20.0);
        context.fill().ok();
        context.set_source_rgb(color.0, color.1, color.2);
        context.rectangle(-14.0, -8.0, 28.0, 5.0);
        context.fill().ok();

        // Turret ring and hatch.
        context.set_source_rgb(0.16, 0.17, 0.18);
        context.arc(0.0, 0.0, 9.0, 0.0, std::f64::consts::TAU);
        context.fill().ok();
        context.set_source_rgb(0.32, 0.33, 0.34);
        context.arc(0.0, 0.0, 5.5, 0.0, std::f64::consts::TAU);
        context.fill().ok();

        // User-requested consistent black cannon silhouette.
        context.set_source_rgb(0.015, 0.015, 0.018);
        context.set_line_width(7.0);
        context.set_line_cap(gtk4::cairo::LineCap::Round);
        context.move_to(1.0, 0.0);
        context.line_to(25.0, 0.0);
        context.stroke().ok();
        context.set_source_rgb(0.35, 0.36, 0.37);
        context.set_line_width(2.0);
        context.move_to(10.0, 0.0);
        context.line_to(25.0, 0.0);
        context.stroke().ok();

        if runtime.ai_id == Some(tank.player_id) {
            // Laika gets subtle red sensor details.
            context.set_source_rgb(0.70, 0.06, 0.07);
            context.arc(0.0, -2.5, 1.5, 0.0, std::f64::consts::TAU);
            context.arc(0.0, 2.5, 1.5, 0.0, std::f64::consts::TAU);
            context.fill().ok();
        }
        context.restore().ok();

        if runtime.ai_id != Some(tank.player_id) || tank.alive {
            context.set_source_rgb(0.12, 0.14, 0.16);
            context.set_font_size(11.0);
            context.move_to(
                (tank.position.x - 22.0) as f64,
                (tank.position.y - 21.0) as f64,
            );
            context.show_text(&runtime.player_name(tank.player_id)).ok();
        }
    }'''
s = replace_once(s, old_tanks, new_tanks, 'tank rendering')

s = replace_once(
    s,
    '    for projectile in &simulation.state.projectiles {\n        context.set_source_rgb(1.0, 0.84, 0.22);',
    '''    for projectile in &simulation.state.projectiles {
        let velocity = projectile.velocity.normalized();
        context.set_source_rgba(1.0, 0.78, 0.16, 0.22);
        context.set_line_width(2.5);
        context.move_to(
            (projectile.position.x - velocity.x * 12.0) as f64,
            (projectile.position.y - velocity.y * 12.0) as f64,
        );
        context.line_to(projectile.position.x as f64, projectile.position.y as f64);
        context.stroke().ok();
        context.set_source_rgb(1.0, 0.84, 0.22);''',
    'projectile trails',
)

s = replace_once(
    s,
    'fn build_game_screen(\n    stack: &Stack,\n    player_count: usize,\n    map_size: MapSize,\n    names: Vec<String>,\n    bindings: Vec<ControlBindings>,\n) -> (Box, DrawingArea) {',
    '''fn build_game_screen(
    stack: &Stack,
    player_count: usize,
    map_size: MapSize,
    names: Vec<String>,
    bindings: Vec<ControlBindings>,
    powerups_enabled: bool,
) -> (Box, DrawingArea) {''',
    'game screen signature',
)
s = replace_once(
    s,
    '        names,\n        arena_seed(),\n    )));',
    '        names,\n        arena_seed(),\n        powerups_enabled,\n    )));',
    'game runtime construction',
)

setup_sig = '''fn build_setup_screen(
    stack: &Stack,
    player_count: SpinButton,
    bindings: Rc<RefCell<Vec<ControlBindings>>>,
) -> Box {'''
setup_new = '''fn build_setup_screen(
    stack: &Stack,
    player_count: SpinButton,
    bindings: Rc<RefCell<Vec<ControlBindings>>>,
) -> Box {'''
s = replace_once(s, setup_sig, setup_new, 'setup signature')

s = replace_once(
    s,
    '    root.append(&Label::new(Some("Arena size")));\n    root.append(&maps);',
    '''    root.append(&Label::new(Some("Arena size")));
    root.append(&maps);
    let powerups = CheckButton::with_label("Enable power-ups");
    powerups.set_active(true);
    powerups.set_tooltip_text(Some("Collect temporary weapons during the match."));
    root.append(&powerups);''',
    'setup power-up option',
)

s = replace_once(
    s,
    '            names,\n            bindings_start.borrow().clone(),\n        );',
    '            names,\n            bindings_start.borrow().clone(),\n            powerups.is_active(),\n        );',
    'start match power-up option',
)

main_path.write_text(s)

map_path = Path('src/game/map.rs')
m = map_path.read_text()
m = replace_once(m, 'let extra_openings = room_count * 2;', 'let extra_openings = room_count * 3;', 'map loop density')
map_path.write_text(m)
