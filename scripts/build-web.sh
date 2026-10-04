#!/usr/bin/env bash
# Builds the SimBridge companion as a web app that runs in iPhone Safari.
# The server (simbridge-server) finds companion/build/web on its own and serves it.
#
#   ./scripts/build-web.sh            # release build (what you want on a phone)
#   ./scripts/build-web.sh --profile  # profile build, for performance tracing
set -euo pipefail

cd "$(dirname "$0")/../companion"

if ! command -v flutter >/dev/null 2>&1; then
  echo "❌ flutter not found on PATH. Install Flutter 3.22+ (https://docs.flutter.dev/get-started/install)." >&2
  exit 1
fi

MODE="--release"
if [ "${1:-}" = "--profile" ]; then MODE="--profile"; fi

echo "📦 flutter pub get"
flutter pub get

# --base-href /            : the server serves the app from the site root
# --no-web-resources-cdn   : bundle CanvasKit (the renderer) instead of fetching it from
#                            gstatic.com, so the app loads on a Wi-Fi/LAN with no internet
echo "🏗️  flutter build web $MODE"
flutter build web "$MODE" --base-href / --no-web-resources-cdn

echo
echo "✅ Built companion/build/web"
echo "   Start the server:  cd server && cargo run --release"
echo "   Then open the 'Open on your iPhone (Safari)' URL it prints."
