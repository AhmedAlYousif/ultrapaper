use std::fs;

use gio::Cancellable;
use gio::glib::clone;
use gtk::{ApplicationWindow, Box, Button, FileDialog, Label, Orientation, StringList};
use gtk::{ToggleButton, prelude::*};

use crate::hypr::hyprctl::set_wallpaper;
use crate::state::{
    get_first_wallpaper_path, get_monitors, has_empty_monitor_name, has_monitor, has_more_than_one_monitors, has_wallpapers, set_selected_monitor
};
use crate::widgets::images_grid_view::ImagesGridView;

pub struct MainWindow {
    pub widget: Box,
}

impl MainWindow {
    pub fn new(window: &ApplicationWindow) -> Self {
        let images_grid_view = ImagesGridView::new(|path: &str| {
            set_wallpaper(path.to_owned());
        });

        let main_box = Box::builder()
            .margin_top(12)
            .margin_bottom(12)
            .margin_top(12)
            .margin_end(12)
            .orientation(Orientation::Vertical)
            .build();
        main_box.add_css_class("image-browser-root");

        let header_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(5)
            .build();
        header_box.add_css_class("image-browser-header");

        if has_more_than_one_monitors() {
            let monitors_box = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(5)
                .build();
            monitors_box.append(&Label::builder().label("Monitors: ").build());
            let all_monitors_toggle_button = ToggleButton::builder()
                .label("All")
                .active(has_empty_monitor_name())
                .build();
            all_monitors_toggle_button.connect_toggled(|tg| {
                if tg.is_active() {
                    set_selected_monitor("".to_string());
                }
            });
            monitors_box.append(&all_monitors_toggle_button);

            let mut had_selected_a_monitor = false;
            let monitors_clone = get_monitors();
            for monitor in monitors_clone {
                let toggle_button = ToggleButton::builder()
                    .label(&monitor)
                    .group(&all_monitors_toggle_button)
                    .build();
                if !had_selected_a_monitor && has_monitor(monitor.clone()) {
                    toggle_button.set_active(true);
                    had_selected_a_monitor = false;
                    set_selected_monitor(monitor.clone());
                }
                toggle_button.connect_toggled(move |tg| {
                    if tg.is_active() {
                        set_selected_monitor(monitor.clone());
                    }
                });
                monitors_box.append(&toggle_button);
            }
            header_box.append(&monitors_box);
            header_box.append(
                &Box::builder()
                    .hexpand(true)
                    .orientation(Orientation::Horizontal)
                    .build(),
            );
        }

        let dir_label = Label::builder()
            .halign(gtk::Align::Start)
            .label("Select a directory")
            .build();
        dir_label.add_css_class("image-browser-dir-label");
        header_box.append(&dir_label);

        let browse_button = Button::builder()
            .label("Browse")
            .halign(gtk::Align::End)
            .build();
        browse_button.add_css_class("image_browser-browse");
        browse_button.connect_clicked(clone!(
            #[weak]
            window,
            #[strong]
            dir_label,
            #[strong(rename_to = images_path_list)]
            images_grid_view.images_path_list,
            move |_| {
                let dialog = FileDialog::builder().title("Select Directory").build();
                dialog.select_folder(
                    Some(&window),
                    Some(&Cancellable::new()),
                    clone!(
                        #[strong]
                        dir_label,
                        #[strong]
                        images_path_list,
                        move |res| {
                            match res {
                                Ok(file) => {
                                    if let Some(path) = file.path() {
                                        if let Some(path_str) = path.to_str() {
                                            on_dir_selected(path_str, dir_label, images_path_list);
                                        } else {
                                            eprintln!("Failed to convert path to string");
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Dialog cancelled or error: {}", e);
                                }
                            }
                        }
                    ),
                );
            },
        ));
        header_box.append(&browse_button);

        main_box.append(&header_box);

        main_box.append(&images_grid_view.widget);

        if has_wallpapers() {
            on_dir_selected(get_first_wallpaper_path().as_str(), dir_label, images_grid_view.images_path_list);
        }

        Self { widget: main_box }
    }
}

fn on_dir_selected(path: &str, dir_label: Label, images_path_list: StringList) {
    dir_label.set_label(path);

    images_path_list.splice(0, images_path_list.n_items(), &[] as &[&str]);

    for image in read_image_entries(path) {
        images_path_list.append(image.as_str());
    }
}

fn read_image_entries(path: &str) -> Vec<String> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut files = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        if entry.file_type().map_or(true, |ft| ft.is_dir()) {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };

        if !metadata.is_file() {
            continue;
        }

        let file_name = entry.file_name();
        let file_name_str = match file_name.to_str() {
            Some(name) => name,
            None => continue,
        };

        let supported_extensions = ["jpg", "jpeg", "png", "bmp", "webp"];
        let is_supported = file_name_str
            .rfind('.')
            .map(|dot_pos| {
                let extension = &file_name_str[dot_pos + 1..];
                supported_extensions.contains(&extension.to_lowercase().as_str())
            })
            .unwrap_or(false);

        if is_supported {
            let full_path = format!("{}/{}", path, file_name_str);
            files.push(full_path);
        }
    }

    files
}
