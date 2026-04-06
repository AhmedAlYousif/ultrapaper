use std::fmt::Display;
use std::path::PathBuf;
use std::{
    env, fs,
    io::{BufRead, BufReader, Error, Write},
};

#[derive(Clone, Default)]
pub enum FitMode {
    Contain,
    #[default]
    Cover,
    Tile,
    Fill,
}

impl FitMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "contain" => Some(FitMode::Contain),
            "cover" => Some(FitMode::Cover),
            "tile" => Some(FitMode::Tile),
            "fill" => Some(FitMode::Fill),
            _ => None,
        }
    }
}

impl Display for FitMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FitMode::Contain => "contain",
            FitMode::Cover => "cover",
            FitMode::Tile => "tile",
            FitMode::Fill => "fill",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Default)]
pub enum Order {
    #[default]
    Default,
    Random,
    RandomShuffle,
}

impl Order {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "default" => Some(Order::Default),
            "random" => Some(Order::Random),
            "random-shuffle" => Some(Order::RandomShuffle),
            _ => None,
        }
    }
}

impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Order::Default => "default",
            Order::Random => "random",
            Order::RandomShuffle => "random-shuffle",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub struct WallpaperEntry {
    pub monitor: String,
    pub path: String,
    pub fit_mode: FitMode,
    pub timeout: Option<u32>,
    pub order: Option<Order>,
    pub recursive: Option<bool>,
}

impl WallpaperEntry {
    pub fn new(monitor: String, path: String) -> Self {
        Self {
            monitor,
            path,
            fit_mode: FitMode::Cover,
            timeout: None,
            order: None,
            recursive: None,
        }
    }
}

#[derive(Clone)]
pub enum ConfigEntry {
    Splash(bool),
    SplashOffset(f32),
    SplashOpacity(f32),
    Ipc(bool),
    Source(String),
    Wallpaper(WallpaperEntry),
    Raw(String),
}

pub struct HyprpaperConfig {
    pub entries: Vec<ConfigEntry>,
    config_path: PathBuf,
}

impl HyprpaperConfig {
    pub fn wallpapers(&self) -> impl Iterator<Item = &WallpaperEntry> {
        self.entries.iter().filter_map(|e| match e {
            ConfigEntry::Wallpaper(w) => Some(w),
            _ => None,
        })
    }

    pub fn sources(&self) -> impl Iterator<Item = &String> {
        self.entries.iter().filter_map(|e| match e {
            ConfigEntry::Source(s) => Some(s),
            _ => None,
        })
    }

    pub fn splash(&self) -> bool {
        self.entries
            .iter()
            .filter_map(|e| match e {
                ConfigEntry::Splash(v) => Some(*v),
                _ => None,
            })
            .last()
            .unwrap_or(true)
    }

    pub fn splash_offset(&self) -> f32 {
        self.entries
            .iter()
            .filter_map(|e| match e {
                ConfigEntry::SplashOffset(v) => Some(*v),
                _ => None,
            })
            .last()
            .unwrap_or(20.0)
    }

    pub fn splash_opacity(&self) -> f32 {
        self.entries
            .iter()
            .filter_map(|e| match e {
                ConfigEntry::SplashOpacity(v) => Some(*v),
                _ => None,
            })
            .last()
            .unwrap_or(0.8)
    }

    pub fn ipc(&self) -> bool {
        self.entries
            .iter()
            .filter_map(|e| match e {
                ConfigEntry::Ipc(v) => Some(*v),
                _ => None,
            })
            .last()
            .unwrap_or(true)
    }

    pub fn new(path: PathBuf) -> Result<Self, Error> {
        let file = fs::File::open(&path)?;
        let reader = BufReader::new(file);
        let mut cfg = HyprpaperConfig {
            entries: Vec::new(),
            config_path: path,
        };
        let mut inside_block = false;
        let mut current_monitor: Option<String> = None;
        let mut current_path: Option<String> = None;
        let mut current_fit_mode: Option<FitMode> = None;
        let mut current_timeout: Option<u32> = None;
        let mut current_order: Option<Order> = None;
        let mut current_recursive: Option<bool> = None;

        for line_result in reader.lines() {
            let line = line_result?;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with("wallpaper") && trimmed.contains('{') {
                inside_block = true;
                current_monitor = None;
                current_path = None;
                current_fit_mode = None;
                current_timeout = None;
                current_order = None;
                current_recursive = None;
                continue;
            }

            if inside_block {
                if trimmed.contains('}') {
                    if let (Some(monitor), Some(path)) =
                        (current_monitor.take(), current_path.take())
                    {
                        cfg.entries.push(ConfigEntry::Wallpaper(WallpaperEntry {
                            monitor,
                            path,
                            fit_mode: current_fit_mode.take().unwrap_or_default(),
                            timeout: current_timeout.take(),
                            order: current_order.take(),
                            recursive: current_recursive.take(),
                        }));
                    }
                    inside_block = false;
                    continue;
                }
                let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    match key {
                        "monitor" => current_monitor = Some(value.to_string()),
                        "path" => current_path = Some(value.to_string()),
                        "fit_mode" => current_fit_mode = FitMode::from_str(value),
                        "timeout" => current_timeout = value.parse().ok(),
                        "order" => current_order = Order::from_str(value),
                        "recursive" => {
                            current_recursive = match value {
                                "true" => Some(true),
                                "false" => Some(false),
                                _ => None,
                            }
                        }
                        _ => {}
                    }
                }
                continue;
            }

            if trimmed.starts_with('#') {
                cfg.entries.push(ConfigEntry::Raw(line.to_string()));
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let value = parts[1].trim();
                match key {
                    "splash" => cfg.entries.push(ConfigEntry::Splash(value == "true")),
                    "splash_offset" => cfg
                        .entries
                        .push(ConfigEntry::SplashOffset(value.parse().unwrap_or(20.0))),
                    "splash_opacity" => cfg
                        .entries
                        .push(ConfigEntry::SplashOpacity(value.parse().unwrap_or(0.8))),
                    "ipc" => cfg.entries.push(ConfigEntry::Ipc(value == "true")),
                    "source" => cfg.entries.push(ConfigEntry::Source(value.to_string())),
                    _ => cfg.entries.push(ConfigEntry::Raw(line.to_string())),
                }
            } else {
                cfg.entries.push(ConfigEntry::Raw(line.to_string()));
            }
        }
        Ok(cfg)
    }

    pub fn get_default_config_path() -> PathBuf {
        match env::var("XDG_CONFIG_HOME") {
            Ok(val) => {
                let mut path = PathBuf::from(val);
                path.push("hypr/hyprpaper.conf");
                path
            }
            Err(_) => {
                let home = env::var("HOME").expect("HOME environment variable not set");
                let mut path = PathBuf::from(home);
                path.push(".config/hypr/hyprpaper.conf");
                path
            }
        }
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
        let has_named = self.wallpapers().any(|w| !w.monitor.is_empty());

        for entry in &self.entries {
            match entry {
                ConfigEntry::Splash(v) => writeln!(f, "splash = {}", v)?,
                ConfigEntry::SplashOffset(v) => writeln!(f, "splash_offset = {}", v)?,
                ConfigEntry::SplashOpacity(v) => writeln!(f, "splash_opacity = {}", v)?,
                ConfigEntry::Ipc(v) => writeln!(f, "ipc = {}", v)?,
                ConfigEntry::Source(s) => writeln!(f, "source = {}", s)?,
                ConfigEntry::Raw(s) => writeln!(f, "{}", s)?,
                ConfigEntry::Wallpaper(w) => {
                    if w.monitor.is_empty() && has_named {
                        continue;
                    }
                    writeln!(f, "wallpaper {{")?;
                    writeln!(f, "\tmonitor = {}", w.monitor)?;
                    writeln!(f, "\tpath = {}", w.path)?;
                    writeln!(f, "\tfit_mode = {}", w.fit_mode)?;
                    if let Some(timeout) = &w.timeout {
                        writeln!(f, "\ttimeout = {}", timeout)?;
                    }
                    if let Some(order) = &w.order {
                        writeln!(f, "\torder = {}", order)?;
                    }
                    if let Some(true) = &w.recursive {
                        writeln!(f, "\trecursive = true")?;
                    }
                    writeln!(f, "}}")?;
                    writeln!(f)?;
                }
            }
        }
        Ok(())
    }
}
