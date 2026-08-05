// SimBridge Server main entry point

use clap::Parser;
use simbridge_shared::logging;
use tracing::{info, error};
use std::sync::Arc;
use tokio::sync::RwLock;

use simbridge_server::{
    core::{session::SessionManager, auth::AuthManager, plugin::PluginManager, PluginContext},
    networking::{websocket::WebSocketServerState, rest::{RestServerState, create_router}},
    adapters::{ios::IosSimulatorAdapter, android::AndroidEmulatorAdapter, discovery::DeviceDiscovery},
    storage::database::Database,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server host address
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Server port
    #[arg(long, default_value_t = 8080)]
    port: u16,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Database path
    #[arg(long, default_value = "simbridge.db")]
    database: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    logging::init_logging(&args.log_level);
    info!("Starting SimBridge Server v{}", env!("CARGO_PKG_VERSION"));

    // Initialize database
    let database = Database::new(std::path::Path::new(&args.database)).await?;
    database.migrate().await?;
    info!("Database initialized at {}", args.database);

    // Initialize core managers
    let session_manager = Arc::new(SessionManager::new(10));
    let auth_manager = Arc::new(AuthManager::new(5, 300));
    
    // Initialize plugin manager
    let plugin_context = PluginContext {
        config_dir: std::path::PathBuf::from(".config"),
        data_dir: std::path::PathBuf::from(".data"),
        server_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let plugin_manager = Arc::new(RwLock::new(PluginManager::new(plugin_context)));

    // Initialize device discovery
    let device_discovery = Arc::new(DeviceDiscovery::new());
    
    // Discover devices on startup
    info!("Discovering Android devices...");
    match device_discovery.discover_android().await {
        Ok(devices) => info!("Found {} Android device(s)", devices.len()),
        Err(e) => error!("Failed to discover Android devices: {}", e),
    }
    
    info!("Discovering iOS devices...");
    match device_discovery.discover_ios().await {
        Ok(devices) => info!("Found {} iOS device(s)", devices.len()),
        Err(e) => error!("Failed to discover iOS devices: {}", e),
    }

    // Initialize WebSocket server state
    let ws_state = WebSocketServerState::new();
    
    // Initialize REST server state with discovered devices
    let android_devices = device_discovery.get_android_adapters().await;
    let ios_devices = device_discovery.get_ios_adapters().await;
    
    let android_device_ids: Vec<String> = android_devices.iter().map(|a| a.device_id().to_string()).collect();
    let ios_device_ids: Vec<String> = ios_devices.iter().map(|a| a.device_id().to_string()).collect();
    
    let rest_state = RestServerState {
        sessions: Arc::new(RwLock::new(Vec::new())),
        android_adapters: Arc::new(RwLock::new(android_device_ids)),
        ios_adapters: Arc::new(RwLock::new(ios_device_ids)),
    };

    // TODO: Initialize simulator adapters
    let _ios_adapter = IosSimulatorAdapter::new(
        "ios-sim-1".to_string(),
        "iPhone 15 Pro".to_string()
    );
    
    let _android_adapter = AndroidEmulatorAdapter::new(
        "android-emu-1".to_string(),
        "Pixel 7".to_string()
    );

    // Create Axum router
    let app = create_router()
        .route("/ws", axum::routing::get({
            let ws_state = ws_state.clone();
            move |ws| websocket_handler(ws, ws_state)
        }))
        .with_state(rest_state);

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", args.host, args.port))
        .await
        .expect("Failed to bind to address");

    info!("Server listening on {}:{}", args.host, args.port);

    axum::serve(listener, app)
        .await
        .expect("Server error");

    Ok(())
}

/// WebSocket handler (re-exported from networking module)
async fn websocket_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    state: WebSocketServerState,
) -> impl axum::response::IntoResponse {
    simbridge_server::networking::websocket::websocket_handler(ws, state).await
}
