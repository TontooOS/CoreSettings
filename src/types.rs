use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// RAM type (DDR4/DDR5 only, like the daemon backend)
// ---------------------------------------------------------------------------

/// Classified RAM type. Only DDR4 and DDR5 are distinguished.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RamType {
  Ddr4,
  Ddr5,
  Unknown,
}

impl RamType {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Ddr4 => "DDR4",
      Self::Ddr5 => "DDR5",
      Self::Unknown => "unknown",
    }
  }

  pub fn from_str(raw: &str) -> Self {
    match raw.trim().to_uppercase().as_str() {
      "DDR4" => Self::Ddr4,
      "DDR5" => Self::Ddr5,
      _ => Self::Unknown,
    }
  }
}

// ---------------------------------------------------------------------------
// Hardware facts (mirrors sys.fico sections)
// ---------------------------------------------------------------------------

/// Processor facts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Cpu {
  pub name: Option<String>,
  pub vendor: Option<String>,
  pub cores: Option<u32>,
  pub threads: Option<u32>,
  pub mhz: Option<u64>,
}

/// Graphics device facts. `vram_mb` is `None` for shared-memory GPUs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Gpu {
  pub name: Option<String>,
  pub vendor_id: Option<String>,
  pub device_id: Option<String>,
  pub vram_mb: Option<u64>,
  /// Detailed only.
  pub pci_slot: Option<String>,
  pub driver: Option<String>,
  pub subsystem: Option<String>,
}

/// One physical RAM module. Detailed only.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RamModule {
  pub locator: Option<String>,
  pub size_mb: Option<u64>,
  pub speed_mts: Option<u64>,
  pub manufacturer: Option<String>,
  pub part_number: Option<String>,
}

/// Memory facts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ram {
  pub total_mb: u64,
  pub total_gb: f64,
  pub ram_type: RamType,
  pub slots_used: Option<u32>,
  pub slots_total: Option<u32>,
  /// Detailed only.
  pub modules: Vec<RamModule>,
}

impl Default for Ram {
  fn default() -> Self {
    Self {
      total_mb: 0,
      total_gb: 0.0,
      ram_type: RamType::Unknown,
      slots_used: None,
      slots_total: None,
      modules: Vec::new(),
    }
  }
}

/// Full hardware snapshot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Hardware {
  pub cpu: Cpu,
  pub gpus: Vec<Gpu>,
  pub ram: Ram,
}

impl Hardware {
  /// Basis view without detailed fields: GPU name/driver/slot/subsystem are
  /// dropped, RAM type falls back to `Unknown` and modules/slots are
  /// dropped. CPU facts are all cheap and stay.
  pub fn without_details(&self) -> Hardware {
    let mut basis = self.clone();
    for gpu in &mut basis.gpus {
      gpu.name = None;
      gpu.pci_slot = None;
      gpu.driver = None;
      gpu.subsystem = None;
    }
    basis.ram.ram_type = RamType::Unknown;
    basis.ram.slots_used = None;
    basis.ram.slots_total = None;
    basis.ram.modules.clear();
    basis
  }
}

// ---------------------------------------------------------------------------
// OS facts (mirrors the os.fico section)
// ---------------------------------------------------------------------------

/// OS identity facts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Os {
  pub name: String,
  pub display_name: String,
  pub codename: String,
  pub version: String,
  pub beta: bool,
}

impl Default for Os {
  fn default() -> Self {
    Self {
      name: "TontooOS".to_string(),
      display_name: "TontooOS Seal".to_string(),
      codename: "Seal".to_string(),
      version: "26.1.0".to_string(),
      beta: false,
    }
  }
}

// ---------------------------------------------------------------------------
// Customization (mirrors the daemon `customize` domain)
// ---------------------------------------------------------------------------

/// Color theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
  Dark,
  Light,
}

impl ThemeMode {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Dark => "dark",
      Self::Light => "light",
    }
  }

  pub fn from_str(raw: &str) -> Self {
    match raw {
      "light" => Self::Light,
      _ => Self::Dark,
    }
  }
}

/// Accent color. `Multicolor` is the default element and renders as blue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

impl Accent {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Multicolor => "multicolor",
      Self::Blue => "blue",
      Self::Red => "red",
      Self::Orange => "orange",
      Self::Yellow => "yellow",
      Self::Green => "green",
      Self::Teal => "teal",
      Self::Cyan => "cyan",
      Self::Indigo => "indigo",
      Self::Purple => "purple",
      Self::Purple2 => "purple2",
      Self::Pink => "pink",
      Self::Gray => "gray",
    }
  }

  pub fn from_str(raw: &str) -> Self {
    match raw {
      "blue" => Self::Blue,
      "red" => Self::Red,
      "orange" => Self::Orange,
      "yellow" => Self::Yellow,
      "green" => Self::Green,
      "teal" => Self::Teal,
      "cyan" => Self::Cyan,
      "indigo" => Self::Indigo,
      "purple" => Self::Purple,
      "purple2" => Self::Purple2,
      "pink" => Self::Pink,
      "gray" => Self::Gray,
      _ => Self::Multicolor,
    }
  }

  /// Display hex from the Settings app palette. Multicolor renders blue.
  pub fn hex(&self) -> &'static str {
    match self {
      Self::Multicolor | Self::Blue => "#007AFF",
      Self::Red => "#FF3B30",
      Self::Orange => "#FF9500",
      Self::Yellow => "#FFCC00",
      Self::Green => "#34C759",
      Self::Teal => "#00C7BE",
      Self::Cyan => "#30B0C7",
      Self::Indigo => "#5856D6",
      Self::Purple => "#AF52DE",
      Self::Purple2 => "#BF5AF2",
      Self::Pink => "#FF2D55",
      Self::Gray => "#8E8E93",
    }
  }
}

/// Liquid glass amount: the "LiquidGlass Slider" with much glass,
/// balanced glass and less glass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlassAmount {
  Much,
  Glass,
  Less,
}

impl GlassAmount {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Much => "much",
      Self::Glass => "glass",
      Self::Less => "less",
    }
  }

  pub fn from_str(raw: &str) -> Self {
    match raw {
      "much" => Self::Much,
      "less" => Self::Less,
      _ => Self::Glass,
    }
  }
}

/// Effective customization from `customize_get`. Unknown values fall back
/// to the daemon defaults (multicolor accent, dark theme, glass amount).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Customize {
  pub wallpaper: String,
  pub accent: Accent,
  pub theme: ThemeMode,
  pub glass: GlassAmount,
  pub revision: u64,
}

impl Default for Customize {
  fn default() -> Self {
    Self {
      wallpaper: "THAOELAKE".to_string(),
      accent: Accent::Multicolor,
      theme: ThemeMode::Dark,
      glass: GlassAmount::Glass,
      revision: 0,
    }
  }
}

// ---------------------------------------------------------------------------
// Parsing from daemon JSON replies
// ---------------------------------------------------------------------------

fn get_str(value: &serde_json::Value, key: &str) -> Option<String> {
  value.get(key)?.as_str().map(|s| s.to_string())
}

fn get_u64(value: &serde_json::Value, key: &str) -> Option<u64> {
  value.get(key)?.as_u64()
}

fn get_f64(value: &serde_json::Value, key: &str) -> Option<f64> {
  value.get(key)?.as_f64()
}

fn get_bool(value: &serde_json::Value, key: &str) -> Option<bool> {
  value.get(key)?.as_bool()
}

fn section<'a>(result: &'a serde_json::Value, name: &str) -> serde_json::Value {
  result.get(name).cloned().unwrap_or(serde_json::Value::Null)
}

fn null_section() -> serde_json::Value {
  serde_json::Value::Null
}

impl Hardware {
  /// Parse a `get_hardware` result object (`processor`, `gpuN`, `ram`).
  /// Unknown GPUs are collected from `gpu0`..`gpuN` until the first gap.
  pub fn from_json(result: &serde_json::Value) -> Hardware {
    let processor = section(result, "processor");
    let ram_value = section(result, "ram");
    let mut gpus = Vec::new();
    for index in 0..64 {
      let gpu_value = section(result, &format!("gpu{}", index));
      if gpu_value.is_null() {
        break;
      }
      gpus.push(Gpu {
        name: get_str(&gpu_value, "name"),
        vendor_id: get_str(&gpu_value, "vendor_id"),
        device_id: get_str(&gpu_value, "device_id"),
        vram_mb: get_u64(&gpu_value, "vram_mb"),
        pci_slot: get_str(&gpu_value, "pci_slot"),
        driver: get_str(&gpu_value, "driver"),
        subsystem: get_str(&gpu_value, "subsystem"),
      });
    }
    let modules = ram_value
      .get("modules")
      .and_then(|v| v.as_array())
      .cloned()
      .unwrap_or_default()
      .iter()
      .map(|m| RamModule {
        locator: get_str(m, "locator"),
        size_mb: get_u64(m, "size_mb"),
        speed_mts: get_u64(m, "speed_mts"),
        manufacturer: get_str(m, "manufacturer"),
        part_number: get_str(m, "part_number"),
      })
      .collect();
    let mut modules_indexed: Vec<RamModule> = modules;
    // The daemon writes modules as ram.module0..N tables, accept both forms.
    if modules_indexed.is_empty() {
      for index in 0..64 {
        let key = format!("module{}", index);
        let module_value = ram_value.get(&key).cloned().unwrap_or(null_section());
        if module_value.is_null() {
          break;
        }
        modules_indexed.push(RamModule {
          locator: get_str(&module_value, "locator"),
          size_mb: get_u64(&module_value, "size_mb"),
          speed_mts: get_u64(&module_value, "speed_mts"),
          manufacturer: get_str(&module_value, "manufacturer"),
          part_number: get_str(&module_value, "part_number"),
        });
      }
    }
    Hardware {
      cpu: Cpu {
        name: get_str(&processor, "name"),
        vendor: get_str(&processor, "vendor"),
        cores: get_u64(&processor, "cores").map(|v| v as u32),
        threads: get_u64(&processor, "threads").map(|v| v as u32),
        mhz: get_u64(&processor, "mhz"),
      },
      gpus,
      ram: Ram {
        total_mb: get_u64(&ram_value, "total_mb").unwrap_or(0),
        total_gb: get_f64(&ram_value, "total_gb").unwrap_or(0.0),
        ram_type: ram_value
          .get("type")
          .and_then(|v| v.as_str())
          .map(RamType::from_str)
          .unwrap_or(RamType::Unknown),
        slots_used: get_u64(&ram_value, "slots_used").map(|v| v as u32),
        slots_total: get_u64(&ram_value, "slots_total").map(|v| v as u32),
        modules: modules_indexed,
      },
    }
  }
}

impl Os {
  /// Parse a `get_os` result object (`os` section). Missing fields fall back
  /// to the compiled defaults.
  pub fn from_json(result: &serde_json::Value) -> Os {
    let os_value = section(result, "os");
    let defaults = Os::default();
    Os {
      name: get_str(&os_value, "name").unwrap_or(defaults.name),
      display_name: get_str(&os_value, "display_name").unwrap_or(defaults.display_name),
      codename: get_str(&os_value, "codename").unwrap_or(defaults.codename),
      version: get_str(&os_value, "version").unwrap_or(defaults.version),
      beta: get_bool(&os_value, "beta").unwrap_or(defaults.beta),
    }
  }
}

impl Customize {
  /// Parse a `customize_get` result object (`wallpaper`, `accent`,
  /// `theme`, `glass`, `revision`). Unknown or missing values fall back to
  /// the daemon defaults, so a corrupt reply can never produce invalid
  /// state.
  pub fn from_json(result: &serde_json::Value) -> Customize {
    let defaults = Customize::default();
    Customize {
      wallpaper: get_str(result, "wallpaper").unwrap_or(defaults.wallpaper),
      accent: result
        .get("accent")
        .and_then(|v| v.as_str())
        .map(Accent::from_str)
        .unwrap_or(defaults.accent),
      theme: result
        .get("theme")
        .and_then(|v| v.as_str())
        .map(ThemeMode::from_str)
        .unwrap_or(defaults.theme),
      glass: result
        .get("glass")
        .and_then(|v| v.as_str())
        .map(GlassAmount::from_str)
        .unwrap_or(defaults.glass),
      revision: result.get("revision").and_then(|v| v.as_u64()).unwrap_or(0),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn hardware_fixture() -> serde_json::Value {
    serde_json::json!({
      "processor": {"name": "Test CPU", "vendor": "Intel", "cores": 8, "threads": 16, "mhz": 3700},
      "gpu0": {"name": "Test GPU", "vendor_id": "0x1002", "vram_mb": 8192, "driver": "amdgpu"},
      "ram": {"total_mb": 32768, "total_gb": 32.0, "type": "DDR5", "slots_used": 2,
              "module0": {"locator": "DIMM 0", "size_mb": 16384, "speed_mts": 4800}}
    })
  }

  #[test]
  fn parse_hardware_fixture() {
    let hardware = Hardware::from_json(&hardware_fixture());
    assert_eq!(hardware.cpu.name.as_deref(), Some("Test CPU"));
    assert_eq!(hardware.cpu.cores, Some(8));
    assert_eq!(hardware.gpus.len(), 1);
    assert_eq!(hardware.gpus[0].vram_mb, Some(8192));
    assert_eq!(hardware.ram.ram_type, RamType::Ddr5);
    assert_eq!(hardware.ram.modules.len(), 1);
    assert_eq!(hardware.ram.modules[0].speed_mts, Some(4800));
  }

  #[test]
  fn without_details_strips() {
    let hardware = Hardware::from_json(&hardware_fixture()).without_details();
    assert_eq!(hardware.cpu.name.as_deref(), Some("Test CPU"));
    assert_eq!(hardware.gpus[0].name, None);
    assert_eq!(hardware.gpus[0].driver, None);
    assert_eq!(hardware.gpus[0].vram_mb, Some(8192));
    assert_eq!(hardware.ram.ram_type, RamType::Unknown);
    assert!(hardware.ram.modules.is_empty());
    assert_eq!(hardware.ram.slots_used, None);
    assert_eq!(hardware.ram.total_gb, 32.0);
  }

  #[test]
  fn parse_os_fixture_with_defaults() {
    let os = Os::from_json(&serde_json::json!({"os": {"version": "26.2.0", "beta": true}}));
    assert_eq!(os.version, "26.2.0");
    assert!(os.beta);
    assert_eq!(os.display_name, "TontooOS Seal");
    let empty = Os::from_json(&serde_json::json!({}));
    assert_eq!(empty, Os::default());
  }

  #[test]
  fn ram_type_mapping() {
    assert_eq!(RamType::from_str("DDR4"), RamType::Ddr4);
    assert_eq!(RamType::from_str("ddr5"), RamType::Ddr5);
    assert_eq!(RamType::from_str("LPDDR5"), RamType::Unknown);
  }

  #[test]
  fn parse_customize_fixture() {
    let customize = Customize::from_json(&serde_json::json!({
      "wallpaper": "SONOMA",
      "accent": "blue",
      "theme": "light",
      "glass": "less",
      "revision": 7,
    }));
    assert_eq!(customize.wallpaper, "SONOMA");
    assert_eq!(customize.accent, Accent::Blue);
    assert_eq!(customize.accent.as_str(), "blue");
    assert_eq!(customize.accent.hex(), "#007AFF");
    assert_eq!(customize.theme, ThemeMode::Light);
    assert_eq!(customize.glass, GlassAmount::Less);
    assert_eq!(customize.revision, 7);
  }

  #[test]
  fn customize_unknown_values_fall_back_to_defaults() {
    let customize = Customize::from_json(&serde_json::json!({
      "accent": "neon",
      "theme": "sepia",
    }));
    assert_eq!(customize, Customize::default());
    assert_eq!(Accent::from_str("neon"), Accent::Multicolor);
    assert_eq!(ThemeMode::from_str("sepia"), ThemeMode::Dark);
  }
}
