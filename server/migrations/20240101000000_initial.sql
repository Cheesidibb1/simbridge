-- Initial database schema for SimBridge

-- Devices table
CREATE TABLE IF NOT EXISTS devices (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    device_type TEXT NOT NULL,
    platform TEXT NOT NULL,
    os_version TEXT NOT NULL,
    paired_at TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    is_trusted INTEGER NOT NULL DEFAULT 0,
    public_key TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    simulator_id TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    connected_at TEXT,
    disconnected_at TEXT,
    last_activity TEXT NOT NULL,
    FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE
);

-- Recordings table
CREATE TABLE IF NOT EXISTS recordings (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    duration_seconds INTEGER NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    metadata TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_sessions_device_id ON sessions(device_id);
CREATE INDEX IF NOT EXISTS idx_sessions_simulator_id ON sessions(simulator_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_recordings_session_id ON recordings(session_id);
