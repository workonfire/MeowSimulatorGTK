BINARY  := MeowSimulatorRust
RELEASE := target/release
DESTDIR ?=

DIST_WIN  := dist/windows
DIST_LIN  := dist/linux
ZIP       := dist/meow-simulator-windows.zip
TARBALL   := dist/meow-simulator-linux.tar.gz

OS := $(shell uname -s)

.PHONY: all package package-windows package-linux pkgbuild install uninstall build clean

all: package

package:
ifeq ($(OS),Linux)
	$(MAKE) package-linux
else
	$(MAKE) package-windows
endif

# ── Windows ───────────────────────────────────────────────────────────────────

package-windows: $(ZIP)

$(ZIP): build
	rm -rf $(DIST_WIN)
	mkdir -p $(DIST_WIN)/share/glib-2.0
	cp $(RELEASE)/$(BINARY).exe $(DIST_WIN)/
	ldd $(RELEASE)/$(BINARY).exe \
	  | grep -i ucrt64 \
	  | awk '{print $$3}' \
	  | xargs -I{} cp {} $(DIST_WIN)/
	cp /ucrt64/bin/vulkan-1.dll $(DIST_WIN)/
	cp -r /ucrt64/share/glib-2.0/schemas $(DIST_WIN)/share/glib-2.0/
	cp -r /ucrt64/share/icons $(DIST_WIN)/share/
	glib-compile-schemas $(DIST_WIN)/share/glib-2.0/schemas/
	cp -r $(RELEASE)/assets $(DIST_WIN)/
	cd dist && zip -r meow-simulator-windows.zip windows/

# ── Linux ─────────────────────────────────────────────────────────────────────

package-linux: $(TARBALL)

$(TARBALL): build
	rm -rf $(DIST_LIN)
	mkdir -p $(DIST_LIN)/usr/bin
	mkdir -p $(DIST_LIN)/usr/share/meow-simulator
	mkdir -p $(DIST_LIN)/usr/share/icons/hicolor/256x256/apps
	mkdir -p $(DIST_LIN)/usr/share/applications
	cp $(RELEASE)/$(BINARY) $(DIST_LIN)/usr/bin/meow-simulator
	cp -r $(RELEASE)/assets/. $(DIST_LIN)/usr/share/meow-simulator/
	cp $(RELEASE)/assets/static.png $(DIST_LIN)/usr/share/icons/hicolor/256x256/apps/meow-simulator.png
	cp com.wzium.MeowSimulator.desktop $(DIST_LIN)/usr/share/applications/
	tar -czf $(TARBALL) -C dist linux/

pkgbuild: package-linux
	makepkg -f

# ── Install ───────────────────────────────────────────────────────────────────

install: package-linux
	cp -r $(DIST_LIN)/. $(DESTDIR)/

uninstall:
	rm -f  $(DESTDIR)/usr/bin/meow-simulator
	rm -rf $(DESTDIR)/usr/share/meow-simulator
	rm -f  $(DESTDIR)/usr/share/icons/hicolor/256x256/apps/meow-simulator.png
	rm -f  $(DESTDIR)/usr/share/applications/com.wzium.MeowSimulator.desktop

# ── Common ────────────────────────────────────────────────────────────────────

build:
	cargo build --release

clean:
	rm -rf dist/
