pkgname=meow-simulator
pkgver=1.0.2
pkgrel=1
pkgdesc="A boykisser on your computer"
arch=('x86_64')
depends=('gtk4')
makedepends=('rust' 'cargo')
options=('!strip')
source=('meow-simulator.desktop')
sha256sums=('98490fbd0d10eae4a844eda0d84bb592a512b7373ef2db009d135ff02cf78233')

build() {
  cd "$startdir"
  cargo build --release --locked
}

package() {
  install -Dm755 "$startdir/target/release/MeowSimulatorRust" "$pkgdir/usr/bin/$pkgname"

  local share="$pkgdir/usr/share/$pkgname"
  install -Dm644 "$startdir/assets/gif.gif"     "$share/gif.gif"
  install -Dm644 "$startdir/assets/meow1.mp3"   "$share/meow1.mp3"
  install -Dm644 "$startdir/assets/meow2.mp3"   "$share/meow2.mp3"
  install -Dm644 "$startdir/assets/meow3.mp3"   "$share/meow3.mp3"
  install -Dm644 "$startdir/assets/meow4.mp3"   "$share/meow4.mp3"
  install -Dm644 "$startdir/assets/purr.mp3"    "$share/purr.mp3"
  install -Dm644 "$startdir/assets/static2.gif" "$share/static2.gif"
  install -Dm644 "$startdir/assets/static2.png" "$share/static2.png"
  install -Dm644 "$startdir/assets/static.png"  "$share/static.png"

  install -Dm644 meow-simulator.desktop "$pkgdir/usr/share/applications/$pkgname.desktop"
}
