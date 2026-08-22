// SimBridge Server library

pub mod core;
pub mod networking {
    pub mod rest;
    pub mod websocket;
}
pub mod adapters;
pub mod metrics;
pub mod recording;
pub mod storage;
pub mod streaming;

pub use adapters::{android::AndroidEmulatorAdapter, ios::IosSimulatorAdapter};
pub use core::{
    auth::AuthManager,
    plugin::{PluginContext, PluginManager},
    session::SessionManager,
};
pub use networking::rest::{create_router, RestServerState};
pub use networking::websocket::{websocket_handler, WebSocketServerState};
pub use storage::database::Database;
