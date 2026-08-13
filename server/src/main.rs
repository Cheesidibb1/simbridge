// SimBridge Server main entry point

use clap::Parser;
use simbridge_shared::logging;
use tracing::info;
use std::sync::Arc;

use simbridge_server::{
    networking::rest::{RestServerState, create_router},
    networking::websocket::{WebSocketServerState, websocket_handler},
    storage::database::Database,
    core::session::SessionManager,
    core::auth::AuthManager,
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
    let _session_manager = Arc::new(SessionManager::new(10));
    let _auth_manager = Arc::new(AuthManager::new(5, 300));

    // Initialize REST server state
    let rest_state = RestServerState::new();

    // Initialize WebSocket server state
    let ws_state = WebSocketServerState::new();

    // Create Axum router with WebSocket support
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
