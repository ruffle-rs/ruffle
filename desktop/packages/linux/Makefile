# Required tools: itstool gettext appstream
#
# See also:
#   * <https://specifications.freedesktop.org/desktop-entry-spec/latest/localized-keys.html>
#   * <https://freedesktop.org/software/appstream/docs/sect-Quickstart-Translation.html>

DESKTOP_FILE := rs.ruffle.Ruffle.desktop
METAINFO_FILE := rs.ruffle.Ruffle.metainfo.xml

DESKTOP_POT_PATH := locale/$(DESKTOP_FILE).pot
METAINFO_POT_PATH := locale/$(METAINFO_FILE).pot

METAINFO_PO_FILES := $(wildcard locale/$(METAINFO_FILE)/*.po)
DESKTOP_PO_FILES := $(wildcard locale/$(DESKTOP_FILE)/*.po)
DESKTOP_LINGUAS_FILE = locale/$(DESKTOP_FILE)/LINGUAS

_default: update apply

update:
	$(MAKE) -B $(DESKTOP_POT_PATH) $(METAINFO_POT_PATH)

apply:
	$(MAKE) -B $(METAINFO_FILE) $(DESKTOP_FILE)

.PHONY: _default update apply

# Create a PO template (.pot) file from the original file template
locale/%.pot: %.in
	mkdir -p locale
	xgettext -o "$@" "$<" --itstool --add-comments

# Generate MO from PO if needed (itstool needs it)
%.mo: %.po
	msgfmt -o "$@" "$<"

# Generate LINGUAS automatically based on .po files
$(DESKTOP_LINGUAS_FILE): $(DESKTOP_PO_FILES)
	mkdir -p locale/$(DESKTOP_FILE)
	for f in $^; do basename "$${f%.*}"; done > "$@"

# Generate metainfo using itstool
$(METAINFO_FILE): $(METAINFO_PO_FILES:.po=.mo)
	itstool -l metainfo -o "$@" -j "$@.in" $^

# Generate desktop file using msgfmt
$(DESKTOP_FILE): $(DESKTOP_LINGUAS_FILE)
	msgfmt --desktop -o "$@" --template "$@.in" -d locale/$(DESKTOP_FILE)
