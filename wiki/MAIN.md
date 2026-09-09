# CoreSettings – Wiki

CoreSettings is the TontooOS client library for system facts. It reads
hardware (`sys.fico`) and OS identity (`os.fico`) from the settings daemon
over its unix socket. All data comes from the daemon, a missing daemon
surfaces as an error, never as silent fallback data.

- Repository: https://github.com/TontooOS/Libs
- License: TCL
- Version: 26.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Development and usage rules |
| Provider | [Provider.md](Provider.md) | `SettingsProvider` handle, socket queries, `detailed` flag |
| Types | [Types.md](Types.md) | `Hardware`, `Os` and fact structs, basis vs detailed views |
| Errors | [Errors.md](Errors.md) | `ProviderError` variants and failure policy |
| Localization | [Localization.md](Localization.md) | Error message localization via `lang/en_us.json` and `lang/de_de.json` |

## Quick Start

```rust
use coresettings::SettingsProvider;

let provider = SettingsProvider::new();
let os = provider.os()?;
let hardware = provider.hardware(true)?;
println!("{} {} on {} GB {}", os.display_name, os.version, hardware.ram.total_gb, hardware.ram.ram_type.as_str());
```

With explicit socket path (tests, containers):

```rust
use coresettings::SettingsProvider;

let provider = SettingsProvider::with_socket("/tmp/tontoo-settings.sock");
assert!(provider.ping()?);
```

See [Provider.md](Provider.md) for details.

## Changelog

- 2026-09-07: Initial lib. `SettingsProvider` socket client (`ping`,
  `hardware`, `os`), fact types with detailed filtering, localized errors,
  `Headers/coresettings.h` stub, `print_system` example.
- 2026-09-08: Renamed `SettingsProvider` to `CoreSettings` (crate
  `coresettings`, SDK feature `CoreSettings`).
