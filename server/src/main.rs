// SimBridge Server main entry point

use axum::{
    http::{header, Method},
    response::Html,
    routing::get,
};
use clap::Parser;
use simbridge_shared::logging;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
use tracing::{info, warn};

use simbridge_server::{
    adapters::{discovery::DeviceDiscovery, interface::SimulatorAdapter},
    core::auth::AuthManager,
    core::session::SessionManager,
    networking::rest::{create_router, RestServerState},
    networking::websocket::{websocket_handler, WebSocketServerState},
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

    /// Directory containing the built Flutter web app (`flutter build web`).
    /// When set, or when `companion/build/web` is found automatically, the
    /// server serves the app itself so an iPhone can just open
    /// http://<this-machine>:<port>/ in Safari. Can also be set with the
    /// SIMBRIDGE_WEB_DIR environment variable.
    #[arg(long)]
    web_dir: Option<String>,

    /// Don't send permissive CORS headers. They are on by default so the web
    /// app also works when it is hosted on a different origin or port
    /// (e.g. `flutter run -d web-server`); same-origin use doesn't need them.
    #[arg(long, default_value_t = false)]
    no_cors: bool,
}

/// Finds the built web app: an explicit `--web-dir` wins (and is validated),
/// otherwise a few conventional locations relative to the working directory.
fn resolve_web_dir(explicit: &Option<String>) -> Option<PathBuf> {
    let from_env = std::env::var("SIMBRIDGE_WEB_DIR").ok().filter(|v| !v.is_empty());
    if let Some(dir) = explicit.as_ref().or(from_env.as_ref()) {
        let path = PathBuf::from(dir);
        if path.join("index.html").is_file() {
            return Some(path);
        }
        warn!(
            "--web-dir {} has no index.html; run `flutter build web` first. Falling back to auto-detection",
            dir
        );
    }

    ["companion/build/web", "../companion/build/web", "web"]
        .iter()
        .map(PathBuf::from)
        .find(|path| path.join("index.html").is_file())
}

/// Best-effort LAN address of this machine, for the "open this on your iPhone"
/// hint. Connecting a UDP socket sends no packets; it only makes the OS pick
/// the outbound interface, whose address we then read back.
fn lan_ip() -> Option<std::net::IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("192.0.2.1:9").ok()?; // TEST-NET-1: never routed
    socket.local_addr().ok().map(|addr| addr.ip())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    logging::init_logging(&args.log_level);
    info!("Starting SimBridge Server v{}", env!("CARGO_PKG_VERSION"));

    let server_password = match std::env::var("SIMBRIDGE_PASSWORD") {
        Ok(password) if password.chars().count() >= 12 => {
            info!("Using password configured by SIMBRIDGE_PASSWORD");
            password
        }
        Ok(_) => anyhow::bail!("SIMBRIDGE_PASSWORD must contain at least 12 characters"),
        Err(_) => {
            let password = uuid::Uuid::new_v4().simple().to_string();
            println!(
                "SIMBRIDGE_PASSWORD is unset; generated password: {}",
                password
            );
            password
        }
    };

    // Initialize database
    let database = Database::new(std::path::Path::new(&args.database)).await?;
    database.migrate().await?;
    info!("Database initialized at {}", args.database);

    // Initialize core managers
    let _session_manager = Arc::new(SessionManager::new(10));
    let _auth_manager = Arc::new(AuthManager::new(5, 300));

    // Initialize REST server state
    let rest_state = RestServerState::new();

    // Populate Android devices for the simulator list. ADB may not be
    // available until the Android SDK is configured, so discovery is best effort.
    match DeviceDiscovery::new().discover_android().await {
        Ok(adapters) => {
            let mut android_devices = rest_state.android_adapters.write().await;
            *android_devices = adapters
                .iter()
                .map(|adapter| adapter.simulator_id().to_string())
                .collect();
            info!(count = android_devices.len(), "Discovered Android devices");
        }
        Err(error) => warn!(%error, "Android device discovery unavailable"),
    }

    // Initialize WebSocket server state
    let ws_state = WebSocketServerState::new(server_password);

    // Create Axum router with WebSocket support
    let mut app = create_router()
        .route(
            "/ws",
            axum::routing::get({
                let ws_state = ws_state.clone();
                move |ws| websocket_handler(ws, ws_state)
            }),
        )
        // Browser WebRTC test client; reachable from an iPhone at /test-webrtc.
        .route(
            "/test-webrtc",
            get(|| async { Html(include_str!("../test-webrtc.html")) }),
        )
        .with_state(rest_state);

    // Serve the Flutter web build from this same origin. Same-origin matters
    // for Safari: no CORS preflights, and no mixed-content blocking between
    // the page and its REST/WebSocket calls.
    let web_dir = resolve_web_dir(&args.web_dir);
    match &web_dir {
        Some(dir) => {
            info!("Serving web app from {}", dir.display());
            let index = ServeFile::new(dir.join("index.html"));
            app = app.fallback_service(ServeDir::new(dir).fallback(index));
        }
        None => {
            warn!("No web app build found; serving a setup page at / (run scripts/build-web.sh)");
            app = app.route(
                "/",
                get(|| async { Html(include_str!("../static/landing.html")) }),
            );
        }
    }

    if !args.no_cors {
        app = app.layer(
            // Explicit lists rather than `*`: older Safari releases don't honour
            // the wildcard form of Allow-Methods / Allow-Headers in preflights.
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([header::CONTENT_TYPE, header::ACCEPT, header::AUTHORIZATION]),
        );
    }

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", args.host, args.port))
        .await
        .expect("Failed to bind to address");

    info!("Server listening on {}:{}", args.host, args.port);
    if web_dir.is_some() {
        match lan_ip() {
            Some(ip) => info!("Open on your iPhone (Safari): http://{}:{}/", ip, args.port),
            None => info!("Open on your iPhone (Safari): http://<this-machine-ip>:{}/", args.port),
        }
    }

    axum::serve(listener, app).await.expect("Server error");

    Ok(())
}
