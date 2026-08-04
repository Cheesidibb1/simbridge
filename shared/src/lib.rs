// SimBridge Shared Core Library
// This library contains common protocol definitions, models, networking,
// authentication, and utilities used across the SimBridge platform.

pub mod protocol;
pub mod models;
pub mod networking;
pub mod auth;
pub mod utils;
pub mod logging;

pub use protocol::*;
pub use models::*;
pub use networking::*;
pub use auth::*;
pub use utils::*;
pub use logging::*;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PROTOCOL_VERSION: u32 = 1;
