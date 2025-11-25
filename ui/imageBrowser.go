package ui

import (
	"context"
	"os"
	"slices"
	"strings"
	"ultrapaper/utils"

	"github.com/diamondburned/gotk4/pkg/core/glib"
	"github.com/diamondburned/gotk4/pkg/gdk/v4"
	"github.com/diamondburned/gotk4/pkg/gio/v2"
	"github.com/diamondburned/gotk4/pkg/gtk/v4"
)

type ImageBrowser struct {
	Main gtk.Widgetter

	parentWindow    *gtk.Window
	hyprpaperConfig utils.HyprpaperConfig
	selectedMonitor string

	headerBox    gtk.Box
	dirLabel     gtk.Label
	browseButton gtk.Button

	imageBrowserLayout gtk.ScrolledWindow
	gridView           gtk.GridView
	stringList         *gtk.StringList
	selectionModel     *gtk.SingleSelection
	factory            *gtk.SignalListItemFactory
}

func NewImageBrowser(window *gtk.ApplicationWindow, hyprpaperConfig utils.HyprpaperConfig, monitors []string) ImageBrowser {
	var imageBrowser ImageBrowser
	imageBrowser.selectedMonitor = ""
	mainBox := gtk.NewBox(gtk.OrientationVertical, 15)
	mainBox.SetMarginTop(12)
	mainBox.SetMarginBottom(12)
	mainBox.SetMarginStart(12)
	mainBox.SetMarginEnd(12)
	mainBox.AddCSSClass("image-browser-root")

	provider := gtk.NewCSSProvider()
	provider.LoadFromString(`
	.image-browser-root { background: @theme_bg_color; border-radius: 10px; padding: 8px; }
	.image-browser-header { padding: 6px 10px; background: shade(@theme_bg_color, 0.9); border-radius: 8px; }
	.image-browser-dir-label { font-weight: 600; color: @theme_fg_color; }
	.image-browser-browse { font-weight: 500; padding: 4px 14px; border-radius: 6px; }
	.image-browser-browse:hover { background: @theme_selected_bg_color; color: @theme_selected_fg_color; }
	.image-browser-scroll { background: transparent; }
	.image-browser-grid { margin-top: 8px; }
	.image-frame { background: #000; border: 1px solid alpha(@theme_selected_bg_color,0.6); border-radius: 8px; min-width: 240px; max-width: 240px; min-height: 140px; height: 140px; }
	.image-frame:hover { border-color: @theme_selected_bg_color; box-shadow: 0 2px 6px alpha(#000,0.35); }
	.image-thumb { border-radius: 8px; }
	`)
	display := gdk.DisplayGetDefault()
	gtk.StyleContextAddProviderForDisplay(display, provider, gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)

	headerBox := gtk.NewBox(gtk.OrientationHorizontal, 5)
	headerBox.AddCSSClass("image-browser-header")
	dirLabel := gtk.NewLabel("Select a directory")
	dirLabel.AddCSSClass("image-browser-dir-label")
	dirLabel.SetHAlign(gtk.AlignStart)
	headerBox.Append(dirLabel)

	spacer := gtk.NewBox(gtk.OrientationHorizontal, 0)
	spacer.SetHExpand(true)
	headerBox.Append(spacer)

	if len(monitors) > 1 {
		monitorsBox := gtk.NewBox(gtk.OrientationHorizontal, 5)
		monitorsBox.Append(gtk.NewLabel("Monitors: "))
		allMonitorsToggleButton := gtk.NewToggleButtonWithLabel("All")
		allMonitorsToggleButton.SetActive(slices.ContainsFunc(hyprpaperConfig.Wallpapers, func(w utils.WallpaperEntry) bool { return strings.Compare(w.Monitor, "") == 0 }))
		allMonitorsToggleButton.ConnectToggled(func() {
			if allMonitorsToggleButton.Active() {
				imageBrowser.selectedMonitor = ""
			}
		})
		monitorsBox.Append(allMonitorsToggleButton)
		setActive := true
		for _, monitor := range monitors {
			toggleButton := gtk.NewToggleButtonWithLabel(monitor)
			toggleButton.SetGroup(allMonitorsToggleButton)
			if(setActive && slices.ContainsFunc(hyprpaperConfig.Wallpapers, func(w utils.WallpaperEntry) bool { return strings.Compare(w.Monitor, monitor) == 0 })) {
				toggleButton.SetActive(true)
				setActive = false
			}
			toggleButton.ConnectToggled(func() {
				if toggleButton.Active() {
					imageBrowser.selectedMonitor = monitor
				}
			})
			monitorsBox.Append(toggleButton)
		}
		headerBox.Append(monitorsBox)

		spacer1 := gtk.NewBox(gtk.OrientationHorizontal, 0)
		spacer1.SetHExpand(true)
		headerBox.Append(spacer1)
	}

	browseButton := gtk.NewButton()
	browseButton.SetLabel("Browse")
	browseButton.AddCSSClass("image-browser-browse")
	browseButton.SetHAlign(gtk.AlignEnd)
	headerBox.Append(browseButton)

	mainBox.Append(headerBox)

	scrolledWindow := gtk.NewScrolledWindow()
	scrolledWindow.SetPolicy(gtk.PolicyNever, gtk.PolicyAutomatic)
	scrolledWindow.SetVExpand(true)
	scrolledWindow.AddCSSClass("image-browser-scroll")

	stringList := gtk.NewStringList(nil)
	selection := gtk.NewSingleSelection(stringList)

	factory := gtk.NewSignalListItemFactory()
	factory.ConnectSetup(func(obj *glib.Object) {
		listItem := &gtk.ListItem{Object: obj}
		frame := gtk.NewBox(gtk.OrientationVertical, 0)
		frame.AddCSSClass("image-frame")
		frame.SetSizeRequest(240, 140)
		frame.SetHExpand(false)
		frame.SetVExpand(false)
		picture := gtk.NewPicture()
		picture.SetSizeRequest(240, 140)
		picture.SetHExpand(false)
		picture.SetVExpand(false)
		picture.SetContentFit(gtk.ContentFitContain)
		picture.SetHAlign(gtk.AlignCenter)
		picture.SetVAlign(gtk.AlignCenter)
		picture.AddCSSClass("image-thumb")
		frame.Append(picture)
		listItem.SetChild(frame)
	})
	factory.ConnectBind(func(obj *glib.Object) {
		listItem := &gtk.ListItem{Object: obj}
		itemObj := listItem.Item()
		if itemObj == nil {
			return
		}
		prop := itemObj.ObjectProperty("string")
		fullPath, ok := prop.(string)
		if !ok {
			return
		}
		frame := listItem.Child()
		box, ok := frame.(*gtk.Box)
		if !ok {
			return
		}
		pictureWidget := box.FirstChild()
		picture, ok := pictureWidget.(*gtk.Picture)
		if !ok {
			return
		}
		file := gio.NewFileForPath(fullPath)
		picture.SetFile(file)
		picture.SetTooltipText(fullPath)
		gesture := gtk.NewGestureClick()
		gesture.ConnectPressed(func(nPress int, x, y float64) {
			utils.SetWallpaper(imageBrowser.selectedMonitor, file.Path(), &hyprpaperConfig)
		})
		picture.AddController(gesture)
	})

	gridView := gtk.NewGridView(selection, &factory.ListItemFactory)
	gridView.SetMinColumns(2)
	gridView.SetMaxColumns(6)
	gridView.AddCSSClass("image-browser-grid")
	gridView.SetSingleClickActivate(true)

	scrolledWindow.SetChild(gridView)
	mainBox.Append(scrolledWindow)

	imageBrowser.Main = mainBox
	imageBrowser.parentWindow = &window.Window
	imageBrowser.hyprpaperConfig = hyprpaperConfig
	imageBrowser.headerBox = *headerBox
	imageBrowser.dirLabel = *dirLabel
	imageBrowser.browseButton = *browseButton
	imageBrowser.imageBrowserLayout = *scrolledWindow
	imageBrowser.gridView = *gridView
	imageBrowser.stringList = stringList
	imageBrowser.selectionModel = selection
	imageBrowser.factory = factory
	browseButton.ConnectClicked(imageBrowser.onBrowserButtonClicked)

	return imageBrowser
}

func (imageBrowser *ImageBrowser) onBrowserButtonClicked() {
	dialog := gtk.NewFileDialog()
	dialog.SetTitle("Select Directory")
	dialog.SelectFolder(context.Background(), imageBrowser.parentWindow, func(res gio.AsyncResulter) {
		folder, err := dialog.SelectFolderFinish(res)
		if err != nil {
			return
		}
		imageBrowser.OnFolderSelected(folder.Path())
	})
}

func (imageBrowser *ImageBrowser) OnFolderSelected(path string) {
	imageBrowser.dirLabel.SetLabel(path)
	// Clear existing entries
	imageBrowser.stringList.Splice(0, imageBrowser.stringList.NItems(), nil)
	for _, fp := range readImageEntries(path) {
		imageBrowser.stringList.Append(fp)
	}
}

func readImageEntries(path string) []string {
	entries, err := os.ReadDir(path)
	if err != nil {
		return nil
	}
	var files []string
	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		info, err := entry.Info()
		if err != nil || !info.Mode().IsRegular() {
			continue
		}
		if utils.SupportedByHyprpaper(entry.Name()) {
			files = append(files, path+"/"+entry.Name())
		}
	}
	return files
}
