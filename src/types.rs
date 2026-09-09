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
}
