# SimBridge Quick Start Guide

## 5-Minute Setup (Development Environment)

This guide will get you running a local development environment in under 5 minutes.

### Prerequisites Checklist

- [ ] **Rust** - Install from [rustup.rs](https://rustup.rs/)
- [ ] **Cargo** - Comes with Rust (verify: `cargo --version`)
- [ ] **Git** - For version control
- [ ] **Flutter** - SDK 3.16+ (optional, for companion app)

### Step 1: Clone Repository

```bash
git clone https://github.com/yourusername/simbridge.git
cd simbridge
```

### Step 2: Build Shared Library

```bash
cd shared
cargo build --release
```

**Expected Output:**
```
    Finished release [optimized] target(s) in 0.15s
```

### Step 3: Verify Build

```bash
# Check library compiles
cargo test --lib

# Expected: All tests pass
test result: ok. XX passed; 0 failed
```

### Step 4: Run Server (Development Mode)

```bash
cd ../server
cargo run -- --port 8080 --log-level debug
```

**Expected Output:**
```
[INFO] Starting SimBridge Server v0.1.0
[INFO] Database initialized at simbridge.db
[INFO] Listening on 0.0.0.0:8080
```

### Step 5: Health Check

Open a new terminal and test the server:

```bash
curl http://localhost:8080/health
```

**Expected Response:**
```json
{"status":"healthy","version":"0.1.0"}
```

### Step 6: Verify Simulators Endpoint

```bash
curl http://localhost:8080/api/v1/simulators
```

**Expected Response (with no devices connected):**
```json
{
  "simulators": [
    {"id":"android-emu-1","name":"Pixel 7","platform":"android","status":"offline"},
    {"id":"ios-sim-1","name":"iPhone 15 Pro","platform":"ios","status":"offline"}
  ]
}
```

---

## Running Tests

### Rust Tests

```bash
cd shared
cargo test

# Run specific module
cargo test protocol

# Run with coverage
cargo tarpaulin --out Html
```

### Dart/Flutter Tests (if Flutter installed)

```bash
cd companion
flutter pub get
flutter test
```

---

## Build for Production

### Build Server

```bash
cd server
cargo build --release
# Binary location: ./target/release/simbridge-server
```

### Build Companion App

```bash
cd companion
flutter pub get
flutter build apk  # Android
# or flutter build ios  # iOS
```

---

## Troubleshooting

### Issue: `cargo` not found

**Solution:**
```bash
# Add Rust to PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Or install via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Issue: Flutter build fails

**Solution:**
```bash
# Update Flutter
flutter upgrade

# Get dependencies
flutter pub get

# Check for missing packages
flutter doctor
```

### Issue: Server won't start

**Possible Causes:**
1. Port 8080 already in use - change port with `--port 8081`
2. Database permission issues - ensure write access to current directory
3. Missing dependencies - run `cargo build --all-features`

---

## Next Steps After Quick Start

1. **Read the Full Documentation**
   - [Developer Guide](./DEVELOPER.md) - Detailed setup
   - [Architecture Guide](./ARCHITECTURE.md) - System design
   - [Adapter Implementation](./ADAPTER_IMPLEMENTATION.md) - Add platforms

2. **Run Tests**
   ```bash
   cargo test --workspace
   ```

3. **Explore the Codebase**
   - Start with `shared/src/` for core library
   - Look at `server/src/main.rs` for entry point
   - Check `companion/lib/main.dart` for client app

4. **Join Development**
   - See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines
   - Check [ROADMAP.md](./ROADMAP.md) for future features

---

## Quick Reference Commands

### Build
```bash
# Full workspace build
cargo build --workspace --release

# Debug build
cargo build --workspace
```

### Run
```bash
# Server (debug)
cd server && cargo run

# Server (production)
cd server && ./target/release/simbridge-server

# Companion app
cd companion && flutter run
```

### Test
```bash
# All tests
cargo test --workspace

# Coverage report
cargo tarpaulin --out Html

# Flutter tests
cd companion && flutter test
```

---

## Useful Resources

- **Issues**: https://github.com/yourusername/simbridge/issues
- **Wiki**: [Project Wiki](https://github.com/yourusername/simbridge/wiki)
- **Discussions**: https://github.com/yourusername/simbridge/discussions

---

*Quick Start Guide v0.1 | Last Updated: 2024-01-15*
