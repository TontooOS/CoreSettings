use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{ProviderError, Result};
use crate::types::{Customize, Hardware, Os};
use foundation::serialization::{JsonDocument, JsonObject, JsonValue};

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
    result
      .bool_field("pong")
      .map(|v| v.unwrap_or(false))
      .map_err(|e| ProviderError::Protocol(e.to_string()))
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

  /// Read effective customization (`customize_get` via the daemon):
  /// wallpaper, accent, theme plus the revision counter for change
  /// polling.
  pub fn customize(&self) -> Result<Customize> {
    let result = self.request("customize_get")?;
    Ok(Customize::from_json(&result))
  }

  /// Subscribe to daemon change events (`subscribe` op). The returned
  /// handle holds one persistent connection; the daemon pushes
  /// `{"event", "result"}` frames whenever a write op matching `events`
  /// succeeds. An empty list receives every event.
  pub fn subscribe(&self, events: &[&str]) -> Result<Subscription> {
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

    let items: Vec<String> = events
      .iter()
      .map(|name| JsonValue::Str(name.to_string()).stringify(false))
      .collect();
    let mut request = JsonObject::new();
    request
      .field_f64("id", 1.0)
      .map_err(|e| ProviderError::Protocol(e.to_string()))?;
    request.field_str("op", "subscribe");
    request
      .field_raw("params", &format!("{{\"events\":[{}]}}", items.join(",")))
      .map_err(|e| ProviderError::Protocol(e.to_string()))?;
    let line = request
      .build(false)
      .map_err(|e| ProviderError::Protocol(e.to_string()))?
      + "\n";
    writer
      .write_all(line.as_bytes())
      .and_then(|_| writer.flush())
      .map_err(|e| ProviderError::Connection(e.to_string()))?;

    let mut reply = String::new();
    reader
      .read_line(&mut reply)
      .map_err(|e| ProviderError::Connection(e.to_string()))?;
    let frame =
      JsonDocument::parse(&reply).map_err(|e| ProviderError::Protocol(e.to_string()))?;
    let ok = frame
      .bool_field("ok")
      .map_err(|e| ProviderError::Protocol(e.to_string()))?
      .unwrap_or(false);
    if !ok {
      return Err(ProviderError::Server(
        frame
          .str_field("error")
          .map_err(|e| ProviderError::Protocol(e.to_string()))?
          .unwrap_or_else(|| "unknown error".to_string()),
      ));
    }
    Ok(Subscription { reader })
  }

  fn request(&self, op: &str) -> Result<JsonDocument> {
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

    let mut request = JsonObject::new();
    request
      .field_f64("id", 1.0)
      .map_err(|e| ProviderError::Protocol(e.to_string()))?;
    request.field_str("op", op);
    let line = request
      .build(false)
      .map_err(|e| ProviderError::Protocol(e.to_string()))?
      + "\n";
    writer
      .write_all(line.as_bytes())
      .and_then(|_| writer.flush())
      .map_err(|e| ProviderError::Connection(e.to_string()))?;

    let mut reply = String::new();
    reader
      .read_line(&mut reply)
      .map_err(|e| ProviderError::Connection(e.to_string()))?;
    let frame =
      JsonDocument::parse(&reply).map_err(|e| ProviderError::Protocol(e.to_string()))?;
    let ok = frame
      .bool_field("ok")
      .map_err(|e| ProviderError::Protocol(e.to_string()))?
      .unwrap_or(false);
    if ok {
      Ok(
        frame
          .nested("result")
          .map_err(|e| ProviderError::Protocol(e.to_string()))?
          .unwrap_or_else(JsonDocument::empty),
      )
    } else {
      Err(ProviderError::Server(
        frame
          .str_field("error")
          .map_err(|e| ProviderError::Protocol(e.to_string()))?
          .unwrap_or_else(|| "unknown error".to_string()),
      ))
    }
  }
}

/// One pushed daemon change: the event name (e.g. `customize_changed`)
/// plus the write-op result payload.
#[derive(Debug, Clone)]
pub struct DaemonEvent {
  pub name: String,
  pub payload: JsonDocument,
}

/// Persistent change-event subscription. Holds one connection open; the
/// daemon pushes a frame per matching write op. No polling, no traffic
/// while nothing changes.
#[derive(Debug)]
pub struct Subscription {
  reader: BufReader<UnixStream>,
}

impl Subscription {
  /// Non-blocking event check. Returns `Ok(None)` when no event arrived
  /// yet. Returns `Err` when the daemon closed the connection or sent a
  /// malformed frame.
  pub fn try_next(&mut self) -> Result<Option<DaemonEvent>> {
    // Zero timeouts are rejected, so use 1 ms: no visible block, but the
    // kernel still reports "no data" instead of erroring on the timeout.
    self
      .reader
      .get_ref()
      .set_read_timeout(Some(Duration::from_millis(1)))
      .map_err(|e| ProviderError::Connection(e.to_string()))?;
    let mut line = String::new();
    match self.reader.read_line(&mut line) {
      Err(e)
        if e.kind() == std::io::ErrorKind::WouldBlock
          || e.kind() == std::io::ErrorKind::TimedOut =>
      {
        return Ok(None);
      }
      Err(e) => return Err(ProviderError::Connection(e.to_string())),
      Ok(0) => {
        return Err(ProviderError::Connection(
          "daemon closed the subscription".to_string(),
        ));
      }
      Ok(_) => {}
    }
    let frame =
      JsonDocument::parse(&line).map_err(|e| ProviderError::Protocol(e.to_string()))?;
    let name = frame
      .str_field("event")
      .map_err(|e| ProviderError::Protocol(e.to_string()))?
      .ok_or_else(|| ProviderError::Protocol("event frame without event name".to_string()))?;
    let payload = frame
      .nested("result")
      .map_err(|e| ProviderError::Protocol(e.to_string()))?
      .unwrap_or_else(JsonDocument::empty);
    Ok(Some(DaemonEvent { name, payload }))
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

  /// Mock daemon: accepts one connection, reads one request line, then
  /// writes each canned reply line. Returns the socket path.
  fn mock_daemon(replies: Vec<String>, hold_open_ms: u64) -> PathBuf {
    use std::os::unix::net::UnixListener;

    let path = std::env::temp_dir().join(format!(
      "tontoo-coresettings-mock-{}-{}.sock",
      std::process::id(),
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).unwrap();
    std::thread::spawn(move || {
      if let Ok((mut stream, _)) = listener.accept() {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        let _ = reader.read_line(&mut line);
        for reply in &replies {
          if stream.write_all(reply.as_bytes()).is_err() {
            return;
          }
        }
        let _ = stream.flush();
        std::thread::sleep(std::time::Duration::from_millis(hold_open_ms));
      }
    });
    path
  }

  #[test]
  fn subscribe_receives_pushed_event() {
    let sock = mock_daemon(
      vec![
        "{\"id\":1,\"ok\":true,\"result\":{\"subscribed\":true,\"events\":[]}}\n".to_string(),
        "{\"event\":\"customize_changed\",\"result\":{\"theme\":\"light\",\"revision\":3}}\n".to_string(),
      ],
      500,
    );
    let provider = SettingsProvider::with_socket(&sock);
    let mut subscription = provider.subscribe(&["customize_changed"]).unwrap();
    // The push may need a moment to arrive; retry briefly.
    let mut event = None;
    for _ in 0..50 {
      if let Some(received) = subscription.try_next().unwrap() {
        event = Some(received);
        break;
      }
      std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let event = event.expect("expected a pushed event");
    assert_eq!(event.name, "customize_changed");
    let customize = Customize::from_json(&event.payload);
    assert_eq!(customize.theme, crate::types::ThemeMode::Light);
    assert_eq!(customize.revision, 3);
    let _ = std::fs::remove_file(&sock);
  }

  #[test]
  fn try_next_without_event_returns_none() {
    let sock = mock_daemon(
      vec!["{\"id\":1,\"ok\":true,\"result\":{\"subscribed\":true,\"events\":[]}}\n".to_string()],
      500,
    );
    let provider = SettingsProvider::with_socket(&sock);
    let mut subscription = provider.subscribe(&[]).unwrap();
    assert!(subscription.try_next().unwrap().is_none());
    let _ = std::fs::remove_file(&sock);
  }

  #[test]
  fn subscribe_refused_is_an_error() {
    let sock = mock_daemon(
      vec!["{\"id\":1,\"ok\":false,\"error\":\"unknown op\"}\n".to_string()],
      200,
    );
    let provider = SettingsProvider::with_socket(&sock);
    let err = provider.subscribe(&[]).unwrap_err();
    assert!(matches!(err, ProviderError::Server(_)));
    assert!(err.to_string().contains("unknown op"));
    let _ = std::fs::remove_file(&sock);
  }
}
