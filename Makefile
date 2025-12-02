.PHONY: all clean distclean install uninstall version deb

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
REVISION := $(shell echo $(notdir $(CURDIR)) | sed 's/$(VERSION)//' | tr -cd '0-9')
ifeq ($(REVISION),)
	REVISION := $(shell date +%Y%m%d)
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
	@echo $(VERSION)-$(REVISION)
	@-if ! grep "$(VERSION)-$(REVISION)" $(DEBIAN_DIR)/changelog; then \
	  sed $(SI) '1i\\' $(DEBIAN_DIR)/changelog; \
	  sed $(SI) '1i\ -- $(DEBFULLNAME) <$(DEBEMAIL)>  $(DEBDATE)' $(DEBIAN_DIR)/changelog; \
	  sed $(SI) '1i\\' $(DEBIAN_DIR)/changelog; \
	  cat .github/changelog.entries | while read line; do \
	    sed $(SI) "1i\ \ * $$line" $(DEBIAN_DIR)/changelog; \
	  done; \
	  sed $(SI) '1i\\' $(DEBIAN_DIR)/changelog; \
	  sed $(SI) '1iruffle ($(VERSION)-$(REVISION)) $(DEBSUITE); urgency=medium' $(DEBIAN_DIR)/changelog; \
	fi

deb: version
	@if [ ! -s $(DEBIAN_ORIG_XZ) -a ! -s $(DEBIAN_ORIG_GZ) ]; then \
	  echo 'Creating $(DEBIAN_ORIG_GZ) from HEAD...' >&2; \
	  git archive --prefix=ruffle-$(VERSION)/ -o $(DEBIAN_ORIG_GZ) HEAD; \
	fi
	rm -rf debian
	cp -a $(DEBIAN_DIR) ./
	dpkg-buildpackage -us -uc
