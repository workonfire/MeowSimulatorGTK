pkgname=meow-simulator
pkgver=1.0.2
pkgrel=1
pkgdesc="A boykisser on your computer"
arch=('x86_64')
depends=('gtk4')
makedepends=('rust' 'cargo')
source=(
  'Cargo.toml'
  'Cargo.lock'
  'src/main.rs'
  'meow-simulator.desktop'
  'assets/gif.gif'
  'assets/meow1.mp3'
  'assets/meow2.mp3'
  'assets/meow3.mp3'
  'assets/meow4.mp3'
  'assets/purr.mp3'
  'assets/static2.gif'
  'assets/static2.png'
  'assets/static.png'
)
sha256sums=(SKIP SKIP SKIP SKIP SKIP SKIP SKIP SKIP SKIP SKIP SKIP SKIP SKIP)

prepare() {
  mkdir -p src assets
  cp ../src/main.rs src/
  cp ../assets/*.mp3 ../assets/*.png ../assets/*.gif assets/
}

build() {
  cargo build --release --locked --manifest-path Cargo.toml
}

package() {
  install -Dm755 target/release/MeowSimulatorRust "$pkgdir/usr/bin/$pkgname"

  local share="$pkgdir/usr/share/$pkgname"
  install -Dm644 assets/gif.gif       "$share/gif.gif"
  install -Dm644 assets/meow1.mp3     "$share/meow1.mp3"
  install -Dm644 assets/meow2.mp3     "$share/meow2.mp3"
  install -Dm644 assets/meow3.mp3     "$share/meow3.mp3"
  install -Dm644 assets/meow4.mp3     "$share/meow4.mp3"
  install -Dm644 assets/purr.mp3      "$share/purr.mp3"
  install -Dm644 assets/static2.gif   "$share/static2.gif"
  install -Dm644 assets/static2.png   "$share/static2.png"
  install -Dm644 assets/static.png    "$share/static.png"

  install -Dm644 meow-simulator.desktop "$pkgdir/usr/share/applications/$pkgname.desktop"
}
