# Tontoo CoreSettings

Client library for TontooOS system facts. It reads hardware (`sys.fico`)
and OS identity (`os.fico`) from the settings daemon over its unix socket.
All data comes from the daemon, a missing daemon surfaces as an error.

## Made for TontooOS

Explore more at https://github.com/TontooOS/Libs

## Adding to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
sdk = { path = "/Library/System/sdk", features = ["CoreSettings"] }
```

## License

TCL v26.1