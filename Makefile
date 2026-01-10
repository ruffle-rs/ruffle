.PHONY: all clean distclean ruffle_desktop ruffle_webext install uninstall version deb

SI := -i ''
ifeq ($(shell sed --version 2>/dev/null | head -1 | grep -q GNU && echo GNU),GNU)
	SI := -i
endif

DEBIAN_DIR := desktop/packages/linux/debian
DEBFULLNAME ?= unknown
DEBEMAIL ?= unknown@localhost
DEBDATE ?= $(shell date -R)
DEBSUITE ?= unstable

prefix ?= /usr/local

VERSION := $(shell cargo metadata --format-version=1 --no-deps --offline | jq -r '.packages[] | select(.name == "ruffle_desktop").version')
DEBIAN_ORIG_GZ := ../ruffle_$(VERSION).orig.tar.gz
DEBIAN_ORIG_XZ := ../ruffle_$(VERSION).orig.tar.xz
REVISION := $(shell date -d $(shell echo $(notdir $(CURDIR)) | sed 's/ruffle-//' | sed 's/nightly-//' | sed 's/$(VERSION)-//') +%y%j 2>/dev/null)
ifeq ($(REVISION),)
	REVISION := $(shell date +%y%j)
endif

all: target/release/ruffle_desktop web/packages/extension/dist/ruffle_extension.zip web/packages/extension/dist/firefox_unsigned.xpi

clean:
	-rm -f ruffle_desktop
	-rm -rf target

distclean: clean
	-find . -name "*.o" -delete
	-rm -rf *.swd RECOVER_*.fla /.idea .DS_Store
	-cd $(DEBIAN_DIR) && rm -rf ./ruffle.substvars ./.debhelper/ ./debhelper-build-stamp ./files ./ruffle/ ./ruffle.debhelper.log ./tmp/

ruffle_desktop: target/release/ruffle_desktop

target/release/ruffle_desktop:
	cargo build --release --package=ruffle_desktop

ruffle_webext: web/packages/extension/dist/ruffle_extension.zip web/packages/extension/dist/firefox_unsigned.xpi

web/packages/extension/dist/ruffle_extension.zip web/packages/extension/dist/firefox_unsigned.xpi:
	cd web && npm install
	cd web && CARGO_FEATURES=jpegxr WASM_SOURCE=cargo_and_store npm run build:dual-wasm-repro

install: target/release/ruffle_desktop web/packages/extension/dist/ruffle_extension.zip web/packages/extension/dist/firefox_unsigned.xpi
	install -d $(DESTDIR)$(prefix)/bin/
	install -m755 target/release/ruffle_desktop $(DESTDIR)$(prefix)/bin/
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
	install -d $(DESTDIR)$(prefix)/share/webext/
	install -d $(DESTDIR)$(prefix)/share/webext/ruffle-flash-emulator/
	install -d $(DESTDIR)$(prefix)/share/chromium/
	install -d $(DESTDIR)$(prefix)/share/chromium/extensions/
	install -d $(DESTDIR)$(prefix)/share/mozilla/
	install -d $(DESTDIR)$(prefix)/share/mozilla/extensions/
	install -d $(DESTDIR)$(prefix)/share/mozilla/extensions/{ec8030f7-c20a-464f-9b0e-13a3a9e97384}/
	unzip -o web/packages/extension/dist/ruffle_extension.zip -d $(DESTDIR)$(prefix)/share/webext/ruffle-flash-emulator/
	ln -sf /usr/share/webext/ruffle-flash-emulator $(DESTDIR)$(prefix)/share/chromium/extensions/
	install -m644 web/packages/extension/dist/firefox_unsigned.xpi $(DESTDIR)$(prefix)/share/webext/ruffle@ruffle.rs.xpi
	ln -sf /usr/share/webext/ruffle@ruffle.rs.xpi $(DESTDIR)$(prefix)/share/mozilla/extensions/{ec8030f7-c20a-464f-9b0e-13a3a9e97384}/

uninstall:
	-rm -f $(DESTDIR)$(prefix)/bin/ruffle_desktop
	-rm -f $(DESTDIR)$(prefix)/share/applications/rs.ruffle.Ruffle.desktop
	-rm -f $(DESTDIR)$(prefix)/share/icons/hicolor/scalable/apps/rs.ruffle.Ruffle.svg
	-rm -f $(DESTDIR)$(prefix)/share/metainfo/rs.ruffle.Ruffle.metainfo.xml
	-rm -f $(DESTDIR)$(prefix)/share/chromium/extensions/ruffle-flash-emulator
	-rm -f $(DESTDIR)$(prefix)/share/mozilla/extensions/{ec8030f7-c20a-464f-9b0e-13a3a9e97384}/ruffle@ruffle.rs.xpi
	-rm -rf $(DESTDIR)$(prefix)/share/webext/ruffle-flash-emulator
	-rm -f $(DESTDIR)$(prefix)/share/webext/ruffle@ruffle.rs.xpi

version:
	@echo $(VERSION)-$(REVISION)
	@-if ! grep "$(VERSION)-$(REVISION)" $(DEBIAN_DIR)/changelog; then \
	  if [ -s $(DEBIAN_DIR)/changelog ]; then \
	    sed $(SI) '1i\\' $(DEBIAN_DIR)/changelog; \
	  else \
	    echo > $(DEBIAN_DIR)/changelog; \
	  fi; \
	  sed $(SI) '1i\ -- $(DEBFULLNAME) <$(DEBEMAIL)>  $(DEBDATE)' $(DEBIAN_DIR)/changelog; \
	  sed $(SI) '1i\\' $(DEBIAN_DIR)/changelog; \
	  touch -a .github/changelog.entries; \
	  tac .github/changelog.entries | while read line; do \
	    sed $(SI) "1i$$line" $(DEBIAN_DIR)/changelog; \
	  done; \
	  sed $(SI) '1i\\' $(DEBIAN_DIR)/changelog; \
	  sed $(SI) '1iruffle ($(VERSION)-$(REVISION)) $(DEBSUITE); urgency=medium' $(DEBIAN_DIR)/changelog; \
	fi

deb: version
	@if [ ! -s $(DEBIAN_ORIG_XZ) -a ! -s $(DEBIAN_ORIG_GZ) ]; then \
	  if [ -s ../$(notdir $(CURDIR)).tar.gz ]; then \
	    mv -v ../$(notdir $(CURDIR)).tar.gz $(DEBIAN_ORIG_GZ); \
	  else \
	    echo 'Creating $(DEBIAN_ORIG_GZ) from HEAD...' >&2; \
	    git archive --prefix=ruffle-$(VERSION)/ -o $(DEBIAN_ORIG_GZ) HEAD; \
	  fi; \
	fi
	rm -rf debian
	cp -a $(DEBIAN_DIR) ./
	dpkg-buildpackage -us -uc
