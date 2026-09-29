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
  pub fn customize(&self) -> Result<Customize>;
  pub fn subscribe(&self, events: &[&str]) -> Result<Subscription>;
}
```

- `ping` returns `true` on a valid pong reply. Returns `Err` when the daemon
  is unreachable or answers malformed data.
- `hardware` reads `sys.fico` through the daemon. `detailed=false` returns
  the basis view (see [Types.md](Types.md)), `detailed=true` returns
  everything the daemon collected.
- `os` reads `os.fico` through the daemon. Missing fields fall back to the
  compiled defaults (`TontooOS Seal 27.0.0`).
- `customize` reads the effective customization (`customize_get`):
  wallpaper, accent, theme plus the revision counter for change polling.
- `subscribe` opens one persistent connection (`subscribe` op) and returns
  a `Subscription`. The daemon pushes a frame per matching write op, so no
  polling is needed. An empty list receives every event.

## Subscription

```rust
pub struct DaemonEvent {
  pub name: String,
  pub payload: JsonDocument,
}

impl Subscription {
  pub fn try_next(&mut self) -> Result<Option<DaemonEvent>>;
}
```

- `try_next` never blocks: `Ok(None)` means no event arrived yet.
- `Err` means the daemon closed the connection or sent a malformed frame;
  drop the handle and resubscribe.
- `payload` holds the write-op result (e.g. the effective customization
  with `revision` for `customize_changed`); parse it with
  `Customize::from_json`.

```rust
use coresettings::SettingsProvider;

let provider = SettingsProvider::new();
let mut events = provider.subscribe(&["customize_changed"])?;
while let Some(event) = events.try_next()? {
    let customize = coresettings::Customize::from_json(&event.payload);
    println!("theme is now {:?}", customize.theme);
}
```

## Free Functions

```rust
pub fn ping() -> Result<bool>;
pub fn hardware(detailed: bool) -> Result<Hardware>;
pub fn os() -> Result<Os>;
pub fn customize() -> Result<Customize>;
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

- [Types.md](Types.md) – shapes returned by `hardware`, `os` and `customize`
- [Errors.md](Errors.md) – failure policy and error variants
