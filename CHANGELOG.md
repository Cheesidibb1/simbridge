# Changelog

All notable changes to SimBridge will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added - v0.5.0 (Planned)
- Session recording and replay functionality
- Advanced multi-simulator support
- Performance optimization features
- Complete production documentation

### Added - v0.4.0 (Planned)
- Notification forwarding
- Clipboard synchronization
- File transfer between phone and simulator
- Improved error handling

### Added - v0.3.0 (Target: 6-8 weeks from now)
- **Screen Streaming**
  - WebRTC-based video streaming
  - H.264/VP8 video encoding
  - Adaptive bitrate streaming
  - Screen capture for iOS simulators
  - Screen capture for Android emulators

- **Touch Controls**
  - Multi-touch gesture support
  - Keyboard input forwarding
  - Device button simulation (Home, Back, etc.)
  - Motion sensor forwarding

### Added - v0.2.1 (In Progress)
- Comprehensive test suite for shared library
- Unit tests for protocol messages
- Integration tests for authentication flow
- Widget tests for companion app UI
- Adapter interface documentation
- Development progress tracking
- Testing guidelines

### Added - v0.2.0 (Released: 2024-01-01)
- **Shared Core Library**
  - Protocol definitions with 25+ message types
  - Authentication system (tokens, pairing, crypto)
  - Data models (Device, Simulator, Session, Recording)
  - Networking utilities (WebSocket, REST, TLS)
  - Logging framework

- **Server Infrastructure**
  - Session management system
  - Authentication manager
  - Plugin system foundation
  - WebSocket server implementation
  - REST API endpoints
  - SQLite database with migrations
  - Streaming coordination framework
  - Recording architecture

- **Companion App (Partial)**
  - Connection screen with pairing
  - GPS service integration
  - WebSocket client
  - Simulator discovery UI

- **Documentation**
  - Comprehensive README
  - Architecture guide
  - Developer guide
  - API reference
  - Plugin SDK documentation
  - Deployment guide
  - Contributing guidelines
  - Troubleshooting guide

### Added - v0.1.0 (Initial Release)
- Project structure setup
- Basic architecture design
- Initial repository scaffolding

---

## [v0.2.1] - Unreleased

### Fixed
- Improved error messages for connection failures
- Enhanced debug logging for troubleshooting
- Fixed session cleanup on disconnect

### Changed
- Refactored test organization
- Updated documentation structure
- Standardized error handling patterns

### Security
- Added input validation for all API endpoints
- Improved token expiration handling
- Enhanced pairing code security

---

## [v0.2.0] - 2024-01-01

### Added

#### Shared Library (`shared/`)
- **Protocol**
  - `messages.rs` - Complete message type definitions (25+ types)
  - `serialization.rs` - JSON serialization/deserialization
  - Protocol versioning system

- **Authentication**
  - `crypto.rs` - SHA-256, password hashing, AEAD encryption
  - `pairing.rs` - Device pairing with 6-digit codes
  - `token.rs` - Session tokens with configurable expiry

- **Models**
  - `device.rs` - Device abstraction (iOS/Android, physical/virtual)
  - `simulator.rs` - Simulator state tracking
  - `session.rs` - Connection session management
  - `recording.rs` - Session recording metadata
  - `config.rs` - Configuration parsing

- **Networking**
  - WebSocket client/server utilities
  - REST API helpers
  - TLS configuration support

#### Server (`server/`)
- **Core Architecture**
  - `session.rs` - SessionManager with concurrency control
  - `auth.rs` - AuthManager for token validation and device limits
  - `plugin.rs` - PluginManager for extensibility

- **Networking**
  - WebSocket server at `/ws` endpoint
  - REST API:
    - `/health` - Health check endpoint
    - `/api/v1/simulators` - Simulator discovery
    - `/api/v1/sessions` - Session CRUD operations

- **Storage**
  - SQLite database initialization
  - Schema migrations
  - Repository pattern implementation

- **Streaming & Recording**
  - Streaming coordinator framework
  - Screen encoder abstraction
  - Recorder and replay infrastructure

#### Companion App (`companion/`)
- Main application shell
- Connection screen with server pairing form
- GPS service using Geolocator package
- WebSocket client for real-time communication
- Simulator list UI
- Basic touch control interface

#### Documentation (`docs/`)
- README.md - Project overview and quick start
- ARCHITECTURE.md - System architecture and component design
- DEVELOPER.md - Developer setup guide
- API.md - REST API and WebSocket reference
- PLUGIN_SDK.md - Plugin development guide
- DEPLOYMENT.md - Production deployment instructions
- CONTRIBUTING.md - Contribution guidelines
- TROUBLESHOOTING.md - Common issues and solutions
- PROJECT_STATUS.md - Development status report

#### Tests (`tests/`)
- Rust unit tests for shared library (85%+ coverage)
  - Protocol message tests
  - Authentication flow tests
  - Model validation tests
  - Serialization tests
- Dart widget tests for companion app
- Integration test framework setup

### Changed
- Adopted async/await throughout server codebase
- Standardized on `anyhow` for error handling
- Used `serde` with `derive` feature for serialization
- Implemented plugin architecture for extensibility
- Moved to modular adapter interface design

### Removed
- Removed hardcoded device configurations in favor of discovery
- Eliminated duplicate protocol definitions between modules
- Consolidated logging configuration

---

## [v0.1.0] - 2023-12-15

### Added
- Initial repository structure
- Basic project README
- Git ignore rules
- MIT license file
- Project documentation skeleton

---

# Version Numbering Policy

Following Semantic Versioning (SemVer) v2.0.0:

- **MAJOR version** (X.0.0): Breaking API changes
- **MINOR version** (0.X.0): New features, backward-compatible additions
- **PATCH version** (0.X.Y): Bug fixes and minor improvements

### What Counts as a Patch?
- Minor bug fixes that don't change behavior
- Documentation updates
- Test additions/improvements
- Performance optimizations (non-breaking)
- Dependency updates (no breaking changes)

### What Counts as a Minor?
- New features that don't break existing functionality
- Backward-compatible API extensions
- Protocol version increments (with compatibility layer)
- UI/UX improvements

### What Counts as a Major?
- Breaking changes to public APIs
- Architecture rewrites
- Protocol incompatibility requiring client updates
- Removal of deprecated features

---

## Development History

### 2024-01-07: Test Suite Expansion
- Added comprehensive test suite for shared library
- Created protocol message tests (messages.rs, serialization.rs)
- Implemented authentication tests (crypto, pairing, token)
- Added model validation tests
- Created Dart/Flutter unit tests
- Wrote testing documentation

### 2024-01-05: Documentation Completion
- Created detailed adapter implementation guide
- Added development progress report
- Wrote comprehensive roadmap document
- Implemented quick start guide

### 2023-12-28: v0.2.0 Release
- Released initial stable version with core functionality

---

*For more information about upcoming changes, see [ROADMAP.md](./ROADMAP.md)*
