mod game;

use std::cell::RefCell;
use std::rc::Rc;

use game::config::AudioSettings;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, CheckButton, Label, Orientation};

const APP_ID: &str = "io.github.praxis1071.TankRush";

fn build_settings_window(parent: &ApplicationWindow, audio: Rc<RefCell<AudioSettings>>) {
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

fn build_ui(app: &Application) {
    let audio_settings = Rc::new(RefCell::new(AudioSettings::default()));

    let title = Label::new(Some("TANKRUSH"));
    title.add_css_class("title-1");

    let subtitle = Label::new(Some("Rust + GTK4 tank arena"));
    subtitle.add_css_class("dim-label");

    let single_player = Button::with_label("Single Player");
    let multiplayer = Button::with_label("Multiplayer");
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
    let audio = Rc::clone(&audio_settings);
    settings.connect_clicked(move |_| build_settings_window(&parent, Rc::clone(&audio)));

    let app_weak = app.downgrade();
    quit.connect_clicked(move |_| {
        if let Some(app) = app_weak.upgrade() {
            app.quit();
        }
    });

    let _ = (single_player, multiplayer);
    window.present();
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}
