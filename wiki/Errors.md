# Errors

Every provider method returns `Result<T, ProviderError>`. The policy is
strict: facts come from the daemon socket, a missing daemon is an error,
never silent fallback data.

## `ProviderError`

```rust
pub enum ProviderError {
  SocketMissing(String),
  Connection(String),
  Protocol(String),
  Server(String),
  Parse(String),
}
```

| Variant | Meaning |
|---|---|
| `SocketMissing` | Socket path does not exist, the daemon is not running |
| `Connection` | Transport failure (refused, reset, timed out after 10 seconds) |
| `Protocol` | Malformed frame from the daemon |
| `Server` | Daemon answered `ok: false`, detail carries the daemon message |
| `Parse` | Well-formed reply with unusable content |

- `Display` renders the localized message (see
  [Localization.md](Localization.md)) plus the detail.
- Implements `std::error::Error`.

```rust
pub type Result<T> = std::result::Result<T, ProviderError>;
```

## Failure Policy

| Situation | Result |
|---|---|
| Daemon not running | `Err(SocketMissing)` |
| Daemon file missing on daemon side | `Err(Server)` with the daemon message |
| Unknown op (version skew) | `Err(Server)` |
| Hanging daemon | `Err(Connection)` after 10 seconds |

## Usage / Example

```rust
use coresettings::{ProviderError, SettingsProvider};

match SettingsProvider::new().os() {
    Ok(os) => println!("{}", os.display_name),
    Err(ProviderError::SocketMissing(path)) => eprintln!("daemon down: {}", path),
    Err(e) => eprintln!("{}", e),
}
```

## Cross References

- [Provider.md](Provider.md) – methods returning these errors
- [Localization.md](Localization.md) – message languages
