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
    pub preloads: Vec<String>,
    pub wallpapers: Vec<WallpaperEntry>,

    config_path: PathBuf,
}

impl HyprpaperConfig {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        let file = fs::File::open(&path)?;
        let reader = BufReader::new(file);

        let mut cfg = HyprpaperConfig {
            preloads: Vec::new(),
            wallpapers: Vec::new(),
            config_path: path,
        };

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Split on first '=' only
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0].trim();
            let val = parts[1].trim();

            match key {
                "preload" => {
                    if !val.is_empty() {
                        cfg.preloads.push(val.to_string());
                    }
                }
                "wallpaper" => {
                    let wp: Vec<&str> = val.splitn(2, ',').collect();
                    if wp.len() == 2 {
                        cfg.wallpapers.push(WallpaperEntry {
                            monitor: wp[0].trim().to_string(),
                            path: wp[1].trim().to_string(),
                        });
                    }
                }
                _ => {}
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
        for preload in &self.preloads {
            writeln!(f, "preload = {}", preload)?;
        }
        for wallpaper in &self.wallpapers {
            writeln!(f, "wallpaper = {},{}", wallpaper.monitor, wallpaper.path)?;
        }
        Ok(())
    }
}
