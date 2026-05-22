pkgname=meow-simulator
pkgver=1.0.2
pkgrel=1
pkgdesc="A boykisser on your computer"
arch=('x86_64')
depends=('gtk4')
makedepends=('rust')
options=('!strip')
source=()
sha256sums=()

build() {
  cd "$startdir"
  cargo build --release --locked
}

package() {
  install -Dm755 "$startdir/target/release/MeowSimulatorRust" "$pkgdir/usr/bin/$pkgname"

  local share="$pkgdir/usr/share/$pkgname"
  for f in meow1 meow2 meow3 meow4 purr; do
    install -Dm644 "$startdir/assets/$f.mp3" "$share/$f.mp3"
  done
  install -Dm644 "$startdir/assets/static.png"  "$share/static.png"
  install -Dm644 "$startdir/assets/static2.png" "$share/static2.png"

  install -Dm644 "$startdir/meow-simulator.desktop" "$pkgdir/usr/share/applications/$pkgname.desktop"
}
