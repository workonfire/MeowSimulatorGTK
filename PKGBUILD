pkgname=meow-simulator
pkgver=1.0.0
pkgrel=1
pkgdesc="A boykisser on your computer"
arch=('any')
depends=('pyside6' 'ffmpeg')
source=(
  'gif.gif'
  'icon.qrc'
  'main.py'
  'meow1.mp3'
  'meow2.mp3'
  'meow3.mp3'
  'meow4.mp3'
  'static2.gif'
  'static2.png'
  'static.png'
  'window.ui'
  'meow-simulator.desktop'
)
sha256sums=('da377fe5b26e3b98d251a10d2f9456b9996e28ed916033ab296cf8b6a13b2481'
            'b1816dee92b26dc0ddce4a2264d33603a8b1831796688d5fe89f6e662aa92a4a'
            '8179016f9fbaada3436ff25f2ef8b8f84091cc54814410385d57a20933073280'
            '90fe347642159b3d429a7808559457647b7b3f044c13dd37712cbe583c7ee34c'
            'dc26bd73a2617b9bfb88e946f7b23a5c91f5b93ac0a996d045bd84c10fe4e312'
            '0929a7e2a946f521328746b4285e4832817a478bf750f355187b81f9fc8faa50'
            '7b61b978d29a9c7f4dfee7ec11e4f68b27bf8c1424505f3620d5496f8d7763e2'
            '288e2e43c286fb278b6b12722c30b486d0082759a479bf8b3f8c48eb75cc4696'
            '74ac8e7f8c8433624ed9f8ea3e0bb6631f3dca16cc8a7c4b3c627d452fc8e130'
            '136d024eb34221bd0f48888d2277d541d5660c5faa5541ad8bb8100483fbb368'
            '4376d942e91b48089c3ee1962f4ff22ae3899d740527c973c3b245e019058930'
            '98490fbd0d10eae4a844eda0d84bb592a512b7373ef2db009d135ff02cf78233')

package() {
  install -Dm644 gif.gif "$pkgdir/usr/share/$pkgname/gif.gif"
  install -Dm644 icon.qrc "$pkgdir/usr/share/$pkgname/icon.qrc"
  install -Dm644 main.py "$pkgdir/usr/bin/$pkgname"
  install -Dm644 meow1.mp3 "$pkgdir/usr/share/$pkgname/meow1.mp3"
  install -Dm644 meow2.mp3 "$pkgdir/usr/share/$pkgname/meow2.mp3"
  install -Dm644 meow3.mp3 "$pkgdir/usr/share/$pkgname/meow3.mp3"
  install -Dm644 meow4.mp3 "$pkgdir/usr/share/$pkgname/meow4.mp3"
  install -Dm644 static2.gif "$pkgdir/usr/share/$pkgname/static2.gif"
  install -Dm644 static2.png "$pkgdir/usr/share/$pkgname/static2.png"
  install -Dm644 static.png "$pkgdir/usr/share/$pkgname/static.png"
  install -Dm644 window.ui "$pkgdir/usr/share/$pkgname/window.ui"
  install -Dm644 meow-simulator.desktop "$pkgdir/usr/share/applications/$pkgname.desktop"

  chmod +x "$pkgdir/usr/bin/$pkgname"
}
