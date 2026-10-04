#!/usr/bin/env bash
# One command: (re)build the web app if needed, then start the server so an iPhone on the
# same Wi-Fi can open it in Safari.
#
#   ./scripts/serve-for-iphone.sh            # port 8080
#   ./scripts/serve-for-iphone.sh 9000       # custom port
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PORT="${1:-8080}"

if [ ! -f "$ROOT/companion/build/web/index.html" ]; then
  echo "ℹ️  No web build found; building it first…"
  "$ROOT/scripts/build-web.sh"
fi

cd "$ROOT/server"
exec cargo run --release -- --port "$PORT" --web-dir "$ROOT/companion/build/web"
