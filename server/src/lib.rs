// SimBridge Server library

pub mod core;
pub mod networking;
pub mod adapters;
pub mod streaming;
pub mod storage;
pub mod recording;
pub mod metrics;

pub use core::{session::SessionManager, auth::AuthManager, plugin::{PluginManager, PluginContext}};
pub use networking::{websocket::WebSocketServerState, rest::{RestServerState, create_router}};
pub use adapters::{ios::IosSimulatorAdapter, android::AndroidEmulatorAdapter};
pub use storage::database::Database;
