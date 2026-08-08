# SimBridge Tests

This directory contains all tests for the SimBridge project.

## Structure

```
tests/
├── rust/               # Rust unit and integration tests
│   ├── shared/         # Tests for shared library
│   │   ├── protocol/
│   │   ├── auth/
│   │   ├── models/
│   │   └── networking/
│   ├── server/         # Tests for server components
│   │   ├── core/
│   │   ├── adapters/
│   │   └── streaming/
│   └── integration/    # Integration tests
├── dart/               # Dart/Flutter tests
│   ├── companion/      # Companion app tests
│   │   └── unit/
│   └── desktop/        # Desktop app tests
└── e2e/                # End-to-end tests
    └── mobile/         # Mobile E2E tests with Flutter Driver/Dartium
```

## Running Tests

### Rust Tests
```bash
cd shared
cargo test

cd ../server
cargo test

# Run with coverage
cargo tarpaulin --out Html
```

### Dart Tests
```bash
cd companion
flutter test

cd ../desktop
flutter test
```

### E2E Tests
Coming soon...

## Test Coverage Goals

- Core library: 90%+ coverage
- Server: 85%+ coverage  
- Companion app: 80%+ coverage
- Adapter interfaces: 100% coverage
