#!/usr/bin/env bash
# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
#
# Builds mobile_bridge for the active Xcode SDK/architecture and copies the
# static library to ${SRCROOT}/out/lib/libmobile_bridge.a for linking.

set -euo pipefail

if [[ -f "${HOME}/.cargo/env" ]]; then
    # Xcode build scripts run with a minimal PATH, so load Rust's normal shell setup.
    # shellcheck disable=SC1091
    source "${HOME}/.cargo/env"
fi

export PATH="${HOME}/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:${PATH}"

if [[ -z "${SRCROOT:-}" ]]; then
    SRCROOT="$(cd "$(dirname "$0")/.." && pwd)"
fi

if [[ -z "${PLATFORM_NAME:-}" ]]; then
    PLATFORM_NAME="iphonesimulator"
fi

if [[ -z "${ARCHS:-}" ]]; then
    ARCHS="arm64"
fi

REPO_ROOT="$(cd "${SRCROOT}/../.." && pwd)"
BRIDGE_CRATE="${REPO_ROOT}/mobile/mobile_bridge"
OUT_LIB_DIR="${SRCROOT}/out/lib"
CARGO_TARGET_DIR="${SRCROOT}/out/rust"

case "${PLATFORM_NAME:-}" in
    iphoneos)
        case "${ARCHS:-}" in
            *arm64*) RUST_TARGET="aarch64-apple-ios" ;;
            *)
                echo "error: unsupported iphoneos ARCHS=${ARCHS:-}" >&2
                exit 1
                ;;
        esac
        ;;
    iphonesimulator)
        case "${ARCHS:-}" in
            *arm64*) RUST_TARGET="aarch64-apple-ios-sim" ;;
            *x86_64*) RUST_TARGET="x86_64-apple-ios" ;;
            *)
                echo "error: unsupported iphonesimulator ARCHS=${ARCHS:-}" >&2
                exit 1
                ;;
        esac
        ;;
    macosx)
        echo "note: skipping mobile_bridge build for macOS platform ${PLATFORM_NAME}" >&2
        exit 0
        ;;
    *)
        echo "error: unsupported PLATFORM_NAME=${PLATFORM_NAME:-}" >&2
        exit 1
        ;;
esac

if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo not found in PATH" >&2
    exit 1
fi

if ! rustup target list --installed | grep -qx "${RUST_TARGET}"; then
    echo "Installing Rust target ${RUST_TARGET}..."
    rustup target add "${RUST_TARGET}"
fi

mkdir -p "${OUT_LIB_DIR}" "${CARGO_TARGET_DIR}"

echo "Building mobile_bridge for ${RUST_TARGET} (SDK=${SDKROOT:-unknown})..."
(
    cd "${REPO_ROOT}"
    CARGO_TARGET_DIR="${CARGO_TARGET_DIR}" \
        cargo build --release -p mobile_bridge --target "${RUST_TARGET}"
)

STATIC_LIB="${CARGO_TARGET_DIR}/${RUST_TARGET}/release/libmobile_bridge.a"
if [[ ! -f "${STATIC_LIB}" ]]; then
    echo "error: expected static library at ${STATIC_LIB}" >&2
    exit 1
fi

cp "${STATIC_LIB}" "${OUT_LIB_DIR}/libmobile_bridge.a"
echo "${RUST_TARGET}" > "${OUT_LIB_DIR}/rust_target.txt"
echo "Linked library: ${OUT_LIB_DIR}/libmobile_bridge.a"
