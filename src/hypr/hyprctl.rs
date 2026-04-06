use std::process::Command;

pub fn restart_hyprpaper() {
    let _ = Command::new("sh")
        .arg("-c")
        .arg("pkill hyprpaper && hyprctl dispatch exec hyprpaper")
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
