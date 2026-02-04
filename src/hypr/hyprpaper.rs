use std::fmt::Display;
use std::path::PathBuf;
use std::{
    env, fs,
    io::{BufRead, BufReader, Error, Write},
};

#[derive(Clone)]
pub struct WallpaperEntry {
    pub monitor: String,
    pub path: String,
}

impl WallpaperEntry {
    pub fn new(monitor: String, path: String) -> Self {
        Self {
            monitor: monitor,
            path: path,
        }
    }
}

pub struct HyprpaperConfig {
    pub wallpapers: Vec<WallpaperEntry>,

    config_path: PathBuf,
}

impl HyprpaperConfig {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        let file = fs::File::open(&path)?;
        let reader = BufReader::new(file);

        let mut cfg = HyprpaperConfig {
            wallpapers: Vec::new(),
            config_path: path,
        };

        let mut lines_iter = reader.lines();
        let mut inside_block = false;
        let mut current_monitor: Option<String> = None;
        let mut current_path: Option<String> = None;
        while let Some(line_result) = lines_iter.next() {
            let line = line_result?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if trimmed.starts_with("wallpaper") && trimmed.contains('{') {
                inside_block = true;
                current_monitor = None;
                current_path = None;
                continue;
            }
            if inside_block {
                if trimmed.contains('}') {
                    if let (Some(monitor), Some(path)) =
                        (current_monitor.take(), current_path.take())
                    {
                        cfg.wallpapers.push(WallpaperEntry { monitor, path });
                    }
                    inside_block = false;
                    continue;
                }
                let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    match key {
                        "monitor" => {
                            current_monitor = Some(value.to_string());
                        }
                        "path" => {
                            current_path = Some(value.to_string());
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(cfg)
    }

    pub fn get_default_config_path() -> PathBuf {
        let config_path = match env::var("XDG_CONFIG_HOME") {
            Ok(val) => {
                let mut path = PathBuf::from(val);
                path.push("hypr/hyprpaper.conf");
                path
            }
            Err(_) => {
                let mut path = env::home_dir().unwrap();
                path.push(".config/hypr/hyprpaper.conf");
                path
            }
        };

        config_path
    }

    pub fn save_config(&self) {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.config_path)
            .unwrap();
        match file.write_all(self.to_string().as_bytes()) {
            Ok(()) => {}
            Err(_error) => {}
        };
    }
}

impl Display for HyprpaperConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // for preload in &self.preloads {
        //     writeln!(f, "preload = {}", preload)?;
        // }
        for wallpaper in &self.wallpapers {
            writeln!(
                f,
                "wallpaper {{\n\tmonitor = {}\n\tpath = {}\n}}",
                wallpaper.monitor, wallpaper.path
            )?;
        }
        Ok(())
    }
}
