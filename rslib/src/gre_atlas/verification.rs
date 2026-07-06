// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::GreAtlasClientPlatform;
use anki_proto::brainlift::GreAtlasVerificationResponse;

use crate::collection::Collection;
use crate::error::Result;
use crate::gre_atlas::gre_atlas_storage;
use crate::gre_atlas::questions::eval_pipeline::DUPLICATE_SIMILARITY_MAX;
use crate::gre_atlas::questions::llm::GreAtlasAiConfig;
use crate::version;

pub const VERIFICATION_UNKNOWN: &str = "Unknown";

impl Collection {
    pub fn gre_atlas_get_verification(
        &mut self,
        client: GreAtlasClientPlatform,
    ) -> Result<GreAtlasVerificationResponse> {
        let sync = gre_atlas_storage(self).ok().and_then(|storage| {
            storage
                .sync_status()
                .ok()
                .map(|status| (status.current_usn, status.pending_upload_count))
        });

        let (desktop_build, mobile_build) = client_build_labels(client);
        let (sync_status, offline_queue, conflict_resolution) = sync_fields(sync);
        let duplicate_protection = duplicate_protection_label();
        let commit_hash = version::buildhash().to_string();
        let app_version = version::version().to_string();
        let rust_version = version::rust_toolchain_channel().to_string();
        let ai_enabled = ai_enabled_label();

        Ok(GreAtlasVerificationResponse {
            desktop_build,
            mobile_build,
            sync_status,
            offline_queue,
            conflict_resolution,
            duplicate_protection,
            commit_hash,
            app_version,
            rust_version,
            ai_enabled,
        })
    }
}

fn client_build_labels(client: GreAtlasClientPlatform) -> (String, String) {
    let desktop_platform = std::env::consts::OS;
    let mobile_platform = if cfg!(target_os = "ios") {
        "ios"
    } else if cfg!(any(target_os = "android", target_os = "linux")) {
        "mobile"
    } else {
        "mobile"
    };

    match client {
        GreAtlasClientPlatform::GreAtlasClientDesktop => (
            format_build_label(desktop_platform),
            VERIFICATION_UNKNOWN.into(),
        ),
        GreAtlasClientPlatform::GreAtlasClientMobile => (
            VERIFICATION_UNKNOWN.into(),
            format_build_label(mobile_platform),
        ),
        GreAtlasClientPlatform::GreAtlasClientUnspecified => (
            format_build_label(desktop_platform),
            VERIFICATION_UNKNOWN.into(),
        ),
    }
}

fn format_build_label(platform: &str) -> String {
    format!(
        "{} ({}) {}",
        version::version(),
        version::buildhash(),
        platform
    )
}

fn sync_fields(sync: Option<(i32, u32)>) -> (String, String, String) {
    let Some((current_usn, pending_upload_count)) = sync else {
        return (
            VERIFICATION_UNKNOWN.into(),
            VERIFICATION_UNKNOWN.into(),
            VERIFICATION_UNKNOWN.into(),
        );
    };

    let sync_status = if pending_upload_count == 0 {
        format!("Up to date (USN {current_usn})")
    } else {
        format!("{pending_upload_count} pending upload (USN {current_usn})")
    };
    let offline_queue = if pending_upload_count == 0 {
        "Empty".into()
    } else {
        format!("{pending_upload_count} row(s) queued")
    };
    let conflict_resolution = "mtime_secs LWW".into();

    (sync_status, offline_queue, conflict_resolution)
}

fn duplicate_protection_label() -> String {
    format!(
        "Active (similarity ≥ {:.2})",
        DUPLICATE_SIMILARITY_MAX
    )
}

fn ai_enabled_label() -> String {
    if GreAtlasAiConfig::from_env().is_some() {
        "Enabled".into()
    } else {
        "Disabled".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_client_reports_desktop_build_only() {
        let (desktop, mobile) =
            client_build_labels(GreAtlasClientPlatform::GreAtlasClientDesktop);
        assert!(desktop.contains(version::version()));
        assert_eq!(mobile, VERIFICATION_UNKNOWN);
    }

    #[test]
    fn mobile_client_reports_mobile_build_only() {
        let (desktop, mobile) =
            client_build_labels(GreAtlasClientPlatform::GreAtlasClientMobile);
        assert_eq!(desktop, VERIFICATION_UNKNOWN);
        assert!(mobile.contains(version::version()));
    }

    #[test]
    fn sync_fields_unknown_when_sidecar_unavailable() {
        let (sync_status, offline_queue, conflict_resolution) = sync_fields(None);
        assert_eq!(sync_status, VERIFICATION_UNKNOWN);
        assert_eq!(offline_queue, VERIFICATION_UNKNOWN);
        assert_eq!(conflict_resolution, VERIFICATION_UNKNOWN);
    }

    #[test]
    fn sync_fields_describe_pending_queue() {
        let (sync_status, offline_queue, conflict_resolution) = sync_fields(Some((3, 2)));
        assert!(sync_status.contains("2 pending upload"));
        assert_eq!(offline_queue, "2 row(s) queued");
        assert_eq!(conflict_resolution, "mtime_secs LWW");
    }
}
