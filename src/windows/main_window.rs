use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;

use gio::glib::clone;
use gtk::prelude::*;
use gtk::{AlertDialog, Align, ApplicationWindow, Box, Button, Label, Orientation, ToggleButton};

use crate::hypr::hyprpaper::ConfigEntry;
use crate::state::{
    get_monitors, get_selected_monitor, get_wallpaper_for_monitor, has_empty_monitor_name,
    has_monitor, has_more_than_one_monitors, has_wallpapers, save_config, set_global_entry,
    set_selected_monitor, update_wallpaper_for_monitor,
};
use crate::widgets::images_grid_view::ImagesGridView;
use crate::widgets::settings_panel::SettingsPanel;
use crate::widgets::wallpaper_options_panel::WallpaperOptionsPanel;

pub struct MainWindow {
    pub widget: Box,
}

impl MainWindow {
    pub fn new(window: &ApplicationWindow) -> Self {
        let root = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        root.add_css_class("image-browser-root");

        let current_path: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

        let options_panel = Rc::new(WallpaperOptionsPanel::new(window));
        let settings_panel = SettingsPanel::new();

        let images_grid_view = Rc::new({
            let options_panel = options_panel.clone();
            let current_path = current_path.clone();
            ImagesGridView::new(move |path: &str| {
                if !options_panel.is_dir_mode() {
                    *current_path.borrow_mut() = path.to_string();
                }
            })
        });

        if has_more_than_one_monitors() {
            let monitors_box = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(5)
                .build();
            monitors_box.add_css_class("image-browser-header");
            monitors_box.append(&Label::builder().label("Monitor:").build());

            let all_toggle = ToggleButton::builder().label("All").active(false).build();

            {
                let options_panel = options_panel.clone();
                let current_path = current_path.clone();
                let images_grid_view = images_grid_view.clone();
                all_toggle.connect_toggled(clone!(move |tg| {
                    if tg.is_active() {
                        set_selected_monitor("".to_string());
                        let entry = get_wallpaper_for_monitor("");
                        options_panel.load(entry.as_ref());
                        let path = entry
                            .as_ref()
                            .filter(|e| {
                                !e.path.is_empty() && !std::path::Path::new(&e.path).is_dir()
                            })
                            .map(|e| e.path.clone())
                            .unwrap_or_default();
                        *current_path.borrow_mut() = path.clone();
                        let dir = options_panel.get_dir();
                        if !dir.is_empty() {
                            *images_grid_view.selected_path.borrow_mut() = path;
                            load_images_into_grid_raw(&dir, &images_grid_view.images_path_list);
                        }
                    }
                }));
            }
            monitors_box.append(&all_toggle);

            let mut active_toggle: Option<ToggleButton> = None;

            for monitor in get_monitors() {
                let toggle = ToggleButton::builder()
                    .label(&monitor)
                    .group(&all_toggle)
                    .active(false)
                    .build();
                let options_panel = options_panel.clone();
                let monitor_clone = monitor.clone();
                let current_path = current_path.clone();
                let images_grid_view = images_grid_view.clone();
                toggle.connect_toggled(move |tg| {
                    if tg.is_active() {
                        set_selected_monitor(monitor_clone.clone());
                        let entry = get_wallpaper_for_monitor(&monitor_clone);
                        options_panel.load(entry.as_ref());
                        let path = entry
                            .as_ref()
                            .filter(|e| {
                                !e.path.is_empty() && !std::path::Path::new(&e.path).is_dir()
                            })
                            .map(|e| e.path.clone())
                            .unwrap_or_default();
                        *current_path.borrow_mut() = path.clone();
                        let dir = options_panel.get_dir();
                        if !dir.is_empty() {
                            *images_grid_view.selected_path.borrow_mut() = path;
                            load_images_into_grid_raw(&dir, &images_grid_view.images_path_list);
                        }
                    }
                });
                monitors_box.append(&toggle);
                if !has_empty_monitor_name() && has_monitor(monitor.clone()) {
                    active_toggle = Some(toggle);
                }
            }

            if let Some(t) = active_toggle {
                t.set_active(true);
            } else {
                all_toggle.set_active(true);
            }

            root.append(&monitors_box);
        } else {
            let initial_monitor = get_selected_monitor();
            let initial_entry = get_wallpaper_for_monitor(&initial_monitor);
            options_panel.load(initial_entry.as_ref());
            let initial_path = initial_entry
                .as_ref()
                .filter(|e| !e.path.is_empty() && !std::path::Path::new(&e.path).is_dir())
                .map(|e| e.path.clone())
                .unwrap_or_default();
            *current_path.borrow_mut() = initial_path.clone();
            if has_wallpapers() {
                let dir = options_panel.get_dir();
                if !dir.is_empty() {
                    *images_grid_view.selected_path.borrow_mut() = initial_path.clone();
                    load_images_into_grid(&dir, &images_grid_view);
                }
            }
        }

        root.append(&options_panel.widget);
        root.append(&images_grid_view.widget);
        root.append(&settings_panel.widget);

        {
            let images_grid_view = images_grid_view.clone();
            options_panel.connect_dir_changed(move |new_dir| {
                images_grid_view.set_selected("");
                load_images_into_grid_raw(new_dir, &images_grid_view.images_path_list);
            });
        }

        let apply_btn = Button::builder().label("Apply").halign(Align::End).build();
        apply_btn.add_css_class("apply-btn");

        apply_btn.connect_clicked(clone!(
            #[weak]
            window,
            #[strong]
            options_panel,
            #[strong]
            current_path,
            move |_| {
                let monitor = get_selected_monitor();
                let current = current_path.borrow().clone();
                let entry = options_panel.build_entry(&monitor, &current);
                let is_dir_mode = options_panel.is_dir_mode();

                let old_entry = get_wallpaper_for_monitor(&monitor);
                let path_only_changed = old_entry.as_ref().map_or(false, |old| {
                    !is_dir_mode
                        && old.path != entry.path
                        && old.timeout.is_none()
                        && old.order.is_none()
                        && old.recursive.is_none()
                });

                update_wallpaper_for_monitor(&monitor, entry.clone());

                set_global_entry(ConfigEntry::Splash(settings_panel.get_splash()));
                set_global_entry(ConfigEntry::SplashOffset(
                    settings_panel.get_splash_offset(),
                ));
                set_global_entry(ConfigEntry::SplashOpacity(
                    settings_panel.get_splash_opacity(),
                ));
                set_global_entry(ConfigEntry::Ipc(settings_panel.get_ipc()));

                save_config();

                if path_only_changed && !entry.path.is_empty() {
                    let _ = Command::new("sh")
                        .arg("-c")
                        .arg(format!("hyprctl hyprpaper preload {}", entry.path))
                        .output();
                    let _ = Command::new("sh")
                        .arg("-c")
                        .arg(format!(
                            "hyprctl hyprpaper wallpaper {},{}",
                            monitor, entry.path
                        ))
                        .output();
                    let _ = Command::new("sh")
                        .arg("-c")
                        .arg("hyprctl hyprpaper unload unused")
                        .output();
                } else {
                    let dialog = AlertDialog::builder()
                        .message("Restart required")
                        .detail("Hyprpaper needs to restart to apply these changes. Restart now?")
                        .buttons(["Cancel", "Restart"])
                        .default_button(1)
                        .cancel_button(0)
                        .build();
                    dialog.choose(Some(&window), gio::Cancellable::NONE, move |res| {
                        if let Ok(idx) = res {
                            if idx == 1 {
                                let _ = Command::new("sh")
                                    .arg("-c")
                                    .arg("pkill hyprpaper && hyprctl dispatch exec hyprpaper")
                                    .output();
                            }
                        }
                    });
                }
            }
        ));

        root.append(&apply_btn);

        Self { widget: root }
    }
}

fn load_images_into_grid(dir: &str, grid: &ImagesGridView) {
    load_images_into_grid_raw(dir, &grid.images_path_list);
}

fn load_images_into_grid_raw(dir: &str, list: &gtk::StringList) {
    list.splice(0, list.n_items(), &[] as &[&str]);
    let entries = read_image_entries(dir);
    for path in entries {
        list.append(path.as_str());
    }
}

fn read_image_entries(path: &str) -> Vec<String> {
    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !metadata.is_file() {
            continue;
        }
        let file_name = entry.file_name();
        let name = match file_name.to_str() {
            Some(n) => n,
            None => continue,
        };
        let supported = ["jpg", "jpeg", "png", "bmp", "webp"];
        let ok = name
            .rfind('.')
            .map(|i| supported.contains(&name[i + 1..].to_lowercase().as_str()))
            .unwrap_or(false);
        if ok {
            files.push(format!("{}/{}", path, name));
        }
    }
    files
}
