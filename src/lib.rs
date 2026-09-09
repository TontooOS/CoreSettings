pub mod error;
pub mod lang;
pub mod provider;
pub mod types;

pub use error::{ProviderError, Result};
pub use provider::{SettingsProvider, DEFAULT_SOCKET_PATH};
pub use types::{Cpu, Gpu, Hardware, Os, Ram, RamModule, RamType};

/// Ping the daemon at the default socket.
pub fn ping() -> Result<bool> {
  SettingsProvider::new().ping()
}

/// Read hardware facts at the default socket.
/// See [`SettingsProvider::hardware`] for the `detailed` flag.
pub fn hardware(detailed: bool) -> Result<Hardware> {
  SettingsProvider::new().hardware(detailed)
}

/// Read OS identity facts at the default socket.
pub fn os() -> Result<Os> {
  SettingsProvider::new().os()
}
