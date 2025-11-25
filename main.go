package main

import (
	"fmt"
	"os"
	"path/filepath"
	"ultrapaper/ui"
	"ultrapaper/utils"

	"github.com/diamondburned/gotk4/pkg/gio/v2"
	"github.com/diamondburned/gotk4/pkg/gtk/v4"
)

func main() {
	hypaperConfigPath := utils.DefaultHyprpaperConfigPath()
	hyprpaperConfig, err := utils.LoadHyprpaperConfig(hypaperConfigPath)
	if err != nil {
		panic(fmt.Errorf("loading config: %w", err))
	}
	monitors := utils.GetMonitors()
	app := gtk.NewApplication("sa.ahmedy.hyprgpaper", gio.ApplicationFlagsNone)
	app.ConnectActivate(func() { activate(app, hyprpaperConfig, monitors) })

	if code := app.Run(os.Args); code > 0 {
		os.Exit(code)
	}
}

func activate(app *gtk.Application, hyprpaperConfig utils.HyprpaperConfig, monitors []string) {
	window := gtk.NewApplicationWindow(app)
	imageBrowser := ui.NewImageBrowser(window, hyprpaperConfig, monitors)
	window.SetTitle("Ultrapaper")
	window.SetChild(imageBrowser.Main)
	window.SetVisible(true)

	if len(hyprpaperConfig.Wallpapers) > 0 {
		startingPath := filepath.Dir(hyprpaperConfig.Wallpapers[0].Path)
		imageBrowser.OnFolderSelected(startingPath)
	}
}
