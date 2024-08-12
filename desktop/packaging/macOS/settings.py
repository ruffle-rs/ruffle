format = defines.get("format")
filesystem = defines.get("filesystem")
files = [("package/Ruffle.app", "Ruffle.app")]
symlinks = {"Applications": "/Applications"}
icon_locations = {
	"Ruffle.app": (160, 300),
	"Applications": (480, 300)
}
background = "desktop/packaging/macOS/DMG Background.tiff"
window_rect = ((0, 1_000_000), (640, 480))
