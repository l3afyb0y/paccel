#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

test -f paccel.install
test -f paccel.sysusers
test ! -e maccel.install
test ! -e maccel.sysusers

test "$(awk -F= '/^pkgname=/{print $2}' PKGBUILD)" = "paccel"
grep -q '^BUILT_MODULE_NAME\[0\]="paccel"$' dkms.conf
grep -q '^g paccel - -$' paccel.sysusers

legacy=$(rg -n --hidden \
  --glob '!.git/**' \
  --glob '!target/**' \
  --glob '!driver/*.o' \
  --glob '!driver/*.ko' \
  --glob '!driver/*.cmd' \
  --glob '!driver/Module.symvers' \
  --glob '!driver/modules.order' \
  --glob '!tests/identity.test.sh' \
  '\bmaccel\b|Gnarus-G/maccel|maccel\.org' \
  Cargo.toml cli crates tui driver Makefile PKGBUILD dkms.conf README.md \
  CONTRIBUTING.md .github udev_rules paccel.install paccel.sysusers \
  | rg -v 'conflicts=.*maccel-dkms|conflicts? with (the )?maccel|while maccel is loaded|forked from|Gnarus-G/maccel|import-maccel|Import the active legacy maccel|legacy maccel|never (changes|writes to) or reloads maccel|/sys/module/maccel' \
  || true)

if [ -n "$legacy" ]; then
  printf '%s\n' "$legacy"
  echo "legacy maccel identity remains" >&2
  exit 1
fi

printf '%s\n' "paccel identity is internally consistent"
