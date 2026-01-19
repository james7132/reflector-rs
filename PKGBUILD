# Maintainer: James Liu <contact@no-bull.sh>

pkgname=reflector-rs
pkgver=0.1.0
pkgrel=1
pkgdesc='Retrieve and filter the latest Arch Linux mirror list (Rust implementation)'
arch=('x86_64')
url='https://github.com/james7132/reflector-rs'
license=('GPL-2.0-or-later')
provides=('reflector')
conflicts=('reflector')
makedepends=('cargo' 'git' 'rust')
backup=('etc/xdg/reflector/reflector.conf')
source=("git+$url.git")
sha256sums=('SKIP')

prepare() {
  cd "$srcdir/reflector-rs"
  cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
  cd "$srcdir/reflector-rs"
  export RUSTUP_TOOLCHAIN=1.85
  export CARGO_TARGET_DIR=target
  cargo build --frozen --release --target "$CARCH-unknown-linux-gnu"
}

package() {
  cd "$srcdir/reflector-rs"

  # Install binary
  install -Dm755 "target/$CARCH-unknown-linux-gnu/release/reflector" \
    "$pkgdir/usr/bin/$pkgname"

  # Install systemd service and timer
  install -Dm644 "dist/reflector.service" \
    "$pkgdir/usr/lib/systemd/system/reflector.service"
  install -Dm644 "dist/reflector.timer" \
    "$pkgdir/usr/lib/systemd/system/reflector.timer"

  # Install default configuration
  install -Dm644 "dist/reflector.conf" \
    "$pkgdir/etc/xdg/reflector/reflector.conf"

  # Install man page
  install -Dm644 reflector.1 "usr/share/man/man1/reflector.1"

  # Install license
  install -Dm644 LICENSE "usr/share/licenses/$pkgname/LICENSE"
}
