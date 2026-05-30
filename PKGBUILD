pkgname=meow-simulator
pkgver=1.0.4
pkgrel=1
pkgdesc="A boykisser on your computer"
arch=('x86_64')
depends=('gtk4' 'libadwaita' 'gstreamer')
options=('!strip')
source=()
sha256sums=()

package() {
  if [[ ! -d "$startdir/dist/linux" ]]; then
    error "dist/linux not found — build it first with: make package-linux"
    return 1
  fi
  cp -r "$startdir/dist/linux/." "$pkgdir/"
}
