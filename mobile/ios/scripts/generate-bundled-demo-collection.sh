#!/usr/bin/env bash
# Regenerate the iOS bundled demo collection (collection.anki2 + greatlas.db).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
OUTPUT="${1:-$ROOT/mobile/ios/GREAtlasCompanion/Resources/DemoBundle}"

cd "$ROOT"
./ninja rslib:proto >/dev/null
cargo run -p mobile_bridge --bin generate_ios_demo_bundle -- "$OUTPUT"
