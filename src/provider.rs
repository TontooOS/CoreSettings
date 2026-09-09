use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{ProviderError, Result};
use crate::types::{Hardware, Os};

/// Default daemon socket path (mirrors the daemon default).
pub const DEFAULT_SOCKET_PATH: &str = "/run/tontoo-settings.sock";

/// Socket read timeout per request.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Client handle for the settings daemon. All facts come from the daemon
/// over its unix socket, a missing daemon surfaces as an error.
#[derive(Debug, Clone)]
pub struct SettingsProvider {
  socket_path: PathBuf,
}

impl Default for SettingsProvider {
  fn default() -> Self {
    Self::new()
  }
}

impl SettingsProvider {
  pub fn new() -> Self {
    Self {
      socket_path: PathBuf::from(DEFAULT_SOCKET_PATH),
    }
  }

  pub fn with_socket(path: impl Into<PathBuf>) -> Self {
    Self {
      socket_path: path.into(),
    }
  }

  /// Build from `SETTINGS_SOCKET`, falls back to the default path.
  pub fn from_env() -> Self {
    let socket_path = std::env::var("SETTINGS_SOCKET")
      .ok()
      .map(PathBuf::from)
      .unwrap_or_else(|| PathBuf::from(DEFAULT_SOCKET_PATH));
    Self { socket_path }
  }

  pub fn socket_path(&self) -> &Path {
    &self.socket_path
  }

  /// Ping the daemon. Returns `true` on a valid pong reply.
  pub fn ping(&self) -> Result<bool> {
    let result = self.request("ping")?;
    Ok(result.get("pong").and_then(|v| v.as_bool()).unwrap_or(false))
  }

  /// Read hardware facts (`sys.fico` via the daemon).
  ///
  /// `detailed=false` returns the basis view (no GPU names/drivers, RAM
  /// type `Unknown`, no modules), `detailed=true` returns everything the
  /// daemon collected.
  pub fn hardware(&self, detailed: bool) -> Result<Hardware> {
    let result = self.request("get_hardware")?;
    let hardware = Hardware::from_json(&result);
    if detailed {
      Ok(hardware)
    } else {
      Ok(hardware.without_details())
    }
  }

  /// Read OS identity facts (`os.fico` via the daemon).
  pub fn os(&self) -> Result<Os> {
    let result = self.request("get_os")?;
    Ok(Os::from_json(&result))
  }

  fn request(&self, op: &str) -> Result<serde_json::Value> {
    if !self.socket_path.exists() {
      return Err(ProviderError::SocketMissing(
        self.socket_path.to_string_lossy().into_owned(),
      ));
    }
    let stream = UnixStream::connect(&self.socket_path)
      .map_err(|e| ProviderError::Connection(e.to_string()))?;
    stream
      .set_read_timeout(Some(REQUEST_TIMEOUT))
      .map_err(|e| ProviderError::Connection(e.to_string()))?;
    let mut writer = stream
      .try_clone()
      .map_err(|e| ProviderError::Connection(e.to_string()))?;
    let mut reader = BufReader::new(stream);

    let line = serde_json::json!({"id": 1, "op": op}).to_string() + "\n";
    writer
      .write_all(line.as_bytes())
      .and_then(|_| writer.flush())
      .map_err(|e| ProviderError::Connection(e.to_string()))?;

    let mut reply = String::new();
    reader
      .read_line(&mut reply)
      .map_err(|e| ProviderError::Connection(e.to_string()))?;
    let frame: serde_json::Value =
      serde_json::from_str(&reply).map_err(|e| ProviderError::Protocol(e.to_string()))?;
    if frame.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
      Ok(frame.get("result").cloned().unwrap_or(serde_json::Value::Null))
    } else {
      Err(ProviderError::Server(
        frame
          .get("error")
          .and_then(|v| v.as_str())
          .unwrap_or("unknown error")
          .to_string(),
      ))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn missing_socket_errors() {
    let provider = SettingsProvider::with_socket("/nonexistent/tontoo-settings-test.sock");
    let err = provider.ping().unwrap_err();
    assert!(matches!(err, ProviderError::SocketMissing(_)));
    assert!(err.to_string().contains("/nonexistent/tontoo-settings-test.sock"));
  }

  #[test]
  fn from_env_default() {
    std::env::remove_var("SETTINGS_SOCKET");
    assert_eq!(
      SettingsProvider::from_env().socket_path(),
      Path::new(DEFAULT_SOCKET_PATH)
    );
  }
}
