# Types

Fact structs mirror the daemon `sys.fico`/`os.fico` sections. Every hardware
field that has no guaranteed source is `Option`, unknown data is `None`,
never a placeholder string. Parsing uses Foundation's std-only
`JsonDocument`, so this crate has no `serde` dependency; `from_json` takes a
`foundation::serialization::JsonDocument`.

## `RamType`

```rust
pub enum RamType {
  Ddr4,
  Ddr5,
  Unknown,
}
```

| Variant | Value | Meaning |
|---|---|---|
| `Ddr4` | `DDR4` | DDR4 detected |
| `Ddr5` | `DDR5` | DDR5 detected |
| `Unknown` | `unknown` | Anything else or undetectable |

```rust
impl RamType {
  pub fn as_str(&self) -> &'static str;
  pub fn from_str(raw: &str) -> Self;
}
```

`from_str` is case-insensitive and maps every non-DDR4/DDR5 input to
`Unknown`. Returns the parsed variant, never fails.

## `Cpu`

```rust
pub struct Cpu {
  pub name: Option<String>,
  pub vendor: Option<String>,
  pub cores: Option<u32>,
  pub threads: Option<u32>,
  pub mhz: Option<u64>,
}
```

All CPU facts are cheap (`/proc/cpuinfo`) and stay in the basis view.

## `Gpu`

```rust
pub struct Gpu {
  pub name: Option<String>,
  pub vendor_id: Option<String>,
  pub device_id: Option<String>,
  pub vram_mb: Option<u64>,
  pub pci_slot: Option<String>,
  pub driver: Option<String>,
  pub subsystem: Option<String>,
}
```

| Field group | Basis | Detailed |
|---|---|---|
| `vendor_id`, `device_id`, `vram_mb` | Included | Included |
| `name`, `pci_slot`, `driver`, `subsystem` | Dropped (`None`) | Included |

`vram_mb` is `None` for shared-memory GPUs.

## `Ram` and `RamModule`

```rust
pub struct Ram {
  pub total_mb: u64,
  pub total_gb: f64,
  pub ram_type: RamType,
  pub slots_used: Option<u32>,
  pub slots_total: Option<u32>,
  pub modules: Vec<RamModule>,
}
```

```rust
pub struct RamModule {
  pub locator: Option<String>,
  pub size_mb: Option<u64>,
  pub speed_mts: Option<u64>,
  pub manufacturer: Option<String>,
  pub part_number: Option<String>,
}
```

| Field group | Basis | Detailed |
|---|---|---|
| `total_mb`, `total_gb` | Included | Included |
| `ram_type` | `Unknown` | As collected |
| `slots_used`, `slots_total`, `modules` | Dropped/empty | Included |

## `Hardware`

```rust
pub struct Hardware {
  pub cpu: Cpu,
  pub gpus: Vec<Gpu>,
  pub ram: Ram,
}
```

```rust
impl Hardware {
  pub fn from_json(result: &JsonDocument) -> Hardware;
  pub fn without_details(&self) -> Hardware;
}
```

- `from_json` parses a daemon `get_hardware` result. GPUs are collected from
  `gpu0`..`gpuN` until the first gap, modules from `module0`..`moduleN`
  (array form accepted too). Never fails, missing sections stay empty.
- `without_details` returns the basis view described above.

## `Os`

```rust
pub struct Os {
  pub name: String,
  pub display_name: String,
  pub codename: String,
  pub version: String,
  pub beta: bool,
}
```

```rust
impl Os {
  pub fn from_json(result: &JsonDocument) -> Os;
}
```

`from_json` parses a daemon `get_os` result. Missing fields fall back to
the compiled defaults (`TontooOS Seal 27.0.0`, `beta=false`). Never fails.

## `ThemeMode`, `Accent` and `Customize`

```rust
pub enum ThemeMode {
  Dark,
  Light,
}
```

```rust
pub enum Accent {
  Multicolor,
  Blue,
  Red,
  Orange,
  Yellow,
  Green,
  Teal,
  Cyan,
  Indigo,
  Purple,
  Purple2,
  Pink,
  Gray,
}
```

```rust
pub enum GlassAmount {
  Much,
  Glass,
  Less,
}
```

```rust
pub struct Customize {
  pub wallpaper: String,
  pub accent: Accent,
  pub theme: ThemeMode,
  pub glass: GlassAmount,
  pub revision: u64,
}
```

`GlassAmount::from_str` falls back to `Glass`; `as_str` round-trips the
daemon spelling (`much` / `glass` / `less`).

```rust
impl Customize {
  pub fn from_json(result: &JsonDocument) -> Customize;
}
```

`ThemeMode::from_str` falls back to `Dark`, `Accent::from_str` falls back
to `Multicolor`. `Accent::as_str` round-trips the daemon spelling and
`Accent::hex` returns the Settings app display hex (`Multicolor` renders
blue `#007AFF`). `Customize::from_json` parses a daemon `customize_get`
result; unknown or missing values fall back to the daemon defaults
(`THAOELAKE` / multicolor / dark / revision 0). Never fails.

## Usage / Example

```rust
use coresettings::{Hardware, Os};
use foundation::serialization::JsonDocument;

let reply = JsonDocument::parse(raw)?;
let hardware = Hardware::from_json(&reply);
let basis = hardware.without_details();
assert_eq!(basis.ram.ram_type.as_str(), "unknown");
```

```rust
use coresettings::Customize;

let customize = Customize::from_json(&reply);
assert_eq!(customize.accent.hex(), "#007AFF");
println!("theme={} revision={}", customize.theme.as_str(), customize.revision);
```

## Cross References

- [Provider.md](Provider.md) – queries returning these types
