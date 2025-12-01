.PHONY: all clean distclean install uninstall version deb

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

ruffle_desktop:
	cargo build --release --package=$@
	install -m755 target/release/$@ ./$@

install: ruffle_desktop
	install -d $(DESTDIR)$(prefix)/bin/
	install -m755 $^ $(DESTDIR)$(prefix)/bin/

uninstall:
	-rm -f $(DESTDIR)$(prefix)/bin/ruffle_desktop

version:
	@echo $(VERSION)
	@sed -i '1s/([0-9]\{8\})/($(VERSION))/' debian/changelog

deb: version
	dpkg-buildpackage -us -uc
