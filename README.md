# paccel

paccel is Parker's personal Linux mouse-acceleration driver, forked from
[maccel](https://github.com/Gnarus-G/maccel). It contains a DKMS kernel module,
a CLI, and a terminal UI. The project is public for inspection and personal use;
it is not presented as a broadly supported distribution.

The fork has its own `paccel` module, device, group, executable, DKMS identity,
and configuration directory. It conflicts with the maccel DKMS package because
two input filters should not be active together.

## Safety model

- Configuration is validated and replaced atomically through `/dev/paccel`.
- Motion timing, fractional carry, and frame aggregation are per input device.
- Only relative pointer devices are matched.
- Installing the package does not load, unload, or reload either driver.
- `paccel import-maccel` only reads the legacy driver's active values and writes
  a paccel TOML file. It never changes or reloads maccel.
- Runtime driver tests belong in a VM, not on the development host.

Linux 6.11 or newer is required by the input-handler implementation.

## Arch package

Install the normal Arch build dependencies, matching headers for every kernel
where DKMS should build the module, and Rust. Then build the package:

```fish
makepkg --cleanbuild
sudo pacman -U ./paccel-0.1.0-1-x86_64.pkg.tar.zst
```

Package installation is intentionally inert. When you choose to activate the
driver, load `paccel` yourself. Add your user to the `paccel` group if the CLI
should access `/dev/paccel` without root, then log out and back in.

## Configuration

The CLI reads and writes the entire driver configuration as one transaction:

```fish
paccel get mode
paccel set mode linear
paccel save
paccel load
paccel tui
```

`paccel save` creates `$XDG_CONFIG_HOME/paccel/config.toml`, falling back to
`~/.config/paccel/config.toml`, with mode `0600`. It refuses to overwrite an
existing file unless `--force` is given. `paccel load --if-present` is suitable
for the included user service.

To capture settings from a currently loaded legacy driver without applying them:

```fish
paccel import-maccel
```

Use `--force` only when replacing the existing paccel configuration is intended.

## Development

Host-safe checks:

```fish
make test
cargo test --workspace
make build
bash tests/identity.test.sh
```

None of these commands loads the module. Do not use `modprobe`, `rmmod`, or an
install target on a workstation whose existing pointer setup must remain intact.

The acceleration math currently includes Linear, Natural, Synchronous, and
NoAccel modes. Input DPI normalization, axis ratio, rotation, output caps, and
fixed-point fractional carry are supported.

## License

GPL-2.0-or-later. See [LICENSE](LICENSE).
