//! The package version. `Cargo.toml` is the only place it is written.

/// The version `Cargo.toml` declares, fixed at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
