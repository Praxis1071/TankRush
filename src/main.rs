mod game;
mod network;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use game::{GameConfig, GameMap, GameSimulation, MapSize, PlayerId, TankInput, Vec2};
use glib::{ControlFlow, Propagation};
use gtk4::cairo::Context;
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Application, ApplicationWindow, Box, Button, CheckButton, ComboBoxText,
    DrawingArea, EventControllerKey, Label, Orientation, SpinButton,
};

const APP_ID: &str = "io.github.praxis1071.TankRush";
const PLAYER_COLORS: [(f64, f64, f64); 4] = [
    (0.20, 0.75, 1.0),
    (1.0, 0.35, 0.35),
    (0.35, 1.0, 0.45),
    (1.0, 0.80, 0.25),
];

fn build_settings_window(parent: &ApplicationWindow, audio: Rc<RefCell<game::config::AudioSettings>>) {
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
    maps.append_text("Small");
    maps.append_text("Medium");
    maps.append_text("Large");
    maps.append_text("Very Large");
    maps.set_active(Some(0));

    let content = Box::new(Orientation::Vertical, 12);
    content.set_margin_top(28);
    content.set_margin_bottom(28);
    content.set_margin_start(28);
    content.set_margin_end(28);
    content.append(&Label::new(Some("Single Player")));
    content.append(&Label::new(Some("Players (1–4)")));
    content.append(&players);
    content.append(&Label::new(Some("Map size")));
    content.append(&maps);

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
        .default_width(420)
        .default_height(360)
        .child(&content)
        .build();

    let parent_for_start = parent.clone();
    let window_for_start = window.clone();
    start.connect_clicked(move |_| {
        let player_count = players.value_as_int().clamp(1, 4) as usize;
        let map_size = match maps.active_text().as_deref() {
            Some("Medium") => MapSize::Medium,
            Some("Large") => MapSize::Large,
            Some("Very Large") => MapSize::VeryLarge,
            _ => MapSize::Small,
        };
        window_for_start.close();
        build_game_window(&parent_for_start, player_count, map_size);
    });

    let window_for_cancel = window.clone();
    cancel.connect_clicked(move |_| window_for_cancel.close());
    window.present();
}

fn player_input(player: usize, name: &str, pressed: bool, input: &mut [TankInput; 4]) -> bool {
    let (index, action) = match (player, name) {
        (0, "w") => (0, 0),
        (0, "s") => (0, 1),
        (0, "a") => (0, 2),
        (0, "d") => (0, 3),
        (0, "space") => (0, 4),
        (1, "Up") => (1, 0),
        (1, "Down") => (1, 1),
        (1, "Left") => (1, 2),
        (1, "Right") => (1, 3),
        (1, "Return") => (1, 4),
        (2, "i") => (2, 0),
        (2, "k") => (2, 1),
        (2, "j") => (2, 2),
        (2, "l") => (2, 3),
        (2, "o") => (2, 4),
        (3, "8") => (3, 0),
        (3, "5") => (3, 1),
        (3, "4") => (3, 2),
        (3, "6") => (3, 3),
        (3, "0") => (3, 4),
        _ => return false,
    };

    match action {
        0 => input[index].forward = pressed,
        1 => input[index].backward = pressed,
        2 => input[index].left = pressed,
        3 => input[index].right = pressed,
        4 => input[index].fire = pressed,
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

    context.set_source_rgb(0.07, 0.10, 0.12);
    context.rectangle(0.0, 0.0, simulation.map.width as f64, simulation.map.height as f64);
    context.fill().ok();

    context.set_source_rgb(0.18, 0.22, 0.25);
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

fn build_game_window(parent: &ApplicationWindow, player_count: usize, map_size: MapSize) {
    let config = GameConfig::default();
    let mut simulation = GameSimulation::new(GameMap::generate(map_size), config);
    let spawn_points = simulation.map.spawn_points().to_vec();
    for index in 0..player_count {
        let player_id = simulation
            .state
            .add_player(format!("P{}", index + 1), None)
            .expect("player count is capped at four");
        simulation.state.add_tank(player_id, spawn_points[index]);
    }

    let simulation = Rc::new(RefCell::new(simulation));
    let inputs = Rc::new(RefCell::new([TankInput::idle(); 4]));
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
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if let Some(name) = key.name() {
                let name = name.to_string();
                let mut input = inputs.borrow_mut();
                for player in 0..4 {
                    if player_input(player, &name, true, &mut input) {
                        return Propagation::Stop;
                    }
                }
            }
            Propagation::Proceed
        });
    }
    {
        let inputs = Rc::clone(&inputs);
        key_controller.connect_key_released(move |_, key, _, _| {
            if let Some(name) = key.name() {
                let name = name.to_string();
                let mut input = inputs.borrow_mut();
                for player in 0..4 {
                    if player_input(player, &name, false, &mut input) {
                        return;
                    }
                }
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
    let drawing_for_timer = drawing_area.clone();
    glib::source::timeout_add_local(Duration::from_millis(16), move || {
        let input_snapshot = inputs_for_timer.borrow().to_vec();
        let pairs: Vec<(PlayerId, TankInput)> = input_snapshot
            .into_iter()
            .enumerate()
            .take(player_count)
            .map(|(index, input)| (PlayerId(index as u8), input))
            .collect();
        let events = simulation_for_timer
            .borrow_mut()
            .advance(1.0 / 60.0, &pairs);
        if events
            .iter()
            .any(|event| matches!(event.kind, game::SimulationEventKind::TankDestroyed))
        {
            drawing_for_timer.queue_draw();
        } else {
            drawing_for_timer.queue_draw();
        }
        for input in inputs_for_timer.borrow_mut().iter_mut() {
            input.fire = false;
        }
        ControlFlow::Continue
    });

    game_window.connect_close_request(|_| Propagation::Proceed);
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
                "LAN protocol v{} is ready.\nLobby and network gameplay are next in Stage 9–10.",
                network::PROTOCOL_VERSION
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
