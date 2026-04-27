pkgname=meow-simulator
pkgver=1.0.2
pkgrel=2
pkgdesc="A boykisser on your computer"
arch=('any')
depends=('pyside6' 'ffmpeg')
source=(
  'main.py'
  'meow-simulator.desktop'
)
sha256sums=('e7ff9c8d570ff886d110bc0acc22515f7da06ebbe37f8870fb72743dbb3f2e53'
            '98490fbd0d10eae4a844eda0d84bb592a512b7373ef2db009d135ff02cf78233')

prepare() {
  cp -r ../assets/* .
}

package() {
  install -Dm644 gif.gif "$pkgdir/usr/share/$pkgname/gif.gif"
  install -Dm644 icon.qrc "$pkgdir/usr/share/$pkgname/icon.qrc"
  install -Dm644 main.py "$pkgdir/usr/bin/$pkgname"
  install -Dm644 meow1.mp3 "$pkgdir/usr/share/$pkgname/meow1.mp3"
  install -Dm644 meow2.mp3 "$pkgdir/usr/share/$pkgname/meow2.mp3"
  install -Dm644 meow3.mp3 "$pkgdir/usr/share/$pkgname/meow3.mp3"
  install -Dm644 meow4.mp3 "$pkgdir/usr/share/$pkgname/meow4.mp3"
  install -Dm644 purr.mp3 "$pkgdir/usr/share/$pkgname/purr.mp3"
  install -Dm644 static2.gif "$pkgdir/usr/share/$pkgname/static2.gif"
  install -Dm644 static2.png "$pkgdir/usr/share/$pkgname/static2.png"
  install -Dm644 static.png "$pkgdir/usr/share/$pkgname/static.png"
  install -Dm644 window.ui "$pkgdir/usr/share/$pkgname/window.ui"
  install -Dm644 meow-simulator.desktop "$pkgdir/usr/share/applications/$pkgname.desktop"

  chmod +x "$pkgdir/usr/bin/$pkgname"
}
