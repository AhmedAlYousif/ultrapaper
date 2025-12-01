use std::{path::Path, sync::{OnceLock, RwLock}};

use crate::hypr::hyprpaper::{HyprpaperConfig, WallpaperEntry};

static APP_STATE: OnceLock<RwLock<AppState>> = OnceLock::new();

#[derive(Default)]
pub struct AppState {
    pub config: Option<HyprpaperConfig>,
    pub monitors: Vec<String>,
    selected_monitor: String,
}

fn get_app_state() -> &'static RwLock<AppState> {
    APP_STATE.get_or_init(|| RwLock::new(AppState::default()))
}

pub fn set_config(config: HyprpaperConfig) {
    let mut state = get_app_state().write().unwrap();
    state.config = Some(config);
}

pub fn set_wallpapers(wallpapers: Vec<WallpaperEntry>) {
    let mut state = get_app_state().write().unwrap();
    state.config.as_mut().unwrap().wallpapers = wallpapers;
}

pub fn remove_wallpaper_of_monitor(monitor: String) {
    let mut state = get_app_state().write().unwrap();
    state
        .config
        .as_mut()
        .unwrap()
        .wallpapers
        .retain(|entry| !entry.monitor.eq(&monitor));
}

// pub fn set_preloads(preloads: Vec<String>) {
//     let mut state = get_app_state().write().unwrap();
//     state.config.as_mut().unwrap().preloads = preloads;
// }

pub fn set_preloads_from_wallpapers() {
    let mut state = get_app_state().write().unwrap();
    state.config.as_mut().unwrap().preloads = vec![];
    let wallpapers = state.config.as_ref().unwrap().wallpapers.clone();
    for wallpaper in wallpapers {
        state
            .config
            .as_mut()
            .unwrap()
            .preloads
            .append(&mut vec![wallpaper.path.clone()]);
    }
}

pub fn add_wallpaper(entry: WallpaperEntry) {
    let mut state = get_app_state().write().unwrap();
    state
        .config
        .as_mut()
        .unwrap()
        .wallpapers
        .append(&mut vec![entry]);
}

pub fn set_monitors(monitors: Vec<String>) {
    let mut state = get_app_state().write().unwrap();
    state.monitors = monitors;
}

pub fn get_monitors() -> Vec<String> {
    let state = get_app_state().read().unwrap();
    state.monitors.clone()
}

pub fn has_more_than_one_monitors() -> bool {
    let state = get_app_state().read().unwrap();
    state.monitors.len() > 1
}

pub fn has_more_than_one_wallpaper() -> bool {
    let state = get_app_state().read().unwrap();
    state.config.as_ref().unwrap().wallpapers.len() > 1
}
pub fn has_more_wallpapers_than_monitors() -> bool {
    let state = get_app_state().read().unwrap();
    state.config.as_ref().unwrap().wallpapers.len() > state.monitors.len()
}

pub fn has_wallpapers() -> bool {
    let state = get_app_state().read().unwrap();
    state.config.as_ref().unwrap().wallpapers.len() > 0
}

pub fn get_first_wallpaper_path() -> String {
    let state = get_app_state().read().unwrap();
    Path::new(&state.config.as_ref().unwrap().wallpapers.first().unwrap().path)
    .parent()
    .map(|p| p.to_string_lossy().to_string())
    .unwrap_or_else(|| "/".to_string())
}

pub fn has_empty_monitor_name() -> bool {
    let state = get_app_state().read().unwrap();
    state
        .config
        .as_ref()
        .unwrap()
        .wallpapers
        .iter()
        .any(|e| e.monitor.is_empty())
}

pub fn has_monitor(monitor: String) -> bool {
    let state = get_app_state().read().unwrap();
    state
        .config
        .as_ref()
        .unwrap()
        .wallpapers
        .iter()
        .any(|e| e.monitor.eq(&monitor))
}

pub fn save_config() {
    let state = get_app_state().read().unwrap();
    state.config.as_ref().unwrap().save_config();
}

pub fn set_selected_monitor(monitor: String) {
    let mut state = get_app_state().write().unwrap();
    state.selected_monitor = monitor;
}

pub fn get_selected_monitor() -> String {
    let state = get_app_state().read().unwrap();
    state.selected_monitor.clone()
}
