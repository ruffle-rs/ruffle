.PHONY: all clean distclean install uninstall version deb

DEBIAN_DIR=desktop/packages/linux/debian
prefix ?= /usr/local

VERSION := $(shell echo $(notdir $(CURDIR)) | tr -cd '0-9')
ifeq ($(VERSION),)
	VERSION := $(shell date +%Y%m%d)
endif

all: ruffle_desktop

clean:
	-rm -f ruffle_desktop
	-rm -rf target

distclean: clean
	-find . -name "*.o" -delete
	-rm -rf *.swd RECOVER_*.fla /.idea .DS_Store
	-cd $(DEBIAN_DIR) && rm -rf ./ruffle.substvars ./.debhelper/ ./debhelper-build-stamp ./files ./ruffle/ ./ruffle.debhelper.log ./tmp/

ruffle_desktop:
	cargo build --release --package=$@
	install -m755 target/release/$@ ./$@

install: ruffle_desktop
	install -d $(DESTDIR)$(prefix)/bin/
	install -m755 $^ $(DESTDIR)$(prefix)/bin/
	install -d $(DESTDIR)$(prefix)/share/
	install -d $(DESTDIR)$(prefix)/share/applications/
	install -m644 desktop/packages/linux/rs.ruffle.Ruffle.desktop $(DESTDIR)$(prefix)/share/applications/
	install -d $(DESTDIR)$(prefix)/share/metainfo/
	install -m644 desktop/packages/linux/rs.ruffle.Ruffle.metainfo.xml $(DESTDIR)$(prefix)/share/metainfo/
	install -d $(DESTDIR)$(prefix)/share/icons/
	install -d $(DESTDIR)$(prefix)/share/icons/hicolor/
	install -d $(DESTDIR)$(prefix)/share/icons/hicolor/scalable/
	install -d $(DESTDIR)$(prefix)/share/icons/hicolor/scalable/apps/
	install -m644 desktop/packages/linux/rs.ruffle.Ruffle.svg $(DESTDIR)$(prefix)/share/icons/hicolor/scalable/apps/

uninstall:
	-rm -f $(DESTDIR)$(prefix)/bin/ruffle_desktop

version:
	@echo $(VERSION)
	@-sed -i '1s/([0-9]\{8\})/($(VERSION))/' $(DEBIAN_DIR)/changelog

deb: version
	-ln -s $(DEBIAN_DIR) debian
	dpkg-buildpackage -us -uc
