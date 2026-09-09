use coresettings::SettingsProvider;

fn main() {
  let socket = std::env::args().nth(1);
  let provider = match socket {
    Some(path) => SettingsProvider::with_socket(path),
    None => SettingsProvider::from_env(),
  };

  match provider.os() {
    Ok(os) => println!(
      "{} {} (codename {}, beta={})",
      os.display_name, os.version, os.codename, os.beta
    ),
    Err(e) => eprintln!("os: {}", e),
  }

  match provider.hardware(true) {
    Ok(hardware) => {
      if let Some(name) = &hardware.cpu.name {
        println!(
          "cpu: {} ({} cores, {} threads)",
          name,
          hardware.cpu.cores.unwrap_or(0),
          hardware.cpu.threads.unwrap_or(0)
        );
      }
      for (index, gpu) in hardware.gpus.iter().enumerate() {
        println!(
          "gpu{}: {} (vram {} MB)",
          index,
          gpu.name.as_deref().unwrap_or("unknown"),
          gpu.vram_mb.map(|v| v.to_string()).unwrap_or_else(|| "?".to_string())
        );
      }
      println!(
        "ram: {} GB ({})",
        hardware.ram.total_gb,
        hardware.ram.ram_type.as_str()
      );
    }
    Err(e) => eprintln!("hardware: {}", e),
  }
}
