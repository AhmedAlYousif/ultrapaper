

[![Last Release](https://img.shields.io/github/v/release/AhmedAlYousif/ultrapaper?label=release)](https://github.com/AhmedAlYousif/ultrapaper/releases)

# Ultrapaper
Ultrapaper es una interfaz gráfica (GUI) para [hyprpaper](https://wiki.hypr.land/Hypr-Ecosystem/hyprpaper/), construida con GTK4 y desarrollada en Rust. Utiliza [hyprctl](https://wiki.hypr.land/Configuring/Using-hyprctl/) para aplicar fondos de pantalla en tiempo real y genera automáticamente la configuración de hyprpaper.

![screenshot](./screenshots/screenshot.png)

## Características
- Selección de fondo de pantalla por monitor
- Control del modo de ajuste (contain, cover, tile, fill)
- Modo de directorio con tiempo de espera configurable
- Configuración global: splash, desplazamiento de splash, opacidad de splash, IPC
- Aplicación en tiempo real de fondos de pantalla vía hyprctl (no se requiere reinicio para cambios que solo afecten la ruta)
- Actualiza automáticamente el archivo de configuración de hyprpaper

## Requisitos
- Hyprland con hyprpaper en ejecución
- `hyprctl` disponible en tu `PATH`

## Instalación

### Arch Linux
#### Paquete AUR (Recomendado)
```bash
yay -S ultrapaper
```
```bash
paru -S ultrapaper
```

**Instalación manual desde AUR:**
```bash
git clone https://aur.archlinux.org/ultrapaper.git
cd ultrapaper
makepkg -si
```

### Binario Precompilado
Descarga la última versión desde la [página de lanzamientos](https://github.com/AhmedAlYousif/ultrapaper/releases):
```bash
curl -L -o ultrapaper https://github.com/AhmedAlYousif/ultrapaper/releases/download/vX.Y.Z/ultrapaper-linux-x86_64_vX.Y.Z
chmod +x ultrapaper
mv ultrapaper ~/.local/bin/
```
Reemplaza `vX.Y.Z` por la versión real.

### Compilar desde el Código Fuente
#### Dependencias
Necesitas Rust (estable) y las bibliotecas de desarrollo de GTK4.

##### Arch:
```bash
sudo pacman -S gtk4 gobject-introspection glib2 cairo pango gdk-pixbuf2
```

##### Debian/Ubuntu:
```bash
sudo apt install -y build-essential pkg-config \
    libgtk-4-dev gobject-introspection libgirepository1.0-dev \
    libglib2.0-dev libcairo2-dev libpango1.0-dev libgdk-pixbuf-2.0-dev
```

##### Fedora:
```bash
sudo dnf install gtk4-devel gobject-introspection-devel \
    glib2-devel cairo-devel pango-devel gdk-pixbuf2-devel
```

#### Compilación
```bash
git clone https://github.com/AhmedAlYousif/ultrapaper.git
cd ultrapaper
cargo build --release
./target/release/ultrapaper
```

## Uso
- Asegúrate de que hyprpaper se esté ejecutando en tu configuración de Hyprland.
- Ejecuta Ultrapaper.
- Selecciona un monitor, elige un fondo de pantalla y un modo de ajuste, luego haz clic en Aplicar.
- Los cambios que solo afectan la ruta del fondo de pantalla se aplican en tiempo real; otros cambios solicitarán un reinicio de hyprpaper.

## Solución de Problemas
**La compilación falla por paquetes faltantes de pkg-config:**
Instala las bibliotecas de desarrollo de GTK4 para tu distribución (ver arriba).

**No ocurre nada al cambiar el fondo de pantalla:**
Verifica la salida de `hyprctl monitors` y asegúrate de que hyprpaper se esté ejecutando.

**Fallo de segmentación (segfault) o problemas de visualización:**
Ejecútalo bajo Wayland/Hyprland (no X11). Confirma que `$WAYLAND_DISPLAY` esté configurado.

## Notas
- Exclusivo para Wayland (Hyprland).
