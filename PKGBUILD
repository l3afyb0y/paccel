pkgname=paccel
_pkgname=paccel
pkgver=0.1.0
pkgrel=1
pkgdesc="Personal Linux mouse acceleration driver, CLI, and TUI"
arch=("x86_64")
url="https://github.com/l3afyb0y/paccel"
license=("GPL-2.0-or-later")

install=paccel.install
depends=("dkms")
makedepends=("cargo" "git")
conflicts=("maccel-dkms")
provides=("paccel-dkms=${pkgver}")

# DEBUG_CFLAGS="$DEBUG_CFLAGS -DDEBUG"
options=(!debug !lto)

source=("paccel::git+https://github.com/l3afyb0y/paccel.git")
sha256sums=("SKIP")

prepare() {
  export RUSTUP_TOOLCHAIN=stable

  platform="$(rustc -vV | sed -n 's/host: //p')"

  cargo fetch --locked --target "${platform}" --manifest-path="${srcdir}/paccel/Cargo.toml"
}

build() {
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target

  # Build the CLI
  cargo build --bin paccel --release --frozen --manifest-path="${srcdir}/paccel/Cargo.toml"
}

package() {
  # Add group
  install -Dm 644 "${srcdir}/paccel/paccel.sysusers" "${pkgdir}/usr/lib/sysusers.d/${_pkgname}.conf"

  # Install Driver
  install -Dm 644 "${srcdir}/paccel/dkms.conf" "${pkgdir}/usr/src/${_pkgname}-${pkgver}/dkms.conf"

  # Escape path separators from debug flags values
  DRIVER_CFLAGS=$(echo ${DEBUG_CFLAGS} | sed -e "s/\//\\\\\\//g")

  # Set the package version and build flags for DKMS.
  sed -e "s/@PKGVER@/${pkgver}/" \
    -e "s/@DRIVER_CFLAGS@/'${DRIVER_CFLAGS}'/" \
    -i "${pkgdir}/usr/src/${_pkgname}-${pkgver}/dkms.conf"

  cp -r "${srcdir}/paccel/driver/." "${pkgdir}/usr/src/${_pkgname}-${pkgver}/"

  # Install CLI
  install -Dm 755 "${srcdir}/target/release/paccel" "${pkgdir}/usr/bin/paccel"

  # Install udev rules
  install -Dm 644 "${srcdir}/paccel/udev_rules/99-paccel.rules" "${pkgdir}/usr/lib/udev/rules.d/99-paccel.rules"
  install -Dm 644 "${srcdir}/paccel/systemd/paccel-apply.service" "${pkgdir}/usr/lib/systemd/user/paccel-apply.service"

  # Install License
  install -Dm 644 "${srcdir}/paccel/LICENSE" "${pkgdir}/usr/share/licenses/${_pkgname}/LICENSE"
}
