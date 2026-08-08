# SimBridge Shared Library Tests

## Overview

This directory contains unit tests for the SimBridge shared core library, including protocol definitions, authentication, models, networking utilities, and other shared components.

## Test Organization

```
tests/rust/shared/
├── auth/          # Authentication tests
│   ├── crypto.rs  # Cryptographic operations
│   ├── pairing.rs # Device pairing logic
│   └── token.rs   # Token management
├── models/        # Data model tests
│   ├── config.rs
│   ├── device.rs
│   ├── recording.rs
│   ├── session.rs
│   └── simulator.rs
├── networking/    # Networking tests
│   ├── rest.rs    # REST API client
│   └── websocket.rs
└── protocol/      # Protocol tests
    ├── messages.rs  # Message types and payloads
    └── serialization.rs
```

## Running Tests

### Basic Test Execution

```bash
cd shared
cargo test
```

### Run Specific Test Modules

```bash
# Run all auth tests
cargo test --lib auth

# Run protocol message tests only
cargo test --lib protocol::messages

# Run all model tests
cargo test --lib models
```

### Test with Coverage

```bash
# Install tarpaulin if not present
cargo install cargo-tarpaulin

# Run coverage report
cargo tarpaulin --out Html --out lcov
```

## Test Categories

### Protocol Tests (`protocol/messages.rs`)

Tests cover:
- Message creation and serialization
- Request ID handling
- All message type variants
- Payload structures (GPS, Touch, Gestures, File Transfer)
- Round-trip serialization/deserialization

### Authentication Tests (`auth/`)

**Crypto Tests:**
- Pairing code generation
- Session token generation
- Password hashing and verification
- Challenge-response authentication
- Key pair generation
- Encryption/decryption with AEAD

**Pairing Tests:**
- Pairing session creation
- Session expiration handling
- Session completion tracking
- Finding sessions by pairing code

**Token Tests:**
- Token creation with expiry
- Explicit expiration time setting
- Token serialization/deserialization
- Expiration status checking

### Model Tests (`models/`)

**Device Tests:**
- Device creation and structure
- Platform identification (iOS/Android)
- Capability flags
- Physical vs emulator distinction

**Simulator Tests:**
- Simulator state management
- OS version tracking
- Running state
- Screen size information

**Session Tests:**
- Session lifecycle
- Stream configuration
- Connection/disconnection tracking
- Activity timestamps

**Recording Tests:**
- Recording status states
- Duration and file size tracking
- Session association

### Networking Tests (`networking/`)

**REST API Tests:**
- Health check endpoint construction
- Simulator list endpoint
- Session management endpoints
- Error handling

**WebSocket Tests:**
- WebSocket URL construction
- Message structure validation
- Connection flow

## Test Patterns

All tests follow Rust's standard testing patterns:

```rust
#[test]
fn test_function_name() {
    // Arrange
    let input = ...;
    
    // Act
    let result = subject.action(input);
    
    // Assert
    assert_eq!(result, expected);
}
```

## Integration Tests

Integration tests are located in `tests/rust/integration/` and test the complete system flow:

```bash
# Run integration tests only
cargo test --test integration

# Run all tests including integration
cargo test --all-targets
```

## Expected Test Results

### Successful Test Run

You should see output similar to:

```
running 150 tests
test shared::auth::crypto::tests::test_encryption_decryption ... ok
test shared::models::simulator::tests::test_simulator_creation ... ok
...

test result: ok. 148 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```

### Test Failures

If tests fail, check:
1. Protocol version compatibility
2. Serialization format changes
3. Dependency version conflicts
4. Platform-specific issues (Windows vs Linux/Mac)

## Continuous Integration

Tests should pass in CI before merging any changes to the shared library.

### GitHub Actions Example

```yaml
name: Rust Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-action@v1
        with:
          command: test
          args: --workspace --all-targets
```

## Performance Testing

For serialization performance, use the benchmarking setup defined in `Cargo.toml`:

```bash
cd shared
cargo bench
```

This runs benchmarks defined under `[[bench]]` in Cargo.toml.
