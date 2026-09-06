mod game;
mod lobby;
#[allow(dead_code)]
mod net_session;
mod network;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use game::{GameConfig, GameMap, GameSimulation, MapSize, PlayerId, TankInput};
use glib::{ControlFlow, Propagation};
use gtk4::cairo::Context;
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Application, ApplicationWindow, Box, Button, CheckButton, ComboBoxText, DrawingArea,
    Entry, EventControllerKey, Label, Orientation, SpinButton,
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
        ControlBindings::new(["w", "s", "a", "d", "space"]),
        ControlBindings::new(["Up", "Down", "Left", "Right", "Return"]),
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

fn bind_control_button(
    parent: &ApplicationWindow,
    button: &Button,
    bindings: &Rc<RefCell<Vec<ControlBindings>>>,
    player: usize,
    action: usize,
) {
    let parent = parent.clone();
    let bindings = Rc::clone(bindings);
    let button_for_dialog = button.clone();
    button.connect_clicked(move |_| {
        let dialog = ApplicationWindow::builder()
            .transient_for(&parent)
            .modal(true)
            .title("Change Control")
            .default_width(340)
            .default_height(150)
            .build();
        let content = Box::new(Orientation::Vertical, 10);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.append(&Label::new(Some("Press the key you want to assign.")));
        content.append(&Label::new(Some("Escape cancels the assignment.")));
        dialog.set_child(Some(&content));

        let controller = EventControllerKey::new();
        let dialog_for_key = dialog.clone();
        let bindings_for_key = Rc::clone(&bindings);
        let button_for_key = button_for_dialog.clone();
        controller.connect_key_pressed(move |_, key, _, _| {
            let Some(name) = key.name() else {
                return Propagation::Stop;
            };
            let name = name.to_string();
            if name == "Escape" {
                dialog_for_key.close();
                return Propagation::Stop;
            }
            bindings_for_key.borrow_mut()[player].keys[action] = name.clone();
            button_for_key.set_label(&key_label(&name));
            dialog_for_key.close();
            Propagation::Stop
        });
        dialog.add_controller(controller);
        dialog.present();
    });
}

fn build_settings_window(
    parent: &ApplicationWindow,
    audio: Rc<RefCell<game::config::AudioSettings>>,
) {
    let music = CheckButton::with_label("Music");
    let effects = CheckButton::with_label("Sound effects");
    music.set_active(audio.borrow().music_enabled);
    effects.set_active(audio.borrow().sound_effects_enabled);

    {
        let audio = Rc::clone(&audio);
        music.connect_toggled(move |button| {
            audio.borrow_mut().music_enabled = button.is_active();
        });
    }
    {
        let audio = Rc::clone(&audio);
        effects.connect_toggled(move |button| {
            audio.borrow_mut().sound_effects_enabled = button.is_active();
        });
    }

    let content = Box::new(Orientation::Vertical, 12);
    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);
    content.append(&Label::new(Some("Audio")));
    content.append(&music);
    content.append(&effects);

    let window = ApplicationWindow::builder()
        .transient_for(parent)
        .modal(true)
        .title("TankRush Settings")
        .default_width(320)
        .default_height(220)
        .child(&content)
        .build();
    window.present();
}

fn build_single_player_window(parent: &ApplicationWindow) {
    let players_adjustment = Adjustment::new(1.0, 1.0, 4.0, 1.0, 1.0, 0.0);
    let players = SpinButton::new(Some(&players_adjustment), 1.0, 0);
    let maps = ComboBoxText::new();
    maps.append_text("Small Maze");
    maps.append_text("Medium Maze");
    maps.append_text("Large Maze");
    maps.append_text("Very Large Maze");
    maps.set_active(Some(0));

    let bindings = Rc::new(RefCell::new(default_controls()));
    let mut name_entries = Vec::new();
    let mut control_rows = Vec::new();
    for player in 0..4 {
        let row = Box::new(Orientation::Horizontal, 6);
        let name = Entry::new();
        name.set_text(&format!("Player {}", player + 1));
        name.set_width_chars(12);
        row.append(&Label::new(Some(&format!("P{}", player + 1))));
        row.append(&name);
        let labels = ["↑", "↓", "←", "→", "FIRE"];
        for action in 0..5 {
            let button = Button::with_label(&key_label(&bindings.borrow()[player].keys[action]));
            button.set_tooltip_text(Some(labels[action]));
            bind_control_button(parent, &button, &bindings, player, action);
            row.append(&button);
        }
        name_entries.push(name);
        control_rows.push(row);
    }

    let content = Box::new(Orientation::Vertical, 10);
    content.set_margin_top(22);
    content.set_margin_bottom(22);
    content.set_margin_start(22);
    content.set_margin_end(22);
    content.append(&Label::new(Some("Single Player / Local Battle")));
    content.append(&Label::new(Some("Choose the number of tanks, names and controls.")));
    content.append(&Label::new(Some("Players (1–4)")));
    content.append(&players);
    content.append(&Label::new(Some("Map")));
    content.append(&maps);
    content.append(&Label::new(Some("Player setup")));
    for row in control_rows {
        content.append(&row);
    }

    let start = Button::with_label("Start Match");
    let cancel = Button::with_label("Back");
    let actions = Box::new(Orientation::Horizontal, 8);
    actions.append(&cancel);
    actions.append(&start);
    content.append(&actions);

    let window = ApplicationWindow::builder()
        .transient_for(parent)
        .modal(true)
        .title("TankRush — Match Setup")
        .default_width(780)
        .default_height(520)
        .child(&content)
        .build();

    let parent_for_start = parent.clone();
    let window_for_start = window.clone();
    start.connect_clicked(move |_| {
        let player_count = players.value_as_int().clamp(1, 4) as usize;
        let map_size = match maps.active_text().as_deref() {
            Some("Medium Maze") => MapSize::Medium,
            Some("Large Maze") => MapSize::Large,
            Some("Very Large Maze") => MapSize::VeryLarge,
            _ => MapSize::Small,
        };
        let names = name_entries
            .iter()
            .take(player_count)
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
        let selected_bindings = bindings.borrow().clone();
        window_for_start.close();
        build_game_window(
            &parent_for_start,
            player_count,
            map_size,
            names,
            selected_bindings,
        );
    });

    let window_for_cancel = window.clone();
    cancel.connect_clicked(move |_| window_for_cancel.close());
    window.present();
}

fn apply_key(
    player: usize,
    name: &str,
    pressed: bool,
    bindings: &[ControlBindings],
    input: &mut [TankInput; 4],
    fire_pending: &mut [bool; 4],
    fire_down: &mut [bool; 4],
) -> bool {
    if player >= bindings.len() {
        return false;
    }
    let Some(action) = bindings[player].keys.iter().position(|key| key == name) else {
        return false;
    };
    match action {
        0 => input[player].forward = pressed,
        1 => input[player].backward = pressed,
        2 => input[player].left = pressed,
        3 => input[player].right = pressed,
        4 => {
            if pressed && !fire_down[player] {
                fire_pending[player] = true;
            }
            fire_down[player] = pressed;
        }
        _ => unreachable!(),
    }
    true
}

fn draw_game(context: &Context, width: i32, height: i32, simulation: &GameSimulation) {
    let width = width as f64;
    let height = height as f64;
    let scale = (width / simulation.map.width as f64).min(height / simulation.map.height as f64);
    let offset_x = (width - simulation.map.width as f64 * scale) / 2.0;
    let offset_y = (height - simulation.map.height as f64 * scale) / 2.0;

    context.set_source_rgb(0.035, 0.045, 0.055);
    context.paint().ok();
    context.save().ok();
    context.translate(offset_x, offset_y);
    context.scale(scale, scale);

    context.set_source_rgb(0.055, 0.075, 0.085);
    context.rectangle(0.0, 0.0, simulation.map.width as f64, simulation.map.height as f64);
    context.fill().ok();

    context.set_source_rgb(0.16, 0.20, 0.23);
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
        context.set_source_rgb(1.0, 0.85, 0.2);
        context.arc(
            projectile.position.x as f64,
            projectile.position.y as f64,
            simulation.config.projectile_radius as f64,
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
        context.set_line_width(5.0);
        context.move_to(tank.position.x as f64, tank.position.y as f64);
        context.line_to(
            (tank.position.x + direction.x * 20.0) as f64,
            (tank.position.y + direction.y * 20.0) as f64,
        );
        context.stroke().ok();
    }

    context.restore().ok();
    context.set_source_rgb(0.9, 0.92, 0.95);
    context.select_font_face(
        "Sans",
        gtk4::cairo::FontSlant::Normal,
        gtk4::cairo::FontWeight::Bold,
    );
    context.set_font_size(16.0);
    context.move_to(16.0, 26.0);
    let alive = simulation.state.tanks.iter().filter(|tank| tank.alive).count();
    context
        .show_text(&format!("TANKRUSH  •  Survivors: {alive}"))
        .ok();
}

fn build_game_window(
    parent: &ApplicationWindow,
    player_count: usize,
    map_size: MapSize,
    names: Vec<String>,
    bindings: Vec<ControlBindings>,
) {
    let config = GameConfig::default();
    let mut simulation = GameSimulation::new(GameMap::generate(map_size), config);
    let spawn_points = simulation.map.spawn_points().to_vec();
    for index in 0..player_count {
        let player_id = simulation
            .state
            .add_player(names[index].clone(), None)
            .expect("player count is capped at four");
        simulation.state.add_tank(player_id, spawn_points[index]);
    }

    let simulation = Rc::new(RefCell::new(simulation));
    let inputs = Rc::new(RefCell::new([TankInput::idle(); 4]));
    let fire_pending = Rc::new(RefCell::new([false; 4]));
    let fire_down = Rc::new(RefCell::new([false; 4]));
    let drawing_area = DrawingArea::new();
    drawing_area.set_content_width(960);
    drawing_area.set_content_height(640);
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
                if apply_key(player, &name, true, &bindings, &mut input, &mut pending, &mut down) {
                    return Propagation::Stop;
                }
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
            let mut pending = [false; 4];
            let mut down = fire_down.borrow_mut();
            for player in 0..player_count {
                apply_key(player, &name, false, &bindings, &mut input, &mut pending, &mut down);
            }
        });
    }
    drawing_area.add_controller(key_controller);

    let game_window = ApplicationWindow::builder()
        .transient_for(parent)
        .modal(true)
        .title("TankRush — Match")
        .default_width(1000)
        .default_height(700)
        .child(&drawing_area)
        .build();

    let simulation_for_timer = Rc::clone(&simulation);
    let inputs_for_timer = Rc::clone(&inputs);
    let pending_for_timer = Rc::clone(&fire_pending);
    let drawing_for_timer = drawing_area.clone();
    glib::source::timeout_add_local(Duration::from_millis(16), move || {
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

    game_window.present();
    drawing_area.grab_focus();
}

fn build_ui(app: &Application) {
    let audio_settings = Rc::new(RefCell::new(game::config::AudioSettings::default()));

    let title = Label::new(Some("TANKRUSH"));
    title.add_css_class("title-1");
    let subtitle = Label::new(Some("Rust + GTK4 tank arena"));
    subtitle.add_css_class("dim-label");

    let single_player = Button::with_label("Single Player");
    let multiplayer = Button::with_label("LAN Multiplayer");
    let settings = Button::with_label("Settings");
    let quit = Button::with_label("Quit");

    let content = Box::new(Orientation::Vertical, 12);
    content.set_margin_top(48);
    content.set_margin_bottom(48);
    content.set_margin_start(48);
    content.set_margin_end(48);
    content.set_halign(gtk4::Align::Center);
    content.set_valign(gtk4::Align::Center);
    content.append(&title);
    content.append(&subtitle);
    content.append(&single_player);
    content.append(&multiplayer);
    content.append(&settings);
    content.append(&quit);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("TankRush")
        .default_width(720)
        .default_height(540)
        .child(&content)
        .build();

    let parent = window.clone();
    single_player.connect_clicked(move |_| build_single_player_window(&parent));

    let parent = window.clone();
    multiplayer.connect_clicked(move |_| {
        let dialog = ApplicationWindow::builder()
            .transient_for(&parent)
            .modal(true)
            .title("LAN Multiplayer")
            .default_width(420)
            .default_height(220)
            .child(&Label::new(Some(&format!(
                "LAN protocol v{} is ready.\nLobby supports up to {} players.\nAuthoritative LAN session backend is available.",
                network::PROTOCOL_VERSION,
                lobby::MAX_LOBBY_PLAYERS
            ))))
            .build();
        dialog.present();
    });

    let parent = window.clone();
    let audio = Rc::clone(&audio_settings);
    settings.connect_clicked(move |_| build_settings_window(&parent, Rc::clone(&audio)));

    let app_weak = app.downgrade();
    quit.connect_clicked(move |_| {
        if let Some(app) = app_weak.upgrade() {
            app.quit();
        }
    });

    window.present();
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}
