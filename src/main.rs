mod game;
mod lobby;
#[allow(dead_code)]
mod net_session;
mod network;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use game::{GameConfig, GameMap, GameSimulation, MapSize, PlayerId, TankInput};
use glib::{ControlFlow, Propagation};
use gtk4::cairo::Context;
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Application, ApplicationWindow, Box, Button, CheckButton, ComboBoxText,
    DrawingArea, Entry, EventControllerKey, Label, Orientation, SpinButton, Stack,
};

const APP_ID: &str = "io.github.praxis1071.TankRush";
const PLAYER_COLORS: [(f64, f64, f64); 4] = [
    (0.20, 0.75, 1.0),
    (1.0, 0.35, 0.35),
    (0.35, 1.0, 0.45),
    (1.0, 0.80, 0.25),
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
        "space" => "Space".to_string(),
        "Return" => "Enter".to_string(),
        "Up" => "↑".to_string(),
        "Down" => "↓".to_string(),
        "Left" => "←".to_string(),
        "Right" => "→".to_string(),
        _ => key.to_uppercase(),
    }
}

fn arena_seed() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
}

fn draw_game(context: &Context, width: i32, height: i32, simulation: &GameSimulation) {
    let width = width as f64;
    let height = height as f64;
    let scale = (width / simulation.map.width as f64).min(height / simulation.map.height as f64);
    let offset_x = (width - simulation.map.width as f64 * scale) / 2.0;
    let offset_y = (height - simulation.map.height as f64 * scale) / 2.0;

    context.set_source_rgb(0.025, 0.035, 0.042);
    context.paint().ok();
    context.save().ok();
    context.translate(offset_x, offset_y);
    context.scale(scale, scale);

    context.set_source_rgb(0.055, 0.075, 0.085);
    context.rectangle(
        0.0,
        0.0,
        simulation.map.width as f64,
        simulation.map.height as f64,
    );
    context.fill().ok();

    context.set_source_rgb(0.065, 0.085, 0.095);
    context.set_line_width(1.0);
    let mut x = 16.0;
    while x < simulation.map.width - 16.0 {
        context.move_to(x as f64, 16.0);
        context.line_to(x as f64, (simulation.map.height - 16.0) as f64);
        x += 64.0;
    }
    let mut y = 16.0;
    while y < simulation.map.height - 16.0 {
        context.move_to(16.0, y as f64);
        context.line_to((simulation.map.width - 16.0) as f64, y as f64);
        y += 64.0;
    }
    context.stroke().ok();

    context.set_source_rgb(0.18, 0.23, 0.27);
    for wall in &simulation.map.walls {
        context.rectangle(
            wall.min.x as f64,
            wall.min.y as f64,
            (wall.max.x - wall.min.x) as f64,
            (wall.max.y - wall.min.y) as f64,
        );
        context.fill().ok();
    }

    for projectile in &simulation.state.projectiles {
        context.set_source_rgb(1.0, 0.84, 0.22);
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
            continue;
        }
        let color = PLAYER_COLORS[tank.player_id.0 as usize % PLAYER_COLORS.len()];
        context.set_source_rgb(color.0, color.1, color.2);
        context.arc(
            tank.position.x as f64,
            tank.position.y as f64,
            simulation.config.tank_radius as f64,
            0.0,
            std::f64::consts::TAU,
        );
        context.fill().ok();

        let direction = tank.direction();
        context.set_line_width(7.0);
        context.set_line_cap(gtk4::cairo::LineCap::Round);
        context.move_to(tank.position.x as f64, tank.position.y as f64);
        context.line_to(
            (tank.position.x + direction.x * 22.0) as f64,
            (tank.position.y + direction.y * 22.0) as f64,
        );
        context.stroke().ok();

        if let Some(player) = simulation
            .state
            .players
            .iter()
            .find(|player| player.id == tank.player_id)
        {
            context.set_font_size(11.0);
            context.set_source_rgb(0.92, 0.94, 0.96);
            context.move_to(
                (tank.position.x - 22.0) as f64,
                (tank.position.y - 21.0) as f64,
            );
            context.show_text(&player.name).ok();
        }
    }

    context.restore().ok();

    let alive = simulation
        .state
        .tanks
        .iter()
        .filter(|tank| tank.alive)
        .count();
    context.set_source_rgb(0.93, 0.95, 0.97);
    context.select_font_face(
        "Sans",
        gtk4::cairo::FontSlant::Normal,
        gtk4::cairo::FontWeight::Bold,
    );
    context.set_font_size(17.0);
    context.move_to(18.0, 28.0);
    context
        .show_text(&format!("TANKRUSH  •  Survivors: {alive}"))
        .ok();

    if alive <= 1 && !simulation.state.tanks.is_empty() {
        let winner = simulation
            .state
            .tanks
            .iter()
            .find(|tank| tank.alive)
            .and_then(|tank| {
                simulation
                    .state
                    .players
                    .iter()
                    .find(|player| player.id == tank.player_id)
            });
        if let Some(player) = winner {
            context.set_font_size(24.0);
            context.move_to(24.0, height - 28.0);
            context.show_text(&format!("Winner: {}", player.name)).ok();
        }
    }
}

fn build_game_screen(
    stack: &Stack,
    player_count: usize,
    map_size: MapSize,
    names: Vec<String>,
    bindings: Vec<ControlBindings>,
) -> (Box, DrawingArea) {
    let config = GameConfig::default();
    let mut simulation = GameSimulation::new(GameMap::generate_seeded(map_size, arena_seed()), config);
    let spawn_points = simulation.map.spawn_points().to_vec();
    for index in 0..player_count {
        let player_id = simulation
            .state
            .add_player(names[index].clone(), None)
            .expect("local player count is capped at four");
        simulation.state.add_tank(player_id, spawn_points[index]);
    }

    let simulation = Rc::new(RefCell::new(simulation));
    let inputs = Rc::new(RefCell::new([TankInput::idle(); 4]));
    let fire_pending = Rc::new(RefCell::new([false; 4]));
    let fire_down = Rc::new(RefCell::new([false; 4]));
    let game_active = Rc::new(Cell::new(true));

    let root = Box::new(Orientation::Vertical, 0);
    let toolbar = Box::new(Orientation::Horizontal, 8);
    toolbar.set_margin_top(8);
    toolbar.set_margin_bottom(8);
    toolbar.set_margin_start(10);
    toolbar.set_margin_end(10);

    let title = Label::new(Some("TankRush  •  Battle Arena"));
    title.set_hexpand(true);
    title.set_halign(gtk4::Align::Start);
    let back = Button::with_label("Back to Menu");
    toolbar.append(&title);
    toolbar.append(&back);
    root.append(&toolbar);

    let drawing_area = DrawingArea::new();
    drawing_area.set_content_width(1100);
    drawing_area.set_content_height(700);
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    drawing_area.set_focusable(true);

    {
        let simulation = Rc::clone(&simulation);
        drawing_area.set_draw_func(move |_, context, width, height| {
            draw_game(context, width, height, &simulation.borrow());
        });
    }

    let key_controller = EventControllerKey::new();
    {
        let inputs = Rc::clone(&inputs);
        let fire_pending = Rc::clone(&fire_pending);
        let fire_down = Rc::clone(&fire_down);
        let bindings = bindings.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            let Some(name) = key.name() else {
                return Propagation::Proceed;
            };
            let name = name.to_string();
            let mut input = inputs.borrow_mut();
            let mut pending = fire_pending.borrow_mut();
            let mut down = fire_down.borrow_mut();
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
        let fire_down = Rc::clone(&fire_down);
        let bindings = bindings.clone();
        key_controller.connect_key_released(move |_, key, _, _| {
            let Some(name) = key.name() else {
                return;
            };
            let name = name.to_string();
            let mut input = inputs.borrow_mut();
            let mut down = fire_down.borrow_mut();
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

    let timer_active = Rc::clone(&game_active);
    let simulation_for_timer = Rc::clone(&simulation);
    let inputs_for_timer = Rc::clone(&inputs);
    let pending_for_timer = Rc::clone(&fire_pending);
    let drawing_for_timer = drawing_area.clone();
    glib::source::timeout_add_local(Duration::from_millis(16), move || {
        if !timer_active.get() {
            return ControlFlow::Break;
        }
        let mut input_snapshot = inputs_for_timer.borrow().to_vec();
        let mut pending = pending_for_timer.borrow_mut();
        for index in 0..player_count {
            if pending[index] {
                input_snapshot[index].fire = true;
                pending[index] = false;
            }
        }
        let pairs: Vec<(PlayerId, TankInput)> = input_snapshot
            .into_iter()
            .enumerate()
            .take(player_count)
            .map(|(index, input)| (PlayerId(index as u8), input))
            .collect();
        simulation_for_timer
            .borrow_mut()
            .advance(1.0 / 60.0, &pairs);
        drawing_for_timer.queue_draw();
        ControlFlow::Continue
    });

    let stack_for_back = stack.clone();
    let active_for_back = Rc::clone(&game_active);
    back.connect_clicked(move |_| {
        active_for_back.set(false);
        stack_for_back.set_visible_child_name("menu");
    });

    (root, drawing_area)
}

fn build_setup_screen(
    stack: &Stack,
    player_count: SpinButton,
    bindings: Rc<RefCell<Vec<ControlBindings>>>,
) -> Box {
    let root = Box::new(Orientation::Vertical, 12);
    root.set_margin_top(28);
    root.set_margin_bottom(28);
    root.set_margin_start(36);
    root.set_margin_end(36);
    root.set_focusable(true);

    let title = Label::new(Some("Local Battle Setup"));
    title.add_css_class("title-2");
    root.append(&title);
    root.append(&Label::new(Some(
        "TankTrouble-style local arena: 1–4 players, custom names and controls.",
    )));

    let players_adjustment = Adjustment::new(2.0, 1.0, 4.0, 1.0, 1.0, 0.0);
    player_count.set_adjustment(&players_adjustment);
    root.append(&Label::new(Some("Players")));
    root.append(&player_count);

    let maps = ComboBoxText::new();
    maps.append_text("Small Arena");
    maps.append_text("Medium Arena");
    maps.append_text("Large Arena");
    maps.append_text("Very Large Arena");
    maps.set_active(Some(1));
    root.append(&Label::new(Some("Arena size")));
    root.append(&maps);

    let setup_grid = Box::new(Orientation::Vertical, 7);
    let pending = Rc::new(RefCell::new(None::<(usize, usize, Button)>));
    let pending_label = Label::new(Some(
        "Choose a control button, then press a key. Escape cancels.",
    ));
    pending_label.add_css_class("dim-label");
    let mut name_entries = Vec::new();

    for player in 0..4 {
        let row = Box::new(Orientation::Horizontal, 7);
        let name = Entry::new();
        name.set_text(&format!("Player {}", player + 1));
        name.set_width_chars(14);
        name_entries.push(name.clone());
        row.append(&Label::new(Some(&format!("P{}", player + 1))));
        row.append(&name);

        let actions = ["FWD", "BACK", "LEFT", "RIGHT", "FIRE"];
        for action in 0..5 {
            let button = Button::with_label(&key_label(&bindings.borrow()[player].keys[action]));
            button.set_tooltip_text(Some(actions[action]));
            let pending = Rc::clone(&pending);
            let pending_label = pending_label.clone();
            button.connect_clicked(move |button| {
                *pending.borrow_mut() = Some((player, action, button.clone()));
                pending_label.set_text(&format!(
                    "P{} {}: press the new key…",
                    player + 1,
                    actions[action]
                ));
            });
            row.append(&button);
        }
        setup_grid.append(&row);
    }

    root.append(&Label::new(Some("Player names and controls")));
    root.append(&setup_grid);
    root.append(&pending_label);

    let key_controller = EventControllerKey::new();
    {
        let pending = Rc::clone(&pending);
        let bindings = Rc::clone(&bindings);
        let pending_label = pending_label.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            let Some(name) = key.name() else {
                return Propagation::Stop;
            };
            let name = name.to_string();
            if name == "Escape" {
                *pending.borrow_mut() = None;
                pending_label.set_text("Control assignment cancelled.");
                return Propagation::Stop;
            }
            if let Some((player, action, button)) = pending.borrow_mut().take() {
                bindings.borrow_mut()[player].keys[action] = name.clone();
                button.set_label(&key_label(&name));
                pending_label.set_text(&format!(
                    "P{} control changed to {}.",
                    player + 1,
                    key_label(&name)
                ));
            }
            Propagation::Stop
        });
    }
    root.add_controller(key_controller);

    let actions = Box::new(Orientation::Horizontal, 8);
    let back = Button::with_label("Back");
    let start = Button::with_label("Start Match");
    actions.append(&back);
    actions.append(&start);
    root.append(&actions);

    let stack_for_back = stack.clone();
    back.connect_clicked(move |_| stack_for_back.set_visible_child_name("menu"));

    let stack_for_start = stack.clone();
    let bindings_for_start = Rc::clone(&bindings);
    let player_count_for_start = player_count.clone();
    start.connect_clicked(move |_| {
        let count = player_count_for_start.value_as_int().clamp(1, 4) as usize;
        let map_size = match maps.active_text().as_deref() {
            Some("Medium Arena") => MapSize::Medium,
            Some("Large Arena") => MapSize::Large,
            Some("Very Large Arena") => MapSize::VeryLarge,
            _ => MapSize::Small,
        };
        let names = name_entries
            .iter()
            .take(count)
            .enumerate()
            .map(|(index, entry)| {
                let name = entry.text().trim().to_string();
                if name.is_empty() {
                    format!("Player {}", index + 1)
                } else {
                    name.chars().take(18).collect()
                }
            })
            .collect::<Vec<_>>();

        let (game, drawing_area) = build_game_screen(
            &stack_for_start,
            count,
            map_size,
            names,
            bindings_for_start.borrow().clone(),
        );
        if let Some(old) = stack_for_start.child_by_name("game") {
            stack_for_start.remove(&old);
        }
        stack_for_start.add_named(&game, Some("game"));
        stack_for_start.set_visible_child_name("game");
        drawing_area.grab_focus();
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
    let subtitle = Label::new(Some("Tank arena • ricochet combat • 1–4 local players"));
    subtitle.add_css_class("dim-label");
    content.append(&title);
    content.append(&subtitle);

    let single_player = Button::with_label("Single Player / Local Battle");
    let lan = Button::with_label("LAN Multiplayer");
    let settings = Button::with_label("Settings");
    let quit = Button::with_label("Quit");
    for button in [&single_player, &lan, &settings, &quit] {
        button.set_width_request(280);
    }
    content.append(&single_player);
    content.append(&lan);
    content.append(&settings);
    content.append(&quit);

    let stack_for_single = stack.clone();
    single_player.connect_clicked(move |_| {
        stack_for_single.set_visible_child_name("setup");
    });

    let stack_for_lan = stack.clone();
    lan.connect_clicked(move |_| {
        stack_for_lan.set_visible_child_name("lan");
    });

    let stack_for_settings = stack.clone();
    let audio_for_settings = Rc::clone(&audio);
    settings.connect_clicked(move |_| {
        if let Some(page) = stack_for_settings.child_by_name("settings") {
            stack_for_settings.remove(&page);
        }
        let page = Box::new(Orientation::Vertical, 12);
        page.set_margin_top(32);
        page.set_margin_start(36);
        page.set_margin_end(36);
        page.set_margin_bottom(32);
        page.append(&Label::new(Some("Settings")));

        let music = CheckButton::with_label("Music");
        let effects = CheckButton::with_label("Sound effects");
        music.set_active(audio_for_settings.borrow().music_enabled);
        effects.set_active(audio_for_settings.borrow().sound_effects_enabled);
        {
            let audio = Rc::clone(&audio_for_settings);
            music.connect_toggled(move |button| {
                audio.borrow_mut().music_enabled = button.is_active();
            });
        }
        {
            let audio = Rc::clone(&audio_for_settings);
            effects.connect_toggled(move |button| {
                audio.borrow_mut().sound_effects_enabled = button.is_active();
            });
        }
        page.append(&music);
        page.append(&effects);

        let back = Button::with_label("Back");
        let stack = stack_for_settings.clone();
        back.connect_clicked(move |_| stack.set_visible_child_name("menu"));
        page.append(&back);

        stack_for_settings.add_named(&page, Some("settings"));
        stack_for_settings.set_visible_child_name("settings");
    });

    let stack_for_quit = stack.clone();
    quit.connect_clicked(move |_| {
        if let Some(root) = stack_for_quit.root() {
            root.downcast::<ApplicationWindow>()
                .ok()
                .map(|window| window.close());
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
        "Network gameplay is being integrated after the local arena core.",
    )));
    let lan_back = Button::with_label("Back");
    let stack_for_lan = stack.clone();
    lan_back.connect_clicked(move |_| stack_for_lan.set_visible_child_name("menu"));
    lan.append(&lan_back);
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
