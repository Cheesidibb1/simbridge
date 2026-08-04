# SimBridge

An open-source, cross-platform development platform that allows developers to remotely control, monitor, and interact with mobile simulators and emulators from a physical mobile device.

## Mission

Make an iPhone or Android phone act as a companion device for iOS Simulators and Android Emulators, enabling realistic testing of location-based, notification-based, and interactive applications.

## Features

- **Remote Control**: Control iOS Simulators and Android Emulators from your phone
- **Live Screen Streaming**: View simulator screens in near real-time
- **Touch & Gesture Support**: Send touch, gesture, keyboard, and device events
- **GPS Streaming**: Stream real GPS data from your physical phone
- **Notification Forwarding**: Receive simulator notifications on your phone
- **Clipboard Sync**: Bidirectional clipboard synchronization
- **File Transfer**: Upload and download files between phone and simulator
- **Session Recording**: Record and replay testing sessions
- **Multi-Simulator Support**: Control multiple simulators simultaneously

## Architecture

SimBridge consists of four major components:

1. **Companion Mobile App** (Flutter) - Runs on Android/iOS devices
2. **SimBridge Server** (Rust) - Desktop/server application for macOS/Windows/Linux
3. **Simulator Adapters** - Pluggable adapters for iOS Simulator and Android Emulator
4. **Shared Core Library** - Common protocol, models, networking, and utilities

## Quick Start

### Prerequisites

- Rust 1.70+ (for server)
- Flutter 3.16+ (for companion app and desktop dashboard)
- Xcode 15+ (for iOS Simulator support, macOS only)
- Android Studio with Emulator (for Android Emulator support)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/simbridge.git
cd simbridge

# Build the shared core library
cd shared
cargo build

# Build the server
cd ../server
cargo build --release

# Build the companion app
cd ../companion
flutter build apk  # Android
flutter build ios  # iOS

# Build the desktop dashboard
cd ../desktop
flutter build
```

### Running

```bash
# Start the server
./server/target/release/simbridge-server

# Run the companion app on your device
flutter run -d <device-id>
```

## Documentation

- [Architecture Guide](docs/ARCHITECTURE.md)
- [Developer Guide](docs/DEVELOPER.md)
- [API Reference](docs/API.md)
- [Plugin SDK Guide](docs/PLUGIN_SDK.md)
- [Deployment Guide](docs/DEPLOYMENT.md)
- [Contribution Guide](docs/CONTRIBUTING.md)
- [Troubleshooting Guide](docs/TROUBLESHOOTING.md)

## Development Status

This project is under active development. See the [Development Strategy](docs/DEVELOPER.md#development-strategy) for the current roadmap.

## License

MIT License - see LICENSE file for details

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

## Security

For security considerations, see the [Security documentation](docs/SECURITY.md).
