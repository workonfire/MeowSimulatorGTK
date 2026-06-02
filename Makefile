BINARY  := MeowSimulatorRust
RELEASE := target/release
DESTDIR ?=

DIST_WIN  := dist/windows
DIST_LIN  := dist/linux
ZIP       := dist/meow-simulator-windows.zip
SETUP_EXE := dist/meow-simulator-setup.exe
TARBALL   := dist/meow-simulator-linux.tar.gz

OS := $(shell uname -s)

.PHONY: all package package-windows package-linux stage-linux pkgbuild install uninstall build check-rust check-ucrt64 clean

all: package

package:
ifeq ($(OS),Linux)
	$(MAKE) package-linux
else ifeq ($(OS),Windows_NT)
	$(MAKE) package-windows
else
	$(error Unsupported OS: $(OS). Use 'make package-linux' or 'make package-windows' explicitly.)
endif

# ── Windows ───────────────────────────────────────────────────────────────────

check-rust:
	@command -v rustc >/dev/null 2>&1 \
	  || { echo "error: rustc not found — install Rust from https://rustup.rs or: pacman -S mingw-w64-ucrt-x86_64-rust"; exit 1; }
	@RUSTC_VV=$$(rustc -vV 2>/dev/null); \
	echo "$$RUSTC_VV" | grep -q 'host:.*-gnu' \
	  || { echo "error: Rust GNU toolchain required — current host: $$(echo "$$RUSTC_VV" | grep host)"; \
	       echo "       install with: pacman -S mingw-w64-ucrt-x86_64-rust"; exit 1; }

check-ucrt64: check-rust
	@test -d /ucrt64 \
	  || { echo "error: /ucrt64 not found — install the MSYS2 UCRT64 toolchain"; exit 1; }
	@for pkg in mingw-w64-ucrt-x86_64-gtk4 mingw-w64-ucrt-x86_64-pkgconf \
	            mingw-w64-ucrt-x86_64-gcc mingw-w64-ucrt-x86_64-vulkan-loader \
	            mingw-w64-ucrt-x86_64-gstreamer mingw-w64-ucrt-x86_64-gst-plugins-base \
	            mingw-w64-ucrt-x86_64-gst-plugins-good mingw-w64-ucrt-x86_64-gst-plugins-bad \
	            mingw-w64-ucrt-x86_64-libadwaita; do \
	  pacman -Q $$pkg >/dev/null 2>&1 \
	    || { echo "error: $$pkg not installed — run: pacman -S $$pkg"; exit 1; }; \
	done
	@command -v zip >/dev/null 2>&1 \
	  || { echo "error: zip not found — run: pacman -S zip"; exit 1; }

package-windows: $(ZIP) $(SETUP_EXE)

$(ZIP): check-ucrt64 build
	rm -rf $(DIST_WIN)
	mkdir -p $(DIST_WIN)/share/glib-2.0
	cp $(RELEASE)/$(BINARY).exe $(DIST_WIN)/
	cp $(RELEASE)/installer.exe $(DIST_WIN)/
	cp $(RELEASE)/uninstaller.exe $(DIST_WIN)/
	ldd $(RELEASE)/$(BINARY).exe \
	  | grep -i ucrt64 \
	  | awk '{print $$3}' \
	  | xargs -I{} cp {} $(DIST_WIN)/
	cp /ucrt64/bin/vulkan-1.dll $(DIST_WIN)/
	cp -r /ucrt64/share/glib-2.0/schemas $(DIST_WIN)/share/glib-2.0/
	mkdir $(DIST_WIN)/share/icons
	cp -r /ucrt64/share/icons/Adwaita $(DIST_WIN)/share/icons/
	glib-compile-schemas $(DIST_WIN)/share/glib-2.0/schemas/
	cp -r $(RELEASE)/assets $(DIST_WIN)/
	mkdir -p $(DIST_WIN)/lib/gstreamer-1.0
	for plugin in libgstcoreelements libgstplayback libgsttypefindfunctions \
	              libgstaudioconvert libgstaudioresample libgstvolume \
	              libgstautodetect libgstogg libgstopus \
	              libgstwasapi2 libgstgio; do \
	  cp /ucrt64/lib/gstreamer-1.0/$$plugin.dll $(DIST_WIN)/lib/gstreamer-1.0/; \
	done
	for dll in $(DIST_WIN)/lib/gstreamer-1.0/*.dll; do \
	  ldd "$$dll" 2>/dev/null | grep -i ucrt64 | awk '{print $$3}' | xargs -I{} cp -n {} $(DIST_WIN)/; \
	done
	mkdir -p $(DIST_WIN)/lib/gdk-pixbuf-2.0/2.10.0/loaders
	cp /ucrt64/lib/gdk-pixbuf-2.0/2.10.0/loaders/libpixbufloader-png.dll \
	   $(DIST_WIN)/lib/gdk-pixbuf-2.0/2.10.0/loaders/
	ldd /ucrt64/lib/gdk-pixbuf-2.0/2.10.0/loaders/libpixbufloader-png.dll 2>/dev/null \
	  | grep -i ucrt64 | awk '{print $$3}' | xargs -I{} cp -n {} $(DIST_WIN)/
	cd dist && zip -r meow-simulator-windows.zip windows/

$(SETUP_EXE): $(ZIP)
	BUNDLE_ZIP=$$(cd dist && pwd)/meow-simulator-windows.zip cargo build --release --bin setup
	cp $(RELEASE)/setup.exe $(SETUP_EXE)

# ── Linux ─────────────────────────────────────────────────────────────────────

package-linux: $(TARBALL)

stage-linux: build
	rm -rf $(DIST_LIN)
	mkdir -p $(DIST_LIN)/usr/bin $(DIST_LIN)/usr/share/meow-simulator \
	         $(DIST_LIN)/usr/share/icons/hicolor/256x256/apps $(DIST_LIN)/usr/share/applications
	cp $(RELEASE)/$(BINARY) $(DIST_LIN)/usr/bin/meow-simulator
	cp -r $(RELEASE)/assets/. $(DIST_LIN)/usr/share/meow-simulator/
	cp $(RELEASE)/assets/static.png $(DIST_LIN)/usr/share/icons/hicolor/256x256/apps/meow-simulator.png
	cp com.wzium.MeowSimulator.desktop $(DIST_LIN)/usr/share/applications/

$(TARBALL): stage-linux
	tar -czf $(TARBALL) -C dist linux/

pkgbuild: package-linux
	makepkg -f

# ── Install ───────────────────────────────────────────────────────────────────

install: stage-linux
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
