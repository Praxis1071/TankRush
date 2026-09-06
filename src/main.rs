use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Button, Label, Orientation};

const APP_ID: &str = "io.github.praxis1071.TankRush";

fn build_ui(app: &Application) {
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

    let app_weak = app.downgrade();
    quit.connect_clicked(move |_| {
        if let Some(app) = app_weak.upgrade() {
            app.quit();
        }
    });

    // These controls are intentionally wired in later stages. The initial
    // scaffold keeps the main menu visible while the game architecture grows.
    let _ = (single_player, multiplayer, settings);

    window.present();
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}
