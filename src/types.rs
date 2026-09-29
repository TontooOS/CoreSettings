use foundation::serialization::JsonDocument;

// ---------------------------------------------------------------------------
// RAM type (DDR4/DDR5 only, like the daemon backend)
// ---------------------------------------------------------------------------

/// Classified RAM type. Only DDR4 and DDR5 are distinguished.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Default)]
pub struct Cpu {
  pub name: Option<String>,
  pub vendor: Option<String>,
  pub cores: Option<u32>,
  pub threads: Option<u32>,
  pub mhz: Option<u64>,
}

/// Graphics device facts. `vram_mb` is `None` for shared-memory GPUs.
#[derive(Debug, Clone, Default)]
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
#[derive(Debug, Clone, Default)]
pub struct RamModule {
  pub locator: Option<String>,
  pub size_mb: Option<u64>,
  pub speed_mts: Option<u64>,
  pub manufacturer: Option<String>,
  pub part_number: Option<String>,
}

/// Memory facts.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Default)]
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
#[derive(Debug, Clone, PartialEq)]
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
      version: "27.0.0".to_string(),
      beta: false,
    }
  }
}

// ---------------------------------------------------------------------------
// Customization (mirrors the daemon `customize` domain)
// ---------------------------------------------------------------------------

/// Color theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
// Parsing from daemon JSON replies (via Foundation's std-only JsonDocument)
// ---------------------------------------------------------------------------

fn section(result: &JsonDocument, name: &str) -> JsonDocument {
  result
    .nested(name)
    .unwrap_or(None)
    .unwrap_or_else(JsonDocument::empty)
}

fn parse_module(value: &JsonDocument) -> RamModule {
  RamModule {
    locator: value.str_field("locator").unwrap_or(None),
    size_mb: value.u64_field("size_mb").unwrap_or(None),
    speed_mts: value.u64_field("speed_mts").unwrap_or(None),
    manufacturer: value.str_field("manufacturer").unwrap_or(None),
    part_number: value.str_field("part_number").unwrap_or(None),
  }
}

impl Hardware {
  /// Parse a `get_hardware` result object (`processor`, `gpuN`, `ram`).
  /// Unknown GPUs are collected from `gpu0`..`gpuN` until the first gap.
  pub fn from_json(result: &JsonDocument) -> Hardware {
    let processor = section(result, "processor");
    let ram_value = section(result, "ram");
    let mut gpus = Vec::new();
    for index in 0..64 {
      let key = format!("gpu{}", index);
      let Some(gpu_value) = result.nested(&key).unwrap_or(None) else {
        break;
      };
      gpus.push(Gpu {
        name: gpu_value.str_field("name").unwrap_or(None),
        vendor_id: gpu_value.str_field("vendor_id").unwrap_or(None),
        device_id: gpu_value.str_field("device_id").unwrap_or(None),
        vram_mb: gpu_value.u64_field("vram_mb").unwrap_or(None),
        pci_slot: gpu_value.str_field("pci_slot").unwrap_or(None),
        driver: gpu_value.str_field("driver").unwrap_or(None),
        subsystem: gpu_value.str_field("subsystem").unwrap_or(None),
      });
    }
    let mut modules_indexed: Vec<RamModule> = ram_value
      .array_field("modules")
      .unwrap_or_default()
      .iter()
      .map(parse_module)
      .collect();
    // The daemon writes modules as ram.module0..N tables, accept both forms.
    if modules_indexed.is_empty() {
      for index in 0..64 {
        let key = format!("module{}", index);
        let Some(module_value) = ram_value.nested(&key).unwrap_or(None) else {
          break;
        };
        modules_indexed.push(parse_module(&module_value));
      }
    }
    Hardware {
      cpu: Cpu {
        name: processor.str_field("name").unwrap_or(None),
        vendor: processor.str_field("vendor").unwrap_or(None),
        cores: processor.u64_field("cores").unwrap_or(None).map(|v| v as u32),
        threads: processor.u64_field("threads").unwrap_or(None).map(|v| v as u32),
        mhz: processor.u64_field("mhz").unwrap_or(None),
      },
      gpus,
      ram: Ram {
        total_mb: ram_value.u64_field("total_mb").unwrap_or(None).unwrap_or(0),
        total_gb: ram_value.f64_field("total_gb").unwrap_or(None).unwrap_or(0.0),
        ram_type: ram_value
          .str_field("type")
          .unwrap_or(None)
          .map(|v| RamType::from_str(&v))
          .unwrap_or(RamType::Unknown),
        slots_used: ram_value.u64_field("slots_used").unwrap_or(None).map(|v| v as u32),
        slots_total: ram_value.u64_field("slots_total").unwrap_or(None).map(|v| v as u32),
        modules: modules_indexed,
      },
    }
  }
}

impl Os {
  /// Parse a `get_os` result object (`os` section). Missing fields fall back
  /// to the compiled defaults.
  pub fn from_json(result: &JsonDocument) -> Os {
    let os_value = section(result, "os");
    let defaults = Os::default();
    Os {
      name: os_value.str_field("name").unwrap_or(None).unwrap_or(defaults.name),
      display_name: os_value.str_field("display_name").unwrap_or(None).unwrap_or(defaults.display_name),
      codename: os_value.str_field("codename").unwrap_or(None).unwrap_or(defaults.codename),
      version: os_value.str_field("version").unwrap_or(None).unwrap_or(defaults.version),
      beta: os_value.bool_field("beta").unwrap_or(None).unwrap_or(defaults.beta),
    }
  }
}

impl Customize {
  /// Parse a `customize_get` result object (`wallpaper`, `accent`,
  /// `theme`, `glass`, `revision`). Unknown or missing values fall back to
  /// the daemon defaults, so a corrupt reply can never produce invalid
  /// state.
  pub fn from_json(result: &JsonDocument) -> Customize {
    let defaults = Customize::default();
    Customize {
      wallpaper: result.str_field("wallpaper").unwrap_or(None).unwrap_or(defaults.wallpaper),
      accent: result
        .str_field("accent")
        .unwrap_or(None)
        .map(|v| Accent::from_str(&v))
        .unwrap_or(defaults.accent),
      theme: result
        .str_field("theme")
        .unwrap_or(None)
        .map(|v| ThemeMode::from_str(&v))
        .unwrap_or(defaults.theme),
      glass: result
        .str_field("glass")
        .unwrap_or(None)
        .map(|v| GlassAmount::from_str(&v))
        .unwrap_or(defaults.glass),
      revision: result.u64_field("revision").unwrap_or(None).unwrap_or(0),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn hardware_fixture() -> JsonDocument {
    JsonDocument::parse(
      r#"{"processor": {"name": "Test CPU", "vendor": "Intel", "cores": 8, "threads": 16, "mhz": 3700},
          "gpu0": {"name": "Test GPU", "vendor_id": "0x1002", "vram_mb": 8192, "driver": "amdgpu"},
          "ram": {"total_mb": 32768, "total_gb": 32.0, "type": "DDR5", "slots_used": 2,
                  "module0": {"locator": "DIMM 0", "size_mb": 16384, "speed_mts": 4800}}}"#,
    )
    .unwrap()
  }

  fn doc(raw: &str) -> JsonDocument {
    JsonDocument::parse(raw).unwrap()
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
    let os = Os::from_json(&doc(r#"{"os": {"version": "26.2.0", "beta": true}}"#));
    assert_eq!(os.version, "26.2.0");
    assert!(os.beta);
    assert_eq!(os.display_name, "TontooOS Seal");
    let empty = Os::from_json(&doc(r#"{}"#));
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
    let customize = Customize::from_json(&doc(
      r#"{"wallpaper": "SONOMA", "accent": "blue", "theme": "light", "glass": "less", "revision": 7}"#,
    ));
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
    let customize = Customize::from_json(&doc(r#"{"accent": "neon", "theme": "sepia"}"#));
    assert_eq!(customize, Customize::default());
    assert_eq!(Accent::from_str("neon"), Accent::Multicolor);
    assert_eq!(ThemeMode::from_str("sepia"), ThemeMode::Dark);
  }
}
