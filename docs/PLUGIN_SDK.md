# SimBridge Plugin SDK Guide

## Overview

SimBridge supports a plugin system that allows extending functionality without modifying the core codebase. Plugins can provide new simulator backends, authentication methods, streaming codecs, and more.

## Plugin Interface

Plugins are implemented in Rust and must implement the `Plugin` trait:

```rust
use simbridge_server::core::plugin::{Plugin, PluginContext};
use simbridge_shared::protocol::Message;

pub struct MyPlugin {
    // Plugin state
}

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my_plugin"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn initialize(&mut self, context: &PluginContext) -> Result<(), PluginError> {
        // Initialize plugin
        Ok(())
    }

    fn on_message(&mut self, message: &Message) -> Result<Option<Message>, PluginError> {
        // Process message
        Ok(None)
    }

    fn shutdown(&mut self) -> Result<(), PluginError> {
        // Cleanup
        Ok(())
    }
}
```

## Plugin Types

### Simulator Adapter Plugin

Add support for a new simulator/emulator:

```rust
use simbridge_server::adapters::interface::{SimulatorAdapter, AdapterError, SimulatorStatus};
use async_trait::async_trait;

pub struct CustomSimulatorAdapter {
    // Adapter state
}

#[async_trait]
impl SimulatorAdapter for CustomSimulatorAdapter {
    fn name(&self) -> &str {
        "custom-simulator"
    }

    async fn connect(&mut self) -> Result<(), AdapterError> {
        // Connect to simulator
        Ok(())
    }

    // Implement other required methods...
}
```

### Authentication Provider Plugin

Add custom authentication methods:

```rust
pub struct CustomAuthProvider {
    // Auth state
}

impl CustomAuthProvider {
    pub async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken, AuthError> {
        // Custom authentication logic
        Ok(token)
    }
}
```

### Streaming Codec Plugin

Add custom video encoding/decoding:

```rust
pub struct CustomCodec {
    // Codec state
}

impl CustomCodec {
    pub fn encode(&self, frame: &[u8]) -> Result<Vec<u8>, CodecError> {
        // Custom encoding
        Ok(encoded)
    }

    pub fn decode(&self, data: &[u8]) -> Result<Vec<u8>, CodecError> {
        // Custom decoding
        Ok(decoded)
    }
}
```

## Plugin Context

The `PluginContext` provides information about the server environment:

```rust
pub struct PluginContext {
    pub config_dir: PathBuf,  // Plugin configuration directory
    pub data_dir: PathBuf,     // Plugin data directory
    pub server_version: String, // Server version
}
```

## Loading Plugins

Plugins are loaded dynamically at runtime:

```rust
use simbridge_server::core::plugin::PluginManager;

let plugin_manager = PluginManager::new(context);
plugin_manager.load_plugin(Box::new(my_plugin)).await?;
```

## Plugin Configuration

Create a configuration file in the plugin's config directory:

```toml
[plugin]
name = "my_plugin"
version = "0.1.0"
enabled = true

[settings]
custom_setting = "value"
```

## Best Practices

1. **Error Handling**: Always return descriptive errors
2. **Resource Management**: Clean up resources in `shutdown()`
3. **Thread Safety**: Use `Send + Sync` for thread-safe plugins
4. **Logging**: Use the tracing crate for logging
5. **Testing**: Write comprehensive tests for your plugin

## Example Plugin

See the `plugins/` directory for example implementations.

## Plugin Distribution

Plugins can be distributed as:

1. **Dynamic Libraries**: `.so` (Linux), `.dylib` (macOS), `.dll` (Windows)
2. **Source Code**: Compiled with the server
3. **WASM Modules**: For cross-platform compatibility

## Security Considerations

- Plugins run with the same privileges as the server
- Validate all inputs from external sources
- Use secure communication channels
- Limit file system access
- Implement proper error handling
