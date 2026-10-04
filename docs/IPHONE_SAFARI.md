# 📱 SimBridge on iPhone Safari

> Run the SimBridge companion in **Safari on an iPhone**, with no App Store, no Xcode signing and no
> TestFlight. The SimBridge server serves the app itself; you open one URL on the phone.

---

## ⚡ TL;DR: 3 steps

| # | Where | Command / action |
|---|-------|------------------|
| 1️⃣ | Mac / PC running the server | `./scripts/build-web.sh`, which builds the Flutter web app once |
| 2️⃣ | Same machine | `./scripts/serve-for-iphone.sh` (or `cd server && cargo run --release`) |
| 3️⃣ | iPhone, **same Wi-Fi** | Open the `http://<ip>:8080/` URL the server prints → **Share ▸ Add to Home Screen** |

The server prints the exact URL at startup:

```
INFO simbridge_server: Serving web app from companion/build/web
INFO simbridge_server: Server listening on 0.0.0.0:8080
INFO simbridge_server: Open on your iPhone (Safari): http://192.168.1.20:8080/
```

On the first launch the **host and port are already filled in**, because the app reads them from the
address in Safari's URL bar.

---

## 🧭 How it fits together

```
┌──────────────────────────┐        Wi-Fi / LAN        ┌──────────────────────────────┐
│  📱 iPhone Safari        │                           │  🖥️  SimBridge server (Rust)  │
│  Flutter web app         │ ── GET /  (app files) ──► │  • static files (ServeDir)   │
│  (CanvasKit renderer)    │ ── REST  /api/v1/* ─────► │  • REST  /health /api/v1/*   │
│                          │ ◄═ WebSocket /ws ════════►│  • WebSocket /ws (frames)    │
└──────────────────────────┘                           │  • CORS (cross-origin dev)   │
                                                       └───────────────┬──────────────┘
                                                                       │ adb / simctl
                                                                       ▼
                                                            📲 Simulator / Emulator
```

The page and its API come from **one origin**. That is deliberate: it avoids CORS preflights, and it avoids
Safari's *mixed-content* rule (an `https://` page can't open `ws://` sockets).

---

## 🧱 What changed, and why

### 🌐 Web shell: `companion/web/`

| File | Change | Why it matters on iPhone |
|------|--------|--------------------------|
| `index.html` | `viewport` with `maximum-scale=1` | iOS zooms the page when a focused text field uses a font under 16px. Pinning the scale stops it jumping when you tap the host/port fields. |
| | Cancels `gesturestart` / `gesturechange` / `gestureend` | iOS ignores `user-scalable=no`; these non-standard events are how its own pinch-zoom is cancelled. Without this, a two-finger pinch zooms **Safari** instead of reaching the app's pinch gesture. |
| | CSS: `touch-action: none`, `-webkit-touch-callout: none`, `user-select: none`, `overscroll-behavior: none`, tap-highlight off | Stops the Copy / Save Image callout and text-selection magnifier when you hold on the screen mirror (the app's long-press gesture), stops rubber-band bounce, and removes the 300 ms tap delay. |
| | `input, textarea { user-select: text; font-size: 16px }` | WebKit makes inputs uneditable under a `user-select: none` parent. This explicitly opts the text fields back in. |
| | No `viewport-fit=cover` (on purpose) | Flutter web doesn't read `env(safe-area-inset-*)`. Opting in to edge-to-edge would put the app bar under the notch and the buttons under the home indicator. Without it, Safari keeps the app in the safe area. |
| | `apple-mobile-web-app-*` metas, `theme-color`, title "SimBridge" | "Add to Home Screen" gives a full-screen, chrome-less app with a proper name and icon (it was `simbridge_client`). |
| | Loading splash + 30 s "still loading" hint + WebAssembly check | CanvasKit is about 2 MB of wasm, so the first load on a phone takes a moment. A visible splash that clears on Flutter's `flutter-first-frame` event beats a blank page. |
| `manifest.json` | Name, colors, `orientation: any` | The old manifest locked portrait, which is wrong for a mirror you'll want in landscape. |

### 📲 Flutter app: `companion/lib/`

| File | Change | Why it matters on iPhone |
|------|--------|--------------------------|
| `utils/platform_defaults.dart` *(new)* | First-run host/port/TLS come from the page URL on web | On a phone, `localhost` **is the phone**, so the old default could never connect. Pure function, unit-tested. |
| `providers/settings_provider.dart` | `effectiveTls` / `tlsRequired`; `wsUri` and `httpBaseUrl` use `effectiveTls` | If the page is `https://`, Safari refuses `ws://` and `http://` calls. TLS is forced rather than letting the connection fail silently. |
| `services/storage_service.dart` | Defaults via `PlatformDefaults.current` | Native builds are unchanged (same `localhost:8080`, no TLS). |
| `screens/onboarding_screen.dart`, `settings_screen.dart` | TLS switch locks on when required; host field uses URL keyboard, no autocorrect / auto-capitalise | iOS keyboards otherwise capitalise `192.168…` and "fix" hostnames. |
| `services/websocket_service.dart` | Staleness watchdog + `probe()` | iOS suspends sockets when the screen locks or the tab hides, often **without** a close event. The UI would show "connected" on a dead socket and freeze the mirror. Now: 3 silent ping intervals (45 s) → reconnect, and a probe on return to the foreground. |
| `providers/connection_provider.dart` | `WidgetsBindingObserver`; probes on `resumed` | Triggers that probe when you switch back to Safari. Registered only while a connection exists, so tests are unaffected. |
| `screens/control_screen.dart` | Clipboard "paste from this device" wrapped in `try/catch` | Safari only allows programmatic clipboard reads from a tap (with its own "Paste" bubble), and not at all on plain `http://` (insecure context). It used to throw. Now it shows a hint to long-press the field and paste. |
| `main.dart` | On web, caps Flutter's image cache (8 images / 48 MB) | The mirror decodes a full-resolution frame several times a second. The default 100 MB cache can push an iPhone tab over Safari's memory limit, and iOS then silently reloads the page. |

### 🦀 Server: `server/`

| File | Change | Why it matters on iPhone |
|------|--------|--------------------------|
| `src/main.rs` | Serves `companion/build/web` (`--web-dir`, `SIMBRIDGE_WEB_DIR`, or auto-detected) | The phone needs *something* to load. Same origin = no CORS, no mixed content. `.wasm` is served as `application/wasm`, which browsers require. |
| | Permissive CORS with **explicit** method / header lists (`--no-cors` to disable) | Lets you host the app on another origin (e.g. `flutter run -d web-server`). Explicit lists, because older Safari doesn't honour `*` in preflights. |
| | Prints the LAN URL for the iPhone | No hunting for the Mac's IP. |
| | `/test-webrtc` route; `/` setup page when no build exists | Reach the test client from the phone; a clear "build me first" page instead of a 404. |
| `static/landing.html` *(new)* | The setup page | |
| `test-webrtc.html` | See the next table | The test client was broken in every browser, and had Safari-specific gaps on top. |

### 🧪 WebRTC test client: `server/test-webrtc.html`

| Problem | Fix |
|---------|-----|
| Server frames are **binary** WebSocket messages. They arrive as a `Blob`, which `JSON.parse` can't read | `binaryType = 'arraybuffer'` + `TextDecoder` |
| **Safari emits no media section** in an offer unless you declare what you want to receive | `addTransceiver('video'/'audio', { direction: 'recvonly' })` |
| `pc` used outside its scope in `createOffer()` | Rewritten with `async/await` |
| Wrong envelope (`messageType: 'webrtcOffer'`, no `version` / `timestamp`), so the server rejected every message | `{ message_type: 'webrtc_offer', version, timestamp, payload }` |
| `peerConnection.addCandidate` and `sdpMlineIndex` don't exist | `addIceCandidate`, `sdpMLineIndex` |
| `setRemoteDescription(new RTCSessionDescription(string))` | `setRemoteDescription({ type: 'answer', sdp })` |
| Video element never given a stream | `video.srcObject = stream`, plus a "▶️ Tap to play" overlay for when iOS refuses autoplay |
| Stats called the non-existent `report.getVideoTracks()` | Real FPS / bitrate / RTT from `getStats()` |
| Default URL `ws://localhost:8080/ws` | Derived from the page's host; upgraded to `wss://` on https pages |
| Log used `innerHTML` with server-supplied text | `textContent` |
| 14px inputs, small buttons | 16px inputs (no iOS zoom), 44px touch targets |

---

## 🚀 Setup in detail

### 1. Build the web app (once, and after Dart changes)

```bash
./scripts/build-web.sh
```

This runs `flutter build web --release --base-href / --no-web-resources-cdn`.

* `--base-href /` matches where the server mounts the app.
* `--no-web-resources-cdn` **bundles CanvasKit**. By default Flutter fetches it from `gstatic.com`, so on a LAN
  with no internet the iPhone would sit on the splash screen forever.

Requires Flutter 3.22+ (the project already uses `flutter_bootstrap.js`).

### 2. Run the server

```bash
./scripts/serve-for-iphone.sh          # builds first if needed, then runs on :8080
# or manually:
cd server && cargo run --release -- --port 8080
```

| Flag | Meaning |
|------|---------|
| `--web-dir <path>` | Serve a build from somewhere else (also `SIMBRIDGE_WEB_DIR`) |
| `--no-cors` | Don't send CORS headers (fine for same-origin use) |
| `--host 0.0.0.0` | Already the default, so the server is reachable from the phone |

On startup, the server prints a generated password. The companion asks for it when you select a simulator and keeps
it in memory only. To use a custom password, configure `SIMBRIDGE_PASSWORD` with at least 12 characters before
starting the server. For example, in PowerShell:

```powershell
$env:SIMBRIDGE_PASSWORD = "replace-with-a-long-random-password"
.\scripts\serve-for-iphone.bat
```

The WebSocket uses a fresh HMAC-SHA256 challenge for each connection, so the password itself is not sent over the
socket. Use HTTPS/WSS when connecting across an untrusted network.

### 3. Open it on the iPhone

1. Join the **same Wi-Fi** as the server.
2. In **Safari**, open the printed `http://<ip>:8080/`.
3. Check the server address on the onboarding screen (pre-filled) → **Continue**.
4. Optional: **Share ▸ Add to Home Screen** for a full-screen app with its own icon.

> 💡 If the page won't load, first try `http://<ip>:8080/health` in Safari. If that fails too, it's the network or a
> firewall on the Mac (see Troubleshooting), not the app.

---

## 🔐 Should I use HTTPS?

Plain `http://` on your LAN **works** for everything the app does today. HTTPS buys you a few extras:

| Capability | `http://` (LAN) | `https://` |
|------------|:---:|:---:|
| Mirror, touch, gestures, buttons, GPS entry | ✅ | ✅ |
| Add to Home Screen | ✅ | ✅ |
| "Paste from this device" button | ❌ (falls back to long-press ▸ Paste) | ✅ |
| Flutter service worker / offline cache | ❌ | ✅ |

The server speaks plain HTTP/WS itself. For HTTPS, put a TLS-terminating proxy in front (Caddy, nginx,
`tailscale serve`, …), and make sure it forwards **WebSocket upgrades** on `/ws`. The app detects an
`https://` page and switches to `wss://` / `https://` automatically (the TLS switch shows as locked on).
This path has not been exercised here; the app-side logic is covered by `platform_defaults_test.dart`.

---

## 📋 iPhone Safari feature notes

| Feature | Status | Notes |
|---------|:---:|-------|
| 🖼️ Screen mirror | ✅ | Frames are base64 PNG over WebSocket (server-side design, unchanged) |
| 👆 Tap / swipe / long-press / pinch | ✅ | Pinch is cancelled at the Safari level so it reaches the app |
| 🔘 Device buttons | ✅ | |
| 📍 GPS entry | ✅ | Typed / preset coordinates. Not the phone's real GPS, which needs HTTPS and a code change |
| 📋 Clipboard → simulator | ✅ | Type or long-press ▸ Paste |
| 📋 Read phone clipboard button | ⚠️ | Needs HTTPS and a tap (Safari's "Paste" bubble) |
| 🔔 Notifications banner | ✅ | In-app only, no web push |
| 🔒 Screen lock / switching apps | ⚠️ | iOS suspends the socket; the app reconnects when you return |
| ⛶ Browser fullscreen API | ❌ | iPhone Safari doesn't support it. Use **Add to Home Screen** |
| 📳 Haptics | ❌ | Not available to web pages on iOS |

---

## 🛠️ Troubleshooting

| Symptom | Likely cause → fix |
|---------|-------------------|
| Page never loads | Phone isn't on the same Wi-Fi, or a firewall blocks the port. Try `http://<ip>:8080/health` in Safari. |
| Spinner forever, then the "Still loading…" hint | The build was made without `--no-web-resources-cdn` and there's no internet. Rebuild with `./scripts/build-web.sh`. |
| Setup page instead of the app | No build found. Run `./scripts/build-web.sh`, or pass `--web-dir`. |
| "Connection failed" on the simulator list | The host/port on the onboarding screen is wrong. Fix it in **Settings** (gear icon). |
| Works on `http://` but not behind a proxy | The proxy must pass WebSocket upgrades on `/ws`. |
| Page reloads by itself during mirroring | iOS killed the tab for memory. Close other Safari tabs and reopen. (The server currently sends frames at a fixed rate, so the quality / FPS settings don't reduce load yet.) |
| Zoomed in after tapping a field | Hard-reload so the new `viewport` meta applies. If it was added to the Home Screen earlier, remove it and add it again. |
| Old version still showing | Safari caches hard. Pull-to-refresh, or **Settings ▸ Safari ▸ Advanced ▸ Website Data** ▸ remove the site. |

---

## ✅ Verification status (read this)

| Check | Result |
|-------|:---:|
| Rust server builds (`cargo build`) | ✅ |
| Serves files, `.wasm` as `application/wasm`, `/` fallback, `/health`, `/test-webrtc` (live server) | ✅ |
| CORS preflight returns explicit method / header lists (live server) | ✅ |
| `test-webrtc.html`: connect → offer (with media sections) → send → answer, over real binary WebSocket frames, in a phone-sized touch-emulated browser | ✅ |
| `index.html`: splash clears on `flutter-first-frame`, all three `gesture*` events cancelled, no JS errors | ✅ |
| All 39 Dart files parse without syntax errors | ✅ |
| `flutter analyze` / `flutter test` / `flutter build web` | ⛔ **Not run.** The environment this was built in can't reach Flutter's SDK host. |
| Real iPhone / WebKit | ⛔ **Not run.** Checks used Chromium with iPhone emulation, so WebKit-specific behaviour is reasoned from the code, not observed. |

So please run, once:

```bash
cd companion && flutter pub get && flutter analyze && flutter test test/utils
./scripts/build-web.sh
```

`companion/test/widget_test.dart` is the untouched Flutter template (it references a `MyApp` that doesn't exist),
so a plain `flutter test` fails on it regardless of these changes. Delete or replace it.

---

## 🔎 Things noticed but deliberately not changed

* **Mirroring and touch are Android-only on the server today.** `capture_screen_frame`, `handle_touch_event` and
  `handle_gesture` in `server/src/networking/websocket.rs` always build an `AndroidEmulatorAdapter`. The Safari
  client can drive an Android emulator now. An iOS Simulator on the server needs the `IosSimulatorAdapter`
  wired into those handlers.
* **WebRTC signaling is a stub.** `handle_webrtc_offer` echoes the offer back as the "answer". The test page says so
  in its log, rather than failing mysteriously.
* **Frames are base64 PNG at about 10 fps.** That works, but it's heavy on a phone. JPEG or binary frames would
  help later, as a protocol change.
