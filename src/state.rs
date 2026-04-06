use std::{
    path::Path,
    sync::{OnceLock, RwLock},
};

use crate::hypr::hyprpaper::{ConfigEntry, HyprpaperConfig, WallpaperEntry};

static APP_STATE: OnceLock<RwLock<AppState>> = OnceLock::new();

#[derive(Default)]
pub struct AppState {
    pub config: Option<HyprpaperConfig>,
    pub monitors: Vec<String>,
    selected_monitor: String,
    last_browsed_dir: String,
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
    let cfg = state.config.as_mut().unwrap();
    cfg.entries
        .retain(|e| !matches!(e, ConfigEntry::Wallpaper(_)));
    for w in wallpapers {
        cfg.entries.push(ConfigEntry::Wallpaper(w));
    }
}

pub fn remove_wallpaper_of_monitor(monitor: String) {
    let mut state = get_app_state().write().unwrap();
    state.config.as_mut().unwrap().entries.retain(|e| match e {
        ConfigEntry::Wallpaper(w) => !w.monitor.eq(&monitor),
        _ => true,
    });
}

pub fn add_wallpaper(entry: WallpaperEntry) {
    let mut state = get_app_state().write().unwrap();
    state
        .config
        .as_mut()
        .unwrap()
        .entries
        .push(ConfigEntry::Wallpaper(entry));
}

pub fn get_wallpaper_for_monitor(monitor: &str) -> Option<WallpaperEntry> {
    let state = get_app_state().read().unwrap();
    state
        .config
        .as_ref()
        .unwrap()
        .wallpapers()
        .find(|w| w.monitor == monitor)
        .cloned()
}

pub fn update_wallpaper_for_monitor(monitor: &str, entry: WallpaperEntry) {
    let mut state = get_app_state().write().unwrap();
    let cfg = state.config.as_mut().unwrap();
    let mut replaced = false;
    for e in cfg.entries.iter_mut() {
        if let ConfigEntry::Wallpaper(w) = e {
            if w.monitor == monitor {
                *w = entry.clone();
                replaced = true;
                break;
            }
        }
    }
    if !replaced {
        cfg.entries.push(ConfigEntry::Wallpaper(entry));
    }
}

pub fn set_global_entry(new_entry: ConfigEntry) {
    let mut state = get_app_state().write().unwrap();
    let cfg = state.config.as_mut().unwrap();
    let matches = |e: &ConfigEntry| match (&new_entry, e) {
        (ConfigEntry::Splash(_), ConfigEntry::Splash(_)) => true,
        (ConfigEntry::SplashOffset(_), ConfigEntry::SplashOffset(_)) => true,
        (ConfigEntry::SplashOpacity(_), ConfigEntry::SplashOpacity(_)) => true,
        (ConfigEntry::Ipc(_), ConfigEntry::Ipc(_)) => true,
        _ => false,
    };
    if let Some(existing) = cfg.entries.iter_mut().find(|e| matches(e)) {
        *existing = new_entry;
    } else {
        cfg.entries.push(new_entry);
    }
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
    state.config.as_ref().unwrap().wallpapers().count() > 1
}

pub fn has_more_wallpapers_than_monitors() -> bool {
    let state = get_app_state().read().unwrap();
    state.config.as_ref().unwrap().wallpapers().count() > state.monitors.len()
}

pub fn has_wallpapers() -> bool {
    let state = get_app_state().read().unwrap();
    state.config.as_ref().unwrap().wallpapers().next().is_some()
}

pub fn get_first_wallpaper_path() -> String {
    let state = get_app_state().read().unwrap();
    Path::new(
        &state
            .config
            .as_ref()
            .unwrap()
            .wallpapers()
            .next()
            .unwrap()
            .path,
    )
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
        .wallpapers()
        .any(|e| e.monitor.is_empty())
}

pub fn has_monitor(monitor: String) -> bool {
    let state = get_app_state().read().unwrap();
    state
        .config
        .as_ref()
        .unwrap()
        .wallpapers()
        .any(|e| e.monitor.eq(&monitor))
}

pub fn save_config() {
    let state = get_app_state().read().unwrap();
    state.config.as_ref().unwrap().save_config();
}

pub fn get_splash() -> bool {
    let state = get_app_state().read().unwrap();
    state.config.as_ref().unwrap().splash()
}

pub fn get_splash_offset() -> f32 {
    let state = get_app_state().read().unwrap();
    state.config.as_ref().unwrap().splash_offset()
}

pub fn get_splash_opacity() -> f32 {
    let state = get_app_state().read().unwrap();
    state.config.as_ref().unwrap().splash_opacity()
}

pub fn get_ipc() -> bool {
    let state = get_app_state().read().unwrap();
    state.config.as_ref().unwrap().ipc()
}

pub fn set_selected_monitor(monitor: String) {
    let mut state = get_app_state().write().unwrap();
    state.selected_monitor = monitor;
}

pub fn get_selected_monitor() -> String {
    let state = get_app_state().read().unwrap();
    state.selected_monitor.clone()
}

pub fn set_last_browsed_dir(path: String) {
    let mut state = get_app_state().write().unwrap();
    state.last_browsed_dir = path;
}

pub fn get_last_browsed_dir() -> String {
    let state = get_app_state().read().unwrap();
    if !state.last_browsed_dir.is_empty() {
        return state.last_browsed_dir.clone();
    }
    if let Some(cfg) = &state.config {
        if let Some(w) = cfg.wallpapers().next() {
            if let Some(parent) = Path::new(&w.path).parent() {
                return parent.to_string_lossy().to_string();
            }
        }
    }
    std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
}
