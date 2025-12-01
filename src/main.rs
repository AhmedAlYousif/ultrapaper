use gtk::gdk::Display;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, CssProvider, glib};

use crate::hypr::hyprctl;
use crate::hypr::hyprpaper::HyprpaperConfig;
use crate::state::{set_config, set_monitors};
use crate::windows::main_window::MainWindow;
mod hypr;
mod widgets;
mod windows;
mod state;

const APP_ID: &str = "sa.ahmedy.ultrapaper";

fn main() -> glib::ExitCode {
    let hyprpaper_config = match HyprpaperConfig::new(HyprpaperConfig::get_default_config_path()) {
        Ok(config) => config,
        Err(err) => panic!("Error loading config: {}", err),
    };

    set_config(hyprpaper_config);

    let monitors = hyprctl::get_monitors();
    set_monitors(monitors);

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);

    app.run()
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_string(include_str!("style.css"));

    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ultrapaper")
        .build();

    let main_window = MainWindow::new(&window);

    window.set_child(Some(&main_window.widget));

    window.present();
}
