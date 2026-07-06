// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::env;
use std::sync::LazyLock;

pub fn version() -> &'static str {
    include_str!("../../.version").trim()
}

pub fn buildhash() -> &'static str {
    option_env!("BUILDHASH").unwrap_or("dev").trim()
}

pub fn rust_toolchain_channel() -> &'static str {
    option_env!("RUST_TOOLCHAIN_CHANNEL")
        .unwrap_or("Unknown")
        .trim()
}

pub(crate) fn sync_client_version() -> &'static str {
    static VER: LazyLock<String> = LazyLock::new(|| {
        format!(
            "anki,{version} ({buildhash}),{platform}",
            version = version(),
            buildhash = buildhash(),
            platform = env::var("PLATFORM").unwrap_or_else(|_| env::consts::OS.to_string())
        )
    });
    &VER
}

pub(crate) fn sync_client_version_short() -> &'static str {
    static VER: LazyLock<String> = LazyLock::new(|| {
        format!(
            "{version},{buildhash},{platform}",
            version = version(),
            buildhash = buildhash(),
            platform = env::consts::OS
        )
    });
    &VER
}
