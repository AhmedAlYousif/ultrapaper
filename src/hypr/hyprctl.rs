use std::process::Command;

use crate::{
    hypr::hyprpaper::WallpaperEntry,
    state::{
        add_wallpaper, get_selected_monitor, has_more_than_one_wallpaper, has_more_wallpapers_than_monitors, remove_wallpaper_of_monitor, save_config, set_preloads_from_wallpapers, set_wallpapers
    },
};

pub fn set_wallpaper(path: String) {
    let had_more_than_one_wallpaper = has_more_than_one_wallpaper();
    let monitor = get_selected_monitor();

    if monitor.is_empty() {
        set_wallpapers(vec![]);
    } else {
        remove_wallpaper_of_monitor(monitor.clone());
    }

    let entry = WallpaperEntry::new(monitor.clone(), path.clone());
    add_wallpaper(entry);


    if has_more_wallpapers_than_monitors() {
        remove_wallpaper_of_monitor("".to_owned());
    }

    set_preloads_from_wallpapers();

    save_config();

    if monitor.is_empty() && had_more_than_one_wallpaper {
        let _ = Command::new("sh").arg("-c").arg("pkill hyprpaper").output();
        let _ = Command::new("sh")
            .arg("-c")
            .arg("hyprctl dispatch exec hyprpaper")
            .output();
    } else {
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("hyprctl hyprpaper preload {}", &path))
            .output();
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("hyprctl hyprpaper wallpaper {},{}", monitor, path))
            .output();
    }
    let _ = Command::new("sh")
        .arg("-c")
        .arg("hyprctl hyprpaper unload unused")
        .output();
}

pub fn get_monitors() -> Vec<String> {
    let mut result = Vec::new();

    let output = Command::new("sh")
        .arg("-c")
        .arg("hyprctl monitors | awk '/Monitor/ {print $2}'")
        .output()
        .unwrap_or_else(|err| panic!("Could not get monitors: {}", err));

    if !output.status.success() {
        panic!("Command failed with status: {}", output.status);
    }

    let monitors_str = String::from_utf8_lossy(&output.stdout);

    for monitor in monitors_str.lines() {
        let trimmed = monitor.trim();
        if !trimmed.is_empty() {
            result.push(trimmed.to_string());
        }
    }

    result
}
