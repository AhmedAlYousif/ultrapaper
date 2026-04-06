use gtk::prelude::*;
use gtk::{Align, Box, Entry, Label, Orientation, Switch};

use crate::state::{get_ipc, get_splash, get_splash_offset, get_splash_opacity};

pub struct SettingsPanel {
    pub widget: Box,
    splash_switch: Switch,
    splash_offset_entry: Entry,
    splash_opacity_entry: Entry,
    ipc_switch: Switch,
}

impl SettingsPanel {
    pub fn new() -> Self {
        let root = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .build();
        root.add_css_class("settings-panel");

        let title = Label::builder()
            .label("Settings")
            .halign(Align::Start)
            .build();
        title.add_css_class("settings-title");
        root.append(&title);

        let splash_switch = Self::build_row(&root, "Splash screen");
        splash_switch.set_active(get_splash());

        let splash_offset_entry = Self::build_entry_row(&root, "Splash offset");
        splash_offset_entry.set_text(&(get_splash_offset() as i32).to_string());
        Self::filter_uint(&splash_offset_entry);

        let splash_opacity_entry = Self::build_entry_row(&root, "Splash opacity");
        splash_opacity_entry.set_text(&format!("{:.2}", get_splash_opacity()));
        Self::filter_float(&splash_opacity_entry);

        let ipc_switch = Self::build_row(&root, "IPC");
        ipc_switch.set_active(get_ipc());

        Self {
            widget: root,
            splash_switch,
            splash_offset_entry,
            splash_opacity_entry,
            ipc_switch,
        }
    }

    fn filter_uint(entry: &Entry) {
        entry.connect_changed(|e| {
            let text = e.text();
            let filtered: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
            if filtered != text.as_str() {
                let pos = filtered.len() as i32;
                e.set_text(&filtered);
                e.set_position(pos);
            }
        });
    }

    fn filter_float(entry: &Entry) {
        entry.connect_changed(|e| {
            let text = e.text();
            let mut dot_seen = false;
            let filtered: String = text
                .chars()
                .filter(|&c| {
                    if c.is_ascii_digit() {
                        true
                    } else if c == '.' && !dot_seen {
                        dot_seen = true;
                        true
                    } else {
                        false
                    }
                })
                .collect();
            if filtered != text.as_str() {
                let pos = filtered.len() as i32;
                e.set_text(&filtered);
                e.set_position(pos);
            }
        });
    }

    fn build_row(parent: &Box, label_text: &str) -> Switch {
        let row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();
        let label = Label::builder()
            .label(label_text)
            .hexpand(true)
            .halign(Align::Start)
            .build();
        let switch = Switch::builder().valign(Align::Center).build();
        row.append(&label);
        row.append(&switch);
        parent.append(&row);
        switch
    }

    fn build_entry_row(parent: &Box, label_text: &str) -> Entry {
        let row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .build();
        let label = Label::builder()
            .label(label_text)
            .hexpand(true)
            .halign(Align::Start)
            .build();
        let entry = Entry::builder().width_chars(6).build();
        row.append(&label);
        row.append(&entry);
        parent.append(&row);
        entry
    }

    pub fn get_splash(&self) -> bool {
        self.splash_switch.is_active()
    }

    pub fn get_splash_offset(&self) -> f32 {
        self.splash_offset_entry
            .text()
            .parse::<f32>()
            .unwrap_or(20.0)
    }

    pub fn get_splash_opacity(&self) -> f32 {
        self.splash_opacity_entry
            .text()
            .parse::<f32>()
            .unwrap_or(0.8)
            .clamp(0.0, 1.0)
    }

    pub fn get_ipc(&self) -> bool {
        self.ipc_switch.is_active()
    }
}
