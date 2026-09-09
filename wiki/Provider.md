# Provider

`SettingsProvider` is the socket client for the settings daemon. One handle
owns a socket path and opens a fresh connection per request. Every method
returns `Result`, a missing daemon is an error, never empty data.

## Handle

```rust
pub struct SettingsProvider {
  socket_path: PathBuf,
}
```

### Constructors

```rust
impl SettingsProvider {
  pub fn new() -> Self;
  pub fn with_socket(path: impl Into<PathBuf>) -> Self;
  pub fn from_env() -> Self;
  pub fn socket_path(&self) -> &Path;
}
```

| Constructor | Socket path source |
|---|---|
| `new` | `/run/tontoo-settings.sock` |
| `with_socket` | Explicit path, used by tests and examples |
| `from_env` | `SETTINGS_SOCKET`, falls back to the default path |

- `new` never fails, it only stores the default path.
- `with_socket` never fails, it only stores the given path.
- `from_env` never fails. Empty `SETTINGS_SOCKET` is ignored.

```rust
let provider = SettingsProvider::from_env();
```

## Methods

```rust
impl SettingsProvider {
  pub fn ping(&self) -> Result<bool>;
  pub fn hardware(&self, detailed: bool) -> Result<Hardware>;
  pub fn os(&self) -> Result<Os>;
}
```

- `ping` returns `true` on a valid pong reply. Returns `Err` when the daemon
  is unreachable or answers malformed data.
- `hardware` reads `sys.fico` through the daemon. `detailed=false` returns
  the basis view (see [Types.md](Types.md)), `detailed=true` returns
  everything the daemon collected.
- `os` reads `os.fico` through the daemon. Missing fields fall back to the
  compiled defaults (`TontooOS Seal 26.1.0`).

## Free Functions

```rust
pub fn ping() -> Result<bool>;
pub fn hardware(detailed: bool) -> Result<Hardware>;
pub fn os() -> Result<Os>;
```

Same behavior at the default socket path, without constructing a handle.

## Timeout

Each request has a 10 second socket read timeout. A hanging daemon surfaces
as `ProviderError::Connection`, never as a blocked call.

## Usage / Example

```rust
use coresettings::SettingsProvider;

let provider = SettingsProvider::new();
if provider.ping()? {
    let os = provider.os()?;
    let basis = provider.hardware(false)?;
    println!("{} {}", os.display_name, basis.ram.total_gb);
}
```

## Cross References

- [Types.md](Types.md) – shapes returned by `hardware` and `os`
- [Errors.md](Errors.md) – failure policy and error variants
