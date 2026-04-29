use gio::glib::clone;
use gio::Cancellable;
use gtk::prelude::*;
use gtk::{
    Align, Box as GtkBox, Button, CheckButton, DropDown, Entry, Label, Orientation, StringList,
};

use std::cell::RefCell;
use std::rc::Rc;

use crate::hypr::hyprpaper::{FitMode, Order, WallpaperEntry};
use crate::state::get_last_browsed_dir;

pub struct WallpaperOptionsPanel {
    pub widget: GtkBox,
    dir_label: Label,
    fit_mode_drop: DropDown,
    use_dir_check: CheckButton,
    dir_only_box: GtkBox,
    timeout_entry: Entry,
    order_drop: DropDown,
    recursive_check: CheckButton,
    dir_changed_callbacks: Rc<RefCell<Vec<std::boxed::Box<dyn Fn(&str)>>>>,
}

// Helper function to get fit mode string list from the FitMode enum
fn get_fit_modes() -> &'static [&'static str] {
    &["cover", "contain", "tile", "fill"]
}

// Helper function to get order string list from the Order enum
fn get_orders() -> &'static [&'static str] {
    &["default", "random", "random-shuffle"]
}

impl WallpaperOptionsPanel {
    pub fn new(window: &gtk::ApplicationWindow) -> Self {
        let root = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .build();
        root.add_css_class("options-panel");

        let dir_row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();

        let dir_label = Label::builder()
            .label(get_last_browsed_dir().as_str())
            .halign(Align::Start)
            .hexpand(true)
            .ellipsize(gtk::pango::EllipsizeMode::Start)
            .build();
        dir_label.add_css_class("options-path-label");

        let browse_btn = Button::builder().label("Browse").build();
        browse_btn.add_css_class("options-browse-btn");

        dir_row.append(&dir_label);
        dir_row.append(&browse_btn);

        let fit_row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();
        let fit_label = Label::builder()
            .label("Fit mode")
            .hexpand(true)
            .halign(Align::Start)
            .build();
        let fit_list = StringList::new(get_fit_modes());
        let fit_mode_drop = DropDown::builder().model(&fit_list).build();
        fit_row.append(&fit_label);
        fit_row.append(&fit_mode_drop);

        let use_dir_check = CheckButton::builder()
            .label("Use all images in directory")
            .build();

        let dir_only_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .visible(false)
            .build();

        let timeout_row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();
        let timeout_label = Label::builder()
            .label("Timeout (s)")
            .hexpand(true)
            .halign(Align::Start)
            .build();
        let timeout_entry = Entry::builder()
            .placeholder_text("60")
            .width_chars(6)
            .build();
        timeout_row.append(&timeout_label);
        timeout_row.append(&timeout_entry);

        timeout_entry.connect_changed(|entry| {
            let text = entry.text();
            let filtered: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
            if filtered != text.as_str() {
                let pos = filtered.len() as i32;
                entry.set_text(&filtered);
                entry.set_position(pos);
            }
        });

        let order_row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();
        let order_label = Label::builder()
            .label("Order")
            .hexpand(true)
            .halign(Align::Start)
            .build();
        let order_list = StringList::new(get_orders());
        let order_drop = DropDown::builder().model(&order_list).build();
        order_row.append(&order_label);
        order_row.append(&order_drop);

        let recursive_check = CheckButton::builder()
            .label("Search subdirectories recursively")
            .build();

        dir_only_box.append(&timeout_row);
        dir_only_box.append(&order_row);
        dir_only_box.append(&recursive_check);

        root.append(&dir_row);
        root.append(&fit_row);
        root.append(&use_dir_check);
        root.append(&dir_only_box);

        use_dir_check.connect_toggled(clone!(
            #[weak]
            dir_only_box,
            move |check| {
                dir_only_box.set_visible(check.is_active());
            }
        ));

        let dir_changed_callbacks: Rc<RefCell<Vec<std::boxed::Box<dyn Fn(&str)>>>> =
            Rc::new(RefCell::new(Vec::new()));

        browse_btn.connect_clicked(clone!(
            #[weak]
            window,
            #[weak]
            dir_label,
            #[strong]
            dir_changed_callbacks,
            move |_| {
                let dialog = gtk::FileDialog::builder().title("Select Directory").build();
                dialog.select_folder(
                    Some(&window),
                    Some(&Cancellable::new()),
                    clone!(
                        #[weak]
                        dir_label,
                        #[strong]
                        dir_changed_callbacks,
                        move |res| {
                            if let Ok(file) = res {
                                if let Some(path) = file.path() {
                                    if let Some(s) = path.to_str() {
                                        dir_label.set_label(s);
                                        crate::state::set_last_browsed_dir(s.to_string());
                                        for cb in dir_changed_callbacks.borrow().iter() {
                                            cb(s);
                                        }
                                    }
                                }
                            }
                        }
                    ),
                );
            }
        ));

        Self {
            widget: root,
            dir_label,
            fit_mode_drop,
            use_dir_check,
            dir_only_box,
            timeout_entry,
            order_drop,
            recursive_check,
            dir_changed_callbacks,
        }
    }

    pub fn connect_dir_changed<F: Fn(&str) + 'static>(&self, f: F) {
        self.dir_changed_callbacks
            .borrow_mut()
            .push(std::boxed::Box::new(f));
    }

    pub fn load(&self, entry: Option<&WallpaperEntry>) {
        let entry = match entry {
            Some(e) => e.clone(),
            None => WallpaperEntry::new("".to_string(), "".to_string()),
        };

        let path = &entry.path;
        let is_dir = std::path::Path::new(path).is_dir();

        if !path.is_empty() {
            if is_dir {
                self.dir_label.set_label(path);
            } else if let Some(parent) = std::path::Path::new(path).parent() {
                self.dir_label.set_label(parent.to_str().unwrap_or(""));
            }
        } else {
            self.dir_label
                .set_label(crate::state::get_last_browsed_dir().as_str());
        }

        let fit_idx = match entry.fit_mode {
            FitMode::Cover => 0,
            FitMode::Contain => 1,
            FitMode::Tile => 2,
            FitMode::Fill => 3,
        };
        self.fit_mode_drop.set_selected(fit_idx);

        self.dir_only_box.set_visible(is_dir);
        self.use_dir_check.set_active(is_dir);

        let timeout_str = entry.timeout.map(|t| t.to_string()).unwrap_or_default();
        self.timeout_entry.set_text(&timeout_str);

        let order_idx = match entry.order.as_ref() {
            Some(Order::Default) | None => 0,
            Some(Order::Random) => 1,
            Some(Order::RandomShuffle) => 2,
        };
        self.order_drop.set_selected(order_idx);

        self.recursive_check
            .set_active(entry.recursive.unwrap_or(false));
    }

    pub fn is_dir_mode(&self) -> bool {
        self.use_dir_check.is_active()
    }

    pub fn get_dir(&self) -> String {
        self.dir_label.label().to_string()
    }

    pub fn build_entry(&self, monitor: &str, current_path: &str) -> WallpaperEntry {
        let fit_mode = match self.fit_mode_drop.selected() {
            0 => FitMode::Cover,
            1 => FitMode::Contain,
            2 => FitMode::Tile,
            3 => FitMode::Fill,
            _ => FitMode::Cover,
        };

        let (path, timeout, order, recursive) = if self.use_dir_check.is_active() {
            let dir = self.dir_label.label().to_string();
            let timeout = self
                .timeout_entry
                .text()
                .parse::<u32>()
                .ok()
                .filter(|&t| t > 0)
                .or(Some(60));

            let order = match self.order_drop.selected() {
                0 => Some(Order::Default),
                1 => Some(Order::Random),
                2 => Some(Order::RandomShuffle),
                _ => Some(Order::Default),
            };

            let recursive = if self.recursive_check.is_active() {
                Some(true)
            } else {
                None // None means false/default behavior
            };

            (dir, timeout, order, recursive)
        } else {
            (current_path.to_string(), None, None, None)
        };

        WallpaperEntry {
            monitor: monitor.to_string(),
            path,
            fit_mode,
            timeout,
            order,
            recursive,
        }
    }
}
