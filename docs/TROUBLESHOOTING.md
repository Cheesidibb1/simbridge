# SimBridge Troubleshooting Guide

## Common Issues

### Server Issues

#### Server Won't Start

**Symptoms**: Server fails to start or crashes immediately

**Solutions**:
1. Check if port 8080 is already in use:
```bash
netstat -ano | findstr :8080  # Windows
lsof -i :8080  # macOS/Linux
```

2. Check logs for error messages:
```bash
RUST_LOG=debug cargo run
```

3. Verify database directory permissions:
```bash
ls -la simbridge.db
```

4. Ensure all dependencies are installed:
```bash
cargo build
```

#### Connection Refused

**Symptoms**: Cannot connect to server

**Solutions**:
1. Verify server is running:
```bash
curl http://localhost:8080/health
```

2. Check firewall settings:
```bash
# Windows
netsh advfirewall firewall add rule name="SimBridge" dir=in action=allow protocol=TCP localport=8080

# macOS/Linux
sudo ufw allow 8080/tcp
```

3. Ensure server is listening on correct interface:
```bash
# Use 0.0.0.0 to listen on all interfaces
cargo run -- --host 0.0.0.0
```

#### High CPU Usage

**Symptoms**: Server uses excessive CPU

**Solutions**:
1. Reduce streaming quality in configuration
2. Limit concurrent sessions
3. Check for stuck processes
4. Review database query performance

### Companion App Issues

#### Cannot Connect to Server

**Symptoms**: Companion app shows connection error

**Solutions**:
1. Verify server URL is correct
2. Ensure devices are on same network
3. Check if server is accessible:
```bash
curl http://<server-ip>:8080/health
```

4. Disable VPN on mobile device
5. Check firewall on server machine

#### GPS Permission Denied

**Symptoms**: GPS streaming fails with permission error

**Solutions**:
1. Enable location services on device
2. Grant location permission to app
3. Enable "Always allow" location for background streaming
4. Check if high-accuracy location is enabled

#### Screen Not Displaying

**Symptoms**: Screen is black or shows placeholder

**Solutions**:
1. Verify simulator is running
2. Check if screen streaming is enabled
3. Ensure sufficient network bandwidth
4. Try reducing streaming quality

### Simulator Adapter Issues

#### iOS Simulator Not Found

**Symptoms**: iOS Simulator adapter fails to find simulator

**Solutions**:
1. Ensure Xcode is installed
2. Verify Xcode Simulator is running:
```bash
xcrun simctl list devices
```

3. Check simctl is accessible:
```bash
xcrun simctl --version
```

4. Restart Xcode Simulator

#### Android Emulator Not Found

**Symptoms**: Android Emulator adapter fails to find emulator

**Solutions**:
1. Ensure Android Emulator is running
2. Verify ADB is installed:
```bash
adb version
```

3. List connected devices:
```bash
adb devices
```

4. Restart Android Emulator

#### Touch Events Not Working

**Symptoms**: Touch events from companion app don't affect simulator

**Solutions**:
1. Verify simulator is connected
2. Check if touch input is enabled
3. Ensure simulator screen is unlocked
4. Verify coordinate system matches

### Network Issues

#### High Latency

**Symptoms**: Delayed screen updates or touch responses

**Solutions**:
1. Check network connection quality
2. Reduce streaming quality
3. Lower frame rate
4. Use wired connection instead of Wi-Fi
5. Check for network congestion

#### Connection Drops

**Symptoms**: Frequent disconnections

**Solutions**:
1. Check network stability
2. Increase timeout settings
3. Verify WebSocket keep-alive is enabled
4. Check router for connection limits
5. Update network drivers

### Performance Issues

#### Low Frame Rate

**Symptoms**: Screen streaming is choppy

**Solutions**:
1. Reduce streaming resolution
2. Lower frame rate setting
3. Check server CPU usage
4. Close other resource-intensive applications
5. Use hardware acceleration if available

#### High Memory Usage

**Symptoms**: Server or app uses excessive memory

**Solutions**:
1. Reduce concurrent sessions
2. Lower streaming quality
3. Check for memory leaks
4. Restart server periodically
5. Increase available RAM

### Database Issues

#### Database Locked

**Symptoms**: Operations fail with database locked error

**Solutions**:
1. Check for other processes using database
2. Ensure proper cleanup of connections
3. Restart server
4. Check disk space
5. Verify file permissions

#### Migration Failed

**Symptoms**: Database migration fails

**Solutions**:
1. Backup existing database
2. Check migration files for errors
3. Manually apply migration if needed
4. Start with fresh database if possible

## Debugging

### Enable Debug Logging

**Server**:
```bash
RUST_LOG=debug cargo run
```

**Companion App**:
```bash
flutter run --debug
```

### View Logs

**Server Logs**:
```bash
# If running as service
journalctl -u simbridge -f

# If running directly
tail -f /var/log/simbridge/server.log
```

**Companion App Logs**:
```bash
flutter logs
```

### Network Debugging

**Test WebSocket Connection**:
```bash
wscat -c ws://localhost:8080/ws
```

**Test REST API**:
```bash
curl -v http://localhost:8080/health
```

**Trace Route**:
```bash
tracert <server-ip>  # Windows
traceroute <server-ip>  # macOS/Linux
```

## Getting Help

If you're still experiencing issues:

1. Check existing GitHub issues
2. Search the documentation
3. Create a new issue with:
   - Clear description of the problem
   - Steps to reproduce
   - Environment details
   - Relevant logs
   - Screenshots if applicable

4. Join the community chat for real-time help

## Reporting Bugs

When reporting bugs, include:

1. SimBridge version
2. Operating system and version
3. Full error message
4. Steps to reproduce
5. Expected vs actual behavior
6. Configuration (sanitized)
7. Logs and screenshots
