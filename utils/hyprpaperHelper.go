package utils

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"slices"
	"strings"
)

type WallpaperEntry struct {
	Monitor string // empty = all
	Path    string
}

type HyprpaperConfig struct {
	Preloads   []string // each: absolute or relative path
	Wallpapers []WallpaperEntry
	Splash     bool // optional: corresponds to splash = true
	IPCOff     bool // optional: ipc = off
}

var hyprpaperExts = map[string]struct{}{
	".png":  {},
	".jpg":  {},
	".jpeg": {},
	".jxl":  {}, // JPEG XL
	".webp": {},
}

func DefaultHyprpaperConfigPath() string {
	if xd := os.Getenv("XDG_CONFIG_HOME"); xd != "" {
		return filepath.Join(xd, "hypr", "hyprpaper.conf")
	}
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".config", "hypr", "hyprpaper.conf")
}

func LoadHyprpaperConfig(path string) (HyprpaperConfig, error) {
	var cfg HyprpaperConfig
	f, err := os.Open(path)
	if err != nil {
		return cfg, err // caller can check os.IsNotExist
	}
	defer f.Close()
	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		parts := strings.SplitN(line, "=", 2)
		if len(parts) != 2 {
			continue
		}
		key := strings.TrimSpace(parts[0])
		val := strings.TrimSpace(parts[1])
		switch key {
		case "preload":
			if val != "" {
				cfg.Preloads = append(cfg.Preloads, val)
			}
		case "wallpaper":
			wp := strings.SplitN(val, ",", 2)
			if len(wp) != 2 {
				continue
			}
			cfg.Wallpapers = append(cfg.Wallpapers, WallpaperEntry{
				Monitor: strings.TrimSpace(wp[0]),
				Path:    strings.TrimSpace(wp[1]),
			})
		case "splash":
			cfg.Splash = strings.EqualFold(val, "true")
		case "ipc":
			cfg.IPCOff = strings.EqualFold(val, "off")
		}
	}
	if err := scanner.Err(); err != nil {
		return cfg, err
	}
	return cfg, nil
}

func SaveHyprpaperConfig(path string, cfg HyprpaperConfig) error {
	var b strings.Builder
	for _, p := range cfg.Preloads {
		b.WriteString("preload = " + p + "\n")
	}
	for _, w := range cfg.Wallpapers {
		b.WriteString("wallpaper = " + w.Monitor + "," + w.Path + "\n")
	}
	if cfg.Splash {
		b.WriteString("splash = true\n")
	}
	if cfg.IPCOff {
		b.WriteString("ipc = off\n")
	}
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	return os.WriteFile(path, []byte(b.String()), 0o644)
}

func SupportedByHyprpaper(name string) bool {
	ext := strings.ToLower(filepath.Ext(name))
	_, ok := hyprpaperExts[ext]
	return ok
}

func SetWallpaper(monitor string, path string, conf *HyprpaperConfig) {
	hadMoreThanOne := len(conf.Wallpapers) > 1
	if strings.Compare(monitor, "") == 0 {
		conf.Wallpapers = []WallpaperEntry{}
		conf.Preloads = []string{}
	} else {
		conf.Wallpapers = slices.DeleteFunc(conf.Wallpapers, func(w WallpaperEntry) bool {
			return strings.Compare(w.Monitor, monitor) == 0 || strings.Compare(w.Monitor, "") == 0
		})
	}

	entry := &WallpaperEntry{
		Monitor: monitor,
		Path:    path,
	}
	conf.Wallpapers = append(conf.Wallpapers, *entry)

	conf.Preloads = []string{}
	for _, wallpaper := range conf.Wallpapers {
		conf.Preloads = append(conf.Preloads, wallpaper.Path)
	}

	SaveHyprpaperConfig(DefaultHyprpaperConfigPath(), *conf)
	if strings.Compare(monitor, "") == 0 && hadMoreThanOne {
		exec.Command("pkill", "hyprpaper").Run()
		exec.Command("hyprctl", "dispatch", "exec", "hyprpaper").Run()
	} else {
		exec.Command("hyprctl", "hyprpaper", "preload", path).Run()
		exec.Command("hyprctl", "hyprpaper", "wallpaper", monitor+","+path).Run()
	}

	exec.Command("hyprctl", "hyprpaper", "unload", "unused").Run()
}

func GetMonitors() []string {
	result := []string{}
	cmdText := "hyprctl monitors | awk '/Monitor/ {print $2}'"
	output, err := exec.Command("bash", "-c", cmdText).Output()
	if err != nil {
		panic(fmt.Sprintf("Could not get monitors: %v", err))
	}
	monitorsStr := string(output)
	for _, monitor := range strings.Split(monitorsStr, "\n") {
		if len(monitor) > 0 {
			result = append(result, strings.Trim(monitor, " "))
		}
	}
	return result
}
