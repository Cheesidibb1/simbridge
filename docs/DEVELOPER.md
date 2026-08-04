# SimBridge Developer Guide

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Flutter 3.16 or later (for companion app and desktop dashboard)
- Xcode 15+ (for iOS Simulator support, macOS only)
- Android Studio with Emulator (for Android Emulator support)
- SQLite

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/simbridge.git
cd simbridge
```

2. Build the shared core library:
```bash
cd shared
cargo build
```

3. Build the server:
```bash
cd ../server
cargo build --release
```

4. Build the companion app (requires Flutter):
```bash
cd ../companion
flutter pub get
flutter build apk  # Android
flutter build ios  # iOS
```

5. Build the desktop dashboard:
```bash
cd ../desktop
flutter pub get
flutter build
```

### Running the Server

```bash
cd server
cargo run --release
```

The server will start on `http://0.0.0.0:8080` by default.

### Running the Companion App

```bash
cd companion
flutter run
```

### Running the Desktop Dashboard

```bash
cd desktop
flutter run -d windows
```

## Development Strategy

This project follows an incremental development approach:

1. ✅ Define architecture and repository structure
2. ✅ Implement shared core library
3. ✅ Implement backend/server
4. ✅ Implement simulator adapter interfaces
5. ✅ Add Android Emulator support
6. ✅ Add iOS Simulator support
7. ✅ Build companion app
8. ✅ Build desktop dashboard
9. ⏳ Integrate screen streaming (WebRTC)
10. ⏳ Integrate touch controls
11. ⏳ Integrate GPS streaming
12. ⏳ Integrate notification forwarding
13. ⏳ Integrate clipboard sync
14. ⏳ Integrate file transfer
15. ⏳ Implement session recording
16. ⏳ Optimize performance
17. ⏳ Improve UI/UX
18. ⏳ Complete documentation
19. ⏳ Prepare a production release

## Project Structure

```
simbridge/
├── shared/           # Shared Rust library
│   ├── src/
│   │   ├── protocol/  # Protocol definitions
│   │   ├── models/    # Data models
│   │   ├── networking/# Networking utilities
│   │   ├── auth/      # Authentication
│   │   ├── utils/     # Utilities
│   │   └── logging/   # Logging
│   └── Cargo.toml
├── server/           # Rust server
│   ├── src/
│   │   ├── core/      # Core functionality
│   │   ├── networking/# Networking layer
│   │   ├── adapters/  # Simulator adapters
│   │   ├── streaming/ # Screen streaming
│   │   ├── storage/   # Database
│   │   ├── recording/ # Session recording
│   │   └── metrics/   # Performance metrics
│   ├── migrations/    # Database migrations
│   └── Cargo.toml
├── companion/        # Flutter mobile app
│   ├── lib/
│   │   ├── networking/
│   │   ├── services/
│   │   ├── models/
│   │   ├── screens/
│   │   ├── widgets/
│   │   └── protocol/
│   └── pubspec.yaml
├── desktop/          # Flutter desktop app
│   ├── lib/
│   └── pubspec.yaml
├── adapters/         # Standalone adapter implementations
│   ├── android/
│   └── ios/
├── docs/             # Documentation
└── tests/            # Integration tests
```

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture information.

## Testing

### Running Tests

```bash
# Rust tests
cd shared
cargo test

cd ../server
cargo test

# Flutter tests
cd companion
flutter test

cd ../desktop
flutter test
```

### Writing Tests

#### Rust Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(2 + 2, 4);
    }
}
```

#### Flutter Tests

```dart
void main() {
  testWidgets('Example test', (WidgetTester tester) async {
    await tester.pumpWidget(MyWidget());
    expect(find.text('Hello'), findsOneWidget);
  });
}
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Write tests
5. Ensure all tests pass
6. Submit a pull request

## Code Style

### Rust

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow Rust naming conventions

### Flutter/Dart

- Use `flutter format` for formatting
- Use `flutter analyze` for static analysis
- Follow Effective Dart guidelines

## Debugging

### Server Debugging

```bash
RUST_LOG=debug cargo run
```

### Companion App Debugging

```bash
flutter run --debug
```

## Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common issues and solutions.
