# Localization

Error messages are localized through `lang/en_us.json` and
`lang/de_de.json`. The locale is picked once per process from `LC_ALL`,
`LC_MESSAGES` or `LANG` (German when starting with `de`, English otherwise).

## `lang` Module

```rust
pub fn current_locale() -> &'static str;
pub fn t(key: &str) -> String;
```

| Key | `en_us` | `de_de` |
|---|---|---|
| `socket_missing` | `Settings daemon socket not found` | `Settings-Daemon-Socket nicht gefunden` |
| `connection_failed` | `Failed to connect to the settings daemon` | `Verbindung zum Settings-Daemon fehlgeschlagen` |
| `protocol_error` | `Settings daemon protocol error` | `Settings-Daemon-Protokollfehler` |
| `server_error` | `Settings daemon reported an error` | `Settings-Daemon meldet einen Fehler` |
| `parse_error` | `Failed to parse the settings daemon reply` | `Antwort des Settings-Daemon konnte nicht gelesen werden` |

- `current_locale` returns `de_de` or `en_us`. Never fails.
- `t` returns the message for `key`, or the key itself when missing. Never
  fails.

## Usage / Example

```rust
use coresettings::lang;

std::env::set_var("LANG", "de_DE.UTF-8");
assert_eq!(lang::current_locale(), "de_de");
```

## Cross References

- [Errors.md](Errors.md) – errors rendered through `t`
