mod game;
mod lobby;
#[allow(dead_code)]
mod net_session;
mod network;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use game::{
    AiController, GameConfig, GameMap, GameSimulation, MapSize, Mine, PlayerId, PowerUp,
    PowerUpKind, RoundState, TankInput, Vec2,
};
use glib::{ControlFlow, Propagation};
use gtk4::cairo::Context;
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Application, ApplicationWindow, Box, Button, CheckButton, ComboBoxText,
    DrawingArea, Entry, EventControllerKey, Label, LinkButton, Orientation, SpinButton, Stack,
    StackSwitcher,
};

const APP_ID: &str = "io.github.praxis1071.TankRush";
const PLAYER_COLORS: [(f64, f64, f64); 4] = [
    (1.0, 0.78, 0.12),
    (0.08, 0.09, 0.10),
    (0.95, 0.76, 0.18),
    (0.16, 0.17, 0.18),
];

#[derive(Debug, Clone)]
struct ControlBindings {
    keys: [String; 5],
}

impl ControlBindings {
    fn new(keys: [&str; 5]) -> Self {
        Self {
            keys: keys.map(str::to_string),
        }
    }
}

fn default_controls() -> Vec<ControlBindings> {
    vec![
        ControlBindings::new(["w", "s", "a", "d", "q"]),
        ControlBindings::new(["Up", "Down", "Left", "Right", "m"]),
        ControlBindings::new(["i", "k", "j", "l", "o"]),
        ControlBindings::new(["8", "5", "4", "6", "0"]),
    ]
}

fn key_label(key: &str) -> String {
    match key {
        "space" => "Space".into(),
        "Return" => "Enter".into(),
        "Escape" => "Esc".into(),
        "Up" => "↑".into(),
        "Down" => "↓".into(),
        "Left" => "←".into(),
        "Right" => "→".into(),
        _ => key.to_uppercase(),
    }
}

fn arena_seed() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
}

struct MatchRuntime {
    simulation: GameSimulation,
    map_size: MapSize,
    human_count: usize,
    names: Vec<String>,
    round: RoundState,
    ai: Option<AiController>,
    ai_id: Option<PlayerId>,
    weapons: [Option<PowerUpKind>; 4],
    weapon_uses: [u8; 4],
    powerups: Vec<PowerUp>,
    mines: Vec<Mine>,
    next_mine_id: u32,
    powerup_seed: u32,
    round_winner: Option<PlayerId>,
    draw_round: bool,
    respawn_countdown: f32,
    paused: bool,
    guided_projectiles: Vec<game::ProjectileId>,
    powerups_enabled: bool,
}

impl MatchRuntime {
    fn new(
        human_count: usize,
        map_size: MapSize,
        names: Vec<String>,
        seed: u32,
        powerups_enabled: bool,
    ) -> Self {
        let mut runtime = Self {
            simulation: GameSimulation::new(
                GameMap::generate_seeded(map_size, seed),
                GameConfig::default(),
            ),
            map_size,
            human_count,
            names,
            round: RoundState::new(),
            ai: (human_count == 1).then(AiController::default),
            ai_id: None,
            weapons: [None; 4],
            weapon_uses: [0; 4],
            powerups: Vec::new(),
            mines: Vec::new(),
            next_mine_id: 1,
            powerup_seed: seed ^ 0xA51C_39E7,
            round_winner: None,
            draw_round: false,
            respawn_countdown: 0.0,
            paused: false,
            guided_projectiles: Vec::new(),
            powerups_enabled,
        };
        runtime.spawn_round(seed);
        runtime
    }

    fn spawn_round(&mut self, seed: u32) {
        self.simulation = GameSimulation::new(
            GameMap::generate_seeded(self.map_size, seed.max(1)),
            GameConfig::default(),
        );
        self.round_winner = None;
        self.draw_round = false;
        self.respawn_countdown = 0.0;
        self.weapons = [None; 4];
        self.weapon_uses = [0; 4];
        self.ai_id = None;
        self.paused = false;
        self.guided_projectiles.clear();
        self.mines.clear();
        self.next_mine_id = 1;
        let spawns = self.simulation.map.spawn_points().to_vec();
        for index in 0..self.human_count {
            let id = self
                .simulation
                .state
                .add_player(self.names[index].clone(), None)
                .unwrap();
            self.simulation.state.add_tank(id, spawns[index]);
        }
        if self.human_count == 1 {
            let id = self.simulation.state.add_player("Laika", None).unwrap();
            self.simulation.state.add_tank(id, spawns[1]);
            self.weapons[id.0 as usize] = Some(PowerUpKind::Laser);
            self.weapon_uses[id.0 as usize] = u8::MAX;
            self.ai_id = Some(id);
            self.ai = Some(AiController::default());
        }
        self.powerups.clear();
        if self.powerups_enabled {
            while self.powerups.len() < 5 {
                self.powerups.push(PowerUp::generate(
                    &self.simulation.map,
                    &mut self.powerup_seed,
                ));
            }
        }
    }

    fn player_name(&self, id: PlayerId) -> String {
        self.simulation
            .state
            .players
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Tank".into())
    }

    fn input_pairs(&mut self, inputs: &[TankInput; 4], dt: f32) -> Vec<(PlayerId, TankInput)> {
        let mut pairs = inputs
            .iter()
            .copied()
            .enumerate()
            .take(self.human_count)
            .map(|(i, input)| (PlayerId(i as u8), input))
            .collect::<Vec<_>>();
        if let (Some(ai), Some(ai_id)) = (&mut self.ai, self.ai_id) {
            pairs.push((
                ai_id,
                ai.input(
                    &self.simulation.state,
                    &self.simulation.map,
                    &self.simulation.config,
                    &self.powerups,
                    ai_id,
                    dt,
                ),
            ));
        }
        pairs
    }

    fn consume_weapon(&mut self, owner: PlayerId) {
        let index = owner.0 as usize;
        if index >= 4 {
            return;
        }
        self.weapon_uses[index] = self.weapon_uses[index].saturating_sub(1);
        if self.weapon_uses[index] == 0 {
            self.weapons[index] = None;
        }
    }

    fn apply_weapon(&mut self, owner: PlayerId) {
        let index = owner.0 as usize;
        if index >= 4 {
            return;
        }
        let Some(kind) = self.weapons[index] else {
            return;
        };
        match kind {
            PowerUpKind::DoubleShot => {
                self.simulation.state.fire(owner, &self.simulation.config);
                let ids = self
                    .simulation
                    .state
                    .projectiles
                    .iter()
                    .filter(|p| p.owner == owner)
                    .map(|p| p.id)
                    .collect::<Vec<_>>();
                if let Some(id) = ids.last().copied() {
                    if let Some(projectile) = self
                        .simulation
                        .state
                        .projectiles
                        .iter_mut()
                        .find(|p| p.id == id)
                    {
                        let speed = projectile.velocity.length();
                        let angle = projectile.velocity.y.atan2(projectile.velocity.x) + 0.20;
                        projectile.velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
                    }
                }
            }
            PowerUpKind::MachineGun => {}
            PowerUpKind::Laser => {
                if let Some(projectile) = self
                    .simulation
                    .state
                    .projectiles
                    .iter_mut()
                    .rev()
                    .find(|p| p.owner == owner)
                {
                    projectile.velocity = projectile.velocity * 1.8;
                    projectile.remaining_lifetime = 2.2;
                }
            }
            PowerUpKind::GuidedMissile => {
                if let Some(projectile) = self
                    .simulation
                    .state
                    .projectiles
                    .iter_mut()
                    .rev()
                    .find(|p| p.owner == owner)
                {
                    projectile.velocity = projectile.velocity * 0.92;
                    projectile.remaining_lifetime = 5.0;
                    self.guided_projectiles.push(projectile.id);
                }
            }
            PowerUpKind::Shrapnel => {
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
                let Some(tank) = self
                    .simulation
                    .state
                    .tanks
                    .iter()
                    .find(|tank| tank.player_id == owner && tank.alive)
                else {
                    return;
                };
                if Mine::valid_position(
                    tank.position,
                    &self.simulation.map,
                    self.simulation.config.tank_radius,
                ) {
                    self.mines.push(Mine::new(
                        self.next_mine_id,
                        owner,
                        tank.position,
                    ));
                    self.next_mine_id = self.next_mine_id.wrapping_add(1).max(1);
                }
            }
        }
        self.consume_weapon(owner);
    }

    fn collect_powerups(&mut self) {
        if !self.powerups_enabled {
            return;
        }
        let mut collected = Vec::new();
        for (index, powerup) in self.powerups.iter().enumerate() {
            if let Some(tank) = self
                .simulation
                .state
                .tanks
                .iter()
                .find(|t| t.alive && (t.position - powerup.position).length() < 28.0)
            {
                let player = tank.player_id.0 as usize;
                if player < 4 {
                    self.weapons[player] = Some(powerup.kind);
                    self.weapon_uses[player] = powerup.kind.ammo().unwrap_or(u8::MAX);
                    collected.push(index);
                }
            }
        }
        for index in collected.into_iter().rev() {
            self.powerups.remove(index);
        }
        while self.powerups.len() < 5 {
            self.powerups.push(PowerUp::generate(
                &self.simulation.map,
                &mut self.powerup_seed,
            ));
        }
    }

    fn update_mines(&mut self, dt: f32) {
        for mine in &mut self.mines {
            mine.update(dt);
        }
        let triggered = self
            .mines
            .iter()
            .filter_map(|mine| mine.triggered_by(&self.simulation.state).map(|id| (mine.id, id)))
            .collect::<Vec<_>>();
        for (_, player_id) in triggered.iter().copied() {
            self.simulation.state.destroy_tank(player_id);
        }
        self.mines.retain(|mine| {
            !mine.expired() && !triggered.iter().any(|(id, _)| *id == mine.id)
        });
    }

    fn guide_projectiles(&mut self) {
        self.guided_projectiles.retain(|id| {
            self.simulation
                .state
                .projectiles
                .iter()
                .any(|projectile| projectile.id == *id)
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
    }

    fn toggle_pause(&mut self) {
        if self.round_winner.is_none() && !self.draw_round {
            self.paused = !self.paused;
        }
    }

    fn tick(&mut self, dt: f32, inputs: &[TankInput; 4]) {
        if self.paused {
            return;
        }
        if self.round_winner.is_some() || self.draw_round {
            self.respawn_countdown = (self.respawn_countdown - dt).max(0.0);
            if self.respawn_countdown == 0.0 {
                self.round.next_round();
                self.spawn_round(arena_seed());
            }
            return;
        }
        let pairs = self.input_pairs(inputs, dt);
        let events = self.simulation.advance(dt, &pairs);
        for event in events {
            if event.kind == game::SimulationEventKind::Fired {
                self.apply_weapon(event.player_id);
            }
        }
        self.guide_projectiles();
        self.update_mines(dt);
        self.collect_powerups();
        let alive = self
            .simulation
            .state
            .tanks
            .iter()
            .filter(|t| t.alive)
            .count();
        if alive == 1 {
            let winner = self
                .simulation
                .state
                .tanks
                .iter()
                .find(|t| t.alive)
                .unwrap()
                .player_id;
            self.round.award(winner);
            self.round_winner = Some(winner);
            self.respawn_countdown = 2.5;
        } else if alive == 0 {
            self.draw_round = true;
            self.respawn_countdown = 2.5;
        }
    }
}

fn draw_game(context: &Context, width: i32, height: i32, runtime: &MatchRuntime) {
    let width = width as f64;
    let height = height as f64;
    let simulation = &runtime.simulation;
    let scale = (width / simulation.map.width as f64).min(height / simulation.map.height as f64);
    let offset_x = (width - simulation.map.width as f64 * scale) / 2.0;
    let offset_y = (height - simulation.map.height as f64 * scale) / 2.0;

    context.set_source_rgb(0.025, 0.035, 0.042);
    context.paint().ok();
    context.save().ok();
    context.translate(offset_x, offset_y);
    context.scale(scale, scale);
    context.set_source_rgb(0.88, 0.89, 0.90);
    context.rectangle(0.0, 0.0, simulation.map.width as f64, simulation.map.height as f64);
    context.fill().ok();

    context.set_source_rgb(0.16, 0.17, 0.18);
    for wall in &simulation.map.walls {
        context.rectangle(
            wall.min.x as f64,
            wall.min.y as f64,
            (wall.max.x - wall.min.x) as f64,
            (wall.max.y - wall.min.y) as f64,
        );
        context.fill().ok();
    }

    for powerup in &runtime.powerups {
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

    for mine in &runtime.mines {
        context.set_source_rgb(0.025, 0.025, 0.03);
        context.arc(
            mine.position.x as f64,
            mine.position.y as f64,
            8.0,
            0.0,
            std::f64::consts::TAU,
        );
        context.fill().ok();
        if mine.armed() {
            context.set_source_rgb(0.95, 0.70, 0.12);
            context.arc(
                mine.position.x as f64,
                mine.position.y as f64,
                3.0,
                0.0,
                std::f64::consts::TAU,
            );
            context.fill().ok();
        }
    }

    for projectile in &simulation.state.projectiles {
        let velocity = projectile.velocity.normalized();
        context.set_source_rgba(0.02, 0.02, 0.025, 0.25);
        context.set_line_width(2.5);
        context.move_to(
            (projectile.position.x - velocity.x * 12.0) as f64,
            (projectile.position.y - velocity.y * 12.0) as f64,
        );
        context.line_to(projectile.position.x as f64, projectile.position.y as f64);
        context.stroke().ok();
        context.set_source_rgb(0.01, 0.01, 0.012);
        context.arc(
            projectile.position.x as f64,
            projectile.position.y as f64,
            simulation.config.projectile_radius as f64 + 1.0,
            0.0,
            std::f64::consts::TAU,
        );
        context.fill().ok();
    }

    for tank in &simulation.state.tanks {
        if !tank.alive {
            context.set_source_rgba(0.10, 0.10, 0.10, 0.24);
            context.arc(
                tank.position.x as f64,
                tank.position.y as f64,
                18.0,
                0.0,
                std::f64::consts::TAU,
            );
            context.fill().ok();
            continue;
        }
        let color = PLAYER_COLORS[tank.player_id.0 as usize % PLAYER_COLORS.len()];
        context.save().ok();
        context.translate(tank.position.x as f64, tank.position.y as f64);
        context.rotate(tank.rotation_radians as f64);
        context.set_source_rgba(0.02, 0.03, 0.04, 0.28);
        context.rectangle(-19.0, -14.0, 38.0, 28.0);
        context.fill().ok();
        context.set_source_rgb(0.08, 0.09, 0.10);
        context.rectangle(-18.0, -14.0, 36.0, 5.0);
        context.rectangle(-18.0, 9.0, 36.0, 5.0);
        context.fill().ok();
        context.set_source_rgb(color.0 * 0.78, color.1 * 0.78, color.2 * 0.78);
        context.rectangle(-15.0, -10.0, 30.0, 20.0);
        context.fill().ok();
        context.set_source_rgb(color.0, color.1, color.2);
        context.rectangle(-14.0, -8.0, 28.0, 5.0);
        context.fill().ok();
        context.set_source_rgb(0.16, 0.17, 0.18);
        context.arc(0.0, 0.0, 9.0, 0.0, std::f64::consts::TAU);
        context.fill().ok();
        context.set_source_rgb(0.32, 0.33, 0.34);
        context.arc(0.0, 0.0, 5.5, 0.0, std::f64::consts::TAU);
        context.fill().ok();
        context.set_source_rgb(0.015, 0.015, 0.018);
        context.set_line_width(7.0);
        context.set_line_cap(gtk4::cairo::LineCap::Round);
        context.move_to(1.0, 0.0);
        context.line_to(25.0, 0.0);
        context.stroke().ok();
        context.restore().ok();

        context.set_source_rgb(0.12, 0.14, 0.16);
        context.set_font_size(11.0);
        context.move_to(
            (tank.position.x - 22.0) as f64,
            (tank.position.y - 21.0) as f64,
        );
        context.show_text(&runtime.player_name(tank.player_id)).ok();
    }
    context.restore().ok();

    let alive = simulation.state.tanks.iter().filter(|tank| tank.alive).count();
    context.set_source_rgb(0.93, 0.95, 0.97);
    context.select_font_face(
        "Sans",
        gtk4::cairo::FontSlant::Normal,
        gtk4::cairo::FontWeight::Bold,
    );
    context.set_font_size(17.0);
    context.move_to(18.0, 28.0);
    context
        .show_text(&format!(
            "TANKRUSH  •  Round {}  •  Survivors: {}",
            runtime.round.round, alive
        ))
        .ok();
    context.set_font_size(14.0);
    context.move_to(18.0, 50.0);
    let score = (0..runtime.simulation.state.players.len().min(4))
        .map(|i| {
            format!(
                "{}: {}",
                runtime.player_name(PlayerId(i as u8)),
                runtime.round.scores[i]
            )
        })
        .collect::<Vec<_>>()
        .join("   ");
    context.show_text(&score).ok();

    if runtime.paused {
        context.set_source_rgb(0.03, 0.04, 0.05);
        context.rectangle(0.0, 0.0, width, height);
        context.fill().ok();
        context.set_source_rgb(0.95, 0.96, 0.98);
        context.set_font_size(34.0);
        context.move_to(width / 2.0 - 80.0, height / 2.0);
        context.show_text("PAUSED").ok();
        context.set_font_size(15.0);
        context.move_to(width / 2.0 - 115.0, height / 2.0 + 30.0);
        context.show_text("Press Esc to resume").ok();
    } else if let Some(winner) = runtime.round_winner {
        context.set_font_size(28.0);
        context.move_to(24.0, height - 48.0);
        context
            .show_text(&format!("{} WINS THE ROUND!", runtime.player_name(winner)))
            .ok();
        context.set_font_size(15.0);
        context.move_to(24.0, height - 24.0);
        context.show_text("New random arena incoming...").ok();
    } else if runtime.draw_round {
        context.set_font_size(28.0);
        context.move_to(24.0, height - 48.0);
        context.show_text("DRAW — NEW ROUND").ok();
    }
}

fn build_game_screen(
    stack: &Stack,
    player_count: usize,
    map_size: MapSize,
    names: Vec<String>,
    bindings: Vec<ControlBindings>,
    powerups_enabled: bool,
) -> (Box, DrawingArea) {
    let runtime = Rc::new(RefCell::new(MatchRuntime::new(
        player_count,
        map_size,
        names,
        arena_seed(),
        powerups_enabled,
    )));
    let inputs = Rc::new(RefCell::new([TankInput::idle(); 4]));
    let fire_pending = Rc::new(RefCell::new([false; 4]));
    let fire_down = Rc::new(RefCell::new([false; 4]));
    let rapid_timer = Rc::new(RefCell::new([0.0f32; 4]));
    let active = Rc::new(Cell::new(true));

    let root = Box::new(Orientation::Vertical, 0);
    let toolbar = Box::new(Orientation::Horizontal, 8);
    toolbar.set_margin_top(8);
    toolbar.set_margin_bottom(8);
    toolbar.set_margin_start(10);
    toolbar.set_margin_end(10);
    let title = Label::new(Some("TankRush  •  Battle Arena"));
    title.set_hexpand(true);
    title.set_halign(gtk4::Align::Start);
    let pause = Button::with_label("Pause");
    let back = Button::with_label("Back to Menu");
    toolbar.append(&title);
    toolbar.append(&pause);
    toolbar.append(&back);
    root.append(&toolbar);
    let drawing_area = DrawingArea::new();
    drawing_area.set_content_width(1100);
    drawing_area.set_content_height(700);
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    drawing_area.set_focusable(true);
    {
        let runtime = Rc::clone(&runtime);
        drawing_area.set_draw_func(move |_, context, width, height| {
            draw_game(context, width, height, &runtime.borrow())
        });
    }
    let key_controller = EventControllerKey::new();
    {
        let runtime = Rc::clone(&runtime);
        let inputs = Rc::clone(&inputs);
        let pending = Rc::clone(&fire_pending);
        let down = Rc::clone(&fire_down);
        let bindings = bindings.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            let Some(name) = key.name() else {
                return Propagation::Proceed;
            };
            let name = name.to_string();
            if name == "Escape" {
                runtime.borrow_mut().toggle_pause();
                return Propagation::Stop;
            }
            let mut input = inputs.borrow_mut();
            let mut pending = pending.borrow_mut();
            let mut down = down.borrow_mut();
            for player in 0..player_count {
                let Some(action) = bindings[player]
                    .keys
                    .iter()
                    .position(|binding| binding == &name)
                else {
                    continue;
                };
                match action {
                    0 => input[player].forward = true,
                    1 => input[player].backward = true,
                    2 => input[player].left = true,
                    3 => input[player].right = true,
                    4 => {
                        if !down[player] {
                            pending[player] = true;
                        }
                        down[player] = true;
                    }
                    _ => unreachable!(),
                }
                return Propagation::Stop;
            }
            Propagation::Proceed
        });
    }
    {
        let inputs = Rc::clone(&inputs);
        let down = Rc::clone(&fire_down);
        let bindings = bindings.clone();
        key_controller.connect_key_released(move |_, key, _, _| {
            let Some(name) = key.name() else {
                return;
            };
            let name = name.to_string();
            let mut input = inputs.borrow_mut();
            let mut down = down.borrow_mut();
            for player in 0..player_count {
                let Some(action) = bindings[player]
                    .keys
                    .iter()
                    .position(|binding| binding == &name)
                else {
                    continue;
                };
                match action {
                    0 => input[player].forward = false,
                    1 => input[player].backward = false,
                    2 => input[player].left = false,
                    3 => input[player].right = false,
                    4 => down[player] = false,
                    _ => unreachable!(),
                }
            }
        });
    }
    drawing_area.add_controller(key_controller);
    root.append(&drawing_area);

    let runtime_pause = Rc::clone(&runtime);
    let drawing_pause = drawing_area.clone();
    pause.connect_clicked(move |button| {
        let mut runtime = runtime_pause.borrow_mut();
        runtime.toggle_pause();
        button.set_label(if runtime.paused { "Resume" } else { "Pause" });
        drawing_pause.queue_draw();
    });

    let active_timer = Rc::clone(&active);
    let runtime_timer = Rc::clone(&runtime);
    let inputs_timer = Rc::clone(&inputs);
    let pending_timer = Rc::clone(&fire_pending);
    let down_timer = Rc::clone(&fire_down);
    let rapid_timer_ref = Rc::clone(&rapid_timer);
    let drawing = drawing_area.clone();
    glib::source::timeout_add_local(Duration::from_millis(16), move || {
        if !active_timer.get() {
            return ControlFlow::Break;
        }
        let mut snapshot = *inputs_timer.borrow();
        let mut pending = pending_timer.borrow_mut();
        let down = *down_timer.borrow();
        let mut rapid = rapid_timer_ref.borrow_mut();
        let weapon_snapshot = runtime_timer.borrow().weapons;
        for index in 0..player_count {
            if pending[index] {
                snapshot[index].fire = true;
                pending[index] = false;
            }
            if weapon_snapshot[index] == Some(PowerUpKind::MachineGun) && down[index] {
                rapid[index] -= 1.0 / 60.0;
                if rapid[index] <= 0.0 {
                    snapshot[index].fire = true;
                    rapid[index] = 0.12;
                }
            } else {
                rapid[index] = 0.0;
            }
        }
        runtime_timer.borrow_mut().tick(1.0 / 60.0, &snapshot);
        drawing.queue_draw();
        ControlFlow::Continue
    });
    let stack_back = stack.clone();
    let active_back = Rc::clone(&active);
    back.connect_clicked(move |_| {
        active_back.set(false);
        stack_back.set_visible_child_name("menu");
    });
    (root, drawing_area)
}

fn build_setup_screen(
    stack: &Stack,
    player_count: SpinButton,
    bindings: Rc<RefCell<Vec<ControlBindings>>>,
) -> Box {
    let root = Box::new(Orientation::Vertical, 10);
    root.set_margin_top(24);
    root.set_margin_bottom(24);
    root.set_margin_start(32);
    root.set_margin_end(32);
    root.append(&Label::new(Some("Local Battle Setup")));
    let maps = ComboBoxText::new();
    maps.append_text("Small Arena");
    maps.append_text("Medium Arena");
    maps.append_text("Large Arena");
    maps.append_text("Very Large Arena");
    maps.set_active(Some(0));
    root.append(&Label::new(Some("Players")));
    root.append(&player_count);
    root.append(&Label::new(Some("Map size")));
    root.append(&maps);
    let powerups = CheckButton::with_label("Enable power-ups");
    powerups.set_active(true);
    powerups.set_tooltip_text(Some("Collect temporary weapons during the match."));
    root.append(&powerups);
    let setup = Box::new(Orientation::Vertical, 7);
    let pending = Rc::new(RefCell::new(None::<(usize, usize, Button)>));
    let pending_label = Label::new(Some(
        "Choose a control button, then press a key. Escape cancels.",
    ));
    pending_label.add_css_class("dim-label");
    let mut names = Vec::new();
    for player in 0..4 {
        let row = Box::new(Orientation::Horizontal, 7);
        let name = Entry::new();
        name.set_text(&format!("Player {}", player + 1));
        name.set_width_chars(14);
        names.push(name.clone());
        row.append(&Label::new(Some(&format!("P{}", player + 1))));
        row.append(&name);
        for action in 0..5 {
            let button = Button::with_label(&key_label(&bindings.borrow()[player].keys[action]));
            let labels = ["FWD", "BACK", "LEFT", "RIGHT", "FIRE"];
            button.set_tooltip_text(Some(labels[action]));
            let p = Rc::clone(&pending);
            let l = pending_label.clone();
            button.connect_clicked(move |b| {
                *p.borrow_mut() = Some((player, action, b.clone()));
                l.set_text(&format!(
                    "P{} {}: press the new key…",
                    player + 1,
                    labels[action]
                ));
            });
            row.append(&button);
        }
        setup.append(&row);
    }
    root.append(&Label::new(Some("Player names and controls")));
    root.append(&setup);
    root.append(&pending_label);
    let key = EventControllerKey::new();
    {
        let pending = Rc::clone(&pending);
        let bindings = Rc::clone(&bindings);
        let label = pending_label.clone();
        key.connect_key_pressed(move |_, key, _, _| {
            let Some(name) = key.name() else {
                return Propagation::Stop;
            };
            let name = name.to_string();
            if name == "Escape" {
                *pending.borrow_mut() = None;
                label.set_text("Control assignment cancelled.");
                return Propagation::Stop;
            }
            if let Some((p, a, b)) = pending.borrow_mut().take() {
                bindings.borrow_mut()[p].keys[a] = name.clone();
                b.set_label(&key_label(&name));
                label.set_text(&format!(
                    "P{} control changed to {}.",
                    p + 1,
                    key_label(&name)
                ));
            }
            Propagation::Stop
        });
    }
    root.add_controller(key);
    let actions = Box::new(Orientation::Horizontal, 8);
    let back = Button::with_label("Back");
    let start = Button::with_label("Start Match");
    actions.append(&back);
    actions.append(&start);
    root.append(&actions);
    let stack_back = stack.clone();
    back.connect_clicked(move |_| stack_back.set_visible_child_name("menu"));
    let stack_start = stack.clone();
    let bindings_start = Rc::clone(&bindings);
    let player_count_start = player_count.clone();
    start.connect_clicked(move |_| {
        let count = player_count_start.value_as_int().clamp(1, 4) as usize;
        let map_size = match maps.active_text().as_deref() {
            Some("Medium Arena") => MapSize::Medium,
            Some("Large Arena") => MapSize::Large,
            Some("Very Large Arena") => MapSize::VeryLarge,
            _ => MapSize::Small,
        };
        let names = names
            .iter()
            .take(count)
            .enumerate()
            .map(|(i, e)| {
                let n = e.text().trim().to_string();
                if n.is_empty() {
                    format!("Player {}", i + 1)
                } else {
                    n.chars().take(18).collect()
                }
            })
            .collect::<Vec<_>>();
        let (game, area) = build_game_screen(
            &stack_start,
            count,
            map_size,
            names,
            bindings_start.borrow().clone(),
            powerups.is_active(),
        );
        if let Some(old) = stack_start.child_by_name("game") {
            stack_start.remove(&old);
        }
        stack_start.add_named(&game, Some("game"));
        stack_start.set_visible_child_name("game");
        area.grab_focus();
    });
    root
}

fn build_menu_screen(stack: &Stack, audio: Rc<RefCell<game::config::AudioSettings>>) -> Box {
    let content = Box::new(Orientation::Vertical, 12);
    content.set_margin_top(60);
    content.set_margin_bottom(60);
    content.set_margin_start(60);
    content.set_margin_end(60);
    content.set_halign(gtk4::Align::Center);
    content.set_valign(gtk4::Align::Center);
    let title = Label::new(Some("TANKRUSH"));
    title.add_css_class("title-1");
    content.append(&title);
    let subtitle = Label::new(Some("Tank arena • random mazes • ricochet combat"));
    subtitle.add_css_class("dim-label");
    content.append(&subtitle);
    let single = Button::with_label("Single Player / Local Battle");
    let lan = Button::with_label("LAN Multiplayer");
    let settings = Button::with_label("Settings");
    let quit = Button::with_label("Quit");
    for b in [&single, &lan, &settings, &quit] {
        b.set_width_request(280);
    }
    content.append(&single);
    content.append(&lan);
    content.append(&settings);
    content.append(&quit);
    let s = stack.clone();
    single.connect_clicked(move |_| s.set_visible_child_name("setup"));
    let s = stack.clone();
    lan.connect_clicked(move |_| s.set_visible_child_name("lan"));
    let s = stack.clone();
    let a = Rc::clone(&audio);
    settings.connect_clicked(move |_| {
        if let Some(p) = s.child_by_name("settings") {
            s.remove(&p);
        }
        let page = Box::new(Orientation::Vertical, 12);
        page.set_margin_top(32);
        page.set_margin_start(36);
        page.set_margin_end(36);
        page.set_margin_bottom(32);
        let tabs = Stack::new();
        tabs.set_vexpand(true);
        tabs.set_hexpand(true);
        let switcher = StackSwitcher::new();
        switcher.set_stack(Some(&tabs));
        let general = Box::new(Orientation::Vertical, 12);
        general.append(&Label::new(Some("Audio")));
        let music = CheckButton::with_label("Music");
        let effects = CheckButton::with_label("Sound effects");
        music.set_active(a.borrow().music_enabled);
        effects.set_active(a.borrow().sound_effects_enabled);
        {
            let a = Rc::clone(&a);
            music.connect_toggled(move |b| a.borrow_mut().music_enabled = b.is_active());
        }
        {
            let a = Rc::clone(&a);
            effects.connect_toggled(move |b| a.borrow_mut().sound_effects_enabled = b.is_active());
        }
        general.append(&music);
        general.append(&effects);
        general.append(&Label::new(Some("Gameplay")));
        general.append(&Label::new(Some(
            "Esc pauses the match. A new random arena is generated after each round.",
        )));
        tabs.add_titled(&general, Some("general"), "General");
        let about = Box::new(Orientation::Vertical, 10);
        about.set_margin_top(12);
        about.set_margin_start(8);
        about.set_margin_end(8);
        about.append(&Label::new(Some("About TankRush")));
        about.append(&Label::new(Some(
            "TankRush is a Rust + GTK4 tank arena project inspired by classic ricochet tank games.",
        )));
        about.append(&Label::new(Some("Developer")));
        about.append(&Label::new(Some("Praxis1071")));
        about.append(&Label::new(Some("Source code")));
        about.append(&LinkButton::with_label(
            "https://github.com/Praxis1071/TankRush",
            "GitHub • Praxis1071/TankRush",
        ));
        about.append(&Label::new(Some("License: MIT")));
        tabs.add_titled(&about, Some("about"), "About");
        page.append(&switcher);
        page.append(&tabs);
        let back = Button::with_label("Back");
        let s2 = s.clone();
        back.connect_clicked(move |_| s2.set_visible_child_name("menu"));
        page.append(&back);
        s.add_named(&page, Some("settings"));
        s.set_visible_child_name("settings");
    });
    let s = stack.clone();
    quit.connect_clicked(move |_| {
        if let Some(root) = s.root() {
            root.downcast::<ApplicationWindow>().ok().map(|w| w.close());
        }
    });
    content
}

fn build_ui(app: &Application) {
    let audio = Rc::new(RefCell::new(game::config::AudioSettings::default()));
    let stack = Stack::new();
    stack.set_vexpand(true);
    stack.set_hexpand(true);
    let menu = build_menu_screen(&stack, Rc::clone(&audio));
    stack.add_named(&menu, Some("menu"));
    let bindings = Rc::new(RefCell::new(default_controls()));
    let players = SpinButton::with_range(1.0, 4.0, 1.0);
    players.set_value(2.0);
    let setup = build_setup_screen(&stack, players, bindings);
    stack.add_named(&setup, Some("setup"));
    let lan = Box::new(Orientation::Vertical, 12);
    lan.set_margin_top(36);
    lan.set_margin_start(36);
    lan.set_margin_end(36);
    lan.set_margin_bottom(36);
    lan.append(&Label::new(Some("LAN Multiplayer")));
    lan.append(&Label::new(Some(&format!(
        "Authoritative LAN protocol v{} • lobby up to {} players.",
        network::PROTOCOL_VERSION,
        lobby::MAX_LOBBY_PLAYERS
    ))));
    lan.append(&Label::new(Some(
        "LAN gameplay comes after the completed local arena core.",
    )));
    let back = Button::with_label("Back");
    let s = stack.clone();
    back.connect_clicked(move |_| s.set_visible_child_name("menu"));
    lan.append(&back);
    stack.add_named(&lan, Some("lan"));
    stack.set_visible_child_name("menu");
    let window = ApplicationWindow::builder()
        .application(app)
        .title("TankRush")
        .default_width(1200)
        .default_height(800)
        .child(&stack)
        .build();
    window.present();
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}
