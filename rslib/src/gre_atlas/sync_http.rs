// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! HTTP end-to-end GRE Atlas sync loop against SimpleServer.

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::future::Future;
    use std::sync::LazyLock;

    use reqwest::Client;
    use reqwest::Url;
    use tempfile::tempdir;
    use tokio::sync::Mutex;
    use tokio::sync::MutexGuard;
    use tracing::Instrument;
    use tracing::Span;

    use crate::collection::Collection;
    use crate::collection::CollectionBuilder;
    use crate::error::Result;
    use crate::gre_atlas::gre_atlas_storage;
    use crate::gre_atlas::storage::SyncAttemptRow;
    use crate::log::set_global_logger;
    use crate::sync::http_server::default_ip_header;
    use crate::sync::http_server::SimpleServer;
    use crate::sync::http_server::SyncServerConfig;
    use crate::sync::login::SyncAuth;

    const TEST_HOST_KEY: &str = "b2619aa1529dfdc4248e6edbf3c1b2a2b014cf6d";

    struct Device {
        col: Collection,
        _dir: tempfile::TempDir,
    }

    async fn with_gre_sync_server<F, O>(op: F) -> Result<()>
    where
        F: FnOnce(SyncAuth, Client) -> O,
        O: Future<Output = Result<()>>,
    {
        let _ = set_global_logger(None);
        let base_folder = tempdir()?;
        std::env::set_var("SYNC_USER1", "user:pass");
        let (addr, server_fut) = SimpleServer::make_server(SyncServerConfig {
            host: "127.0.0.1".parse().unwrap(),
            port: 0,
            base_folder: base_folder.path().into(),
            ip_header: default_ip_header(),
        })
        .await
        .unwrap();
        tokio::spawn(server_fut.instrument(Span::current()));

        static LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
        let _lock: MutexGuard<()>;
        let endpoint = if let Ok(endpoint) = std::env::var("TEST_ENDPOINT") {
            _lock = LOCK.lock().await;
            endpoint
        } else {
            format!("http://{addr}/")
        };
        let auth = SyncAuth {
            hkey: TEST_HOST_KEY.into(),
            endpoint: Some(Url::try_from(endpoint.as_str()).unwrap()),
            io_timeout_secs: Some(30),
        };
        op(auth, Client::new()).await
    }

    fn new_device(name: &str) -> Result<Device> {
        let dir = tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col = CollectionBuilder::new(dir.path().join(format!("{name}.anki2"))).build()?;
        Ok(Device { col, _dir: dir })
    }

    fn record_practice(
        col: &mut Collection,
        answer: &str,
        response_time_ms: u32,
    ) -> Result<SyncAttemptRow> {
        let attempt = {
            let storage = gre_atlas_storage(col)?;
            let session = storage.create_session("practice")?;
            let q = storage.list_questions("", 1)?.pop().unwrap();
            storage.record_attempt(
                &q.id,
                &q.topic,
                q.difficulty,
                answer,
                answer.contains("correct"),
                response_time_ms,
                None,
                Some(&session.id),
            )?;
            let (attempts, _) = storage.pull_changes(-1, 100)?;
            attempts
                .into_iter()
                .find(|a| a.answer == answer)
                .ok_or_else(|| crate::error::AnkiError::InvalidInput {
                    source: snafu::FromString::without_source(format!(
                        "attempt with answer {answer:?} not found"
                    )),
                })?
        };
        col.state.gre_atlas_signals_cache = None;
        Ok(attempt)
    }

    fn all_attempts(col: &mut Collection) -> Result<Vec<SyncAttemptRow>> {
        let storage = gre_atlas_storage(col)?;
        let (attempts, _) = storage.pull_changes(-1, 10_000)?;
        Ok(attempts)
    }

    fn attempt_identity(a: &SyncAttemptRow) -> (String, i64, Option<String>, String) {
        (
            a.question_id.clone(),
            a.answered_at_secs.0,
            a.session_id.clone(),
            a.answer.clone(),
        )
    }

    fn assert_integrity(col: &mut Collection, expected_count: u32) -> Result<()> {
        let attempts = all_attempts(col)?;
        assert_eq!(
            attempts.len() as u32,
            expected_count,
            "unexpected attempt count"
        );
        let identities: HashSet<_> = attempts.iter().map(attempt_identity).collect();
        assert_eq!(
            identities.len(),
            attempts.len(),
            "duplicate review identities detected"
        );
        for attempt in &attempts {
            assert!(
                attempt.answered_at_secs.0 > 0,
                "attempt {} missing answered_at timestamp",
                attempt.id
            );
            assert!(
                attempt.mtime_secs.0 > 0,
                "attempt {} missing mtime timestamp",
                attempt.id
            );
        }
        Ok(())
    }

    async fn perform_sync(col: &mut Collection, auth: &SyncAuth, client: &Client) -> Result<()> {
        let response = col
            .gre_atlas_perform_sync(auth.clone(), client.clone())
            .await?;
        assert!(response.success, "sync failed: {}", response.message);
        assert!(
            response.status.as_ref().unwrap().pending_upload_count == 0,
            "pending uploads remain after sync"
        );
        Ok(())
    }

    /// Friday loop: desktop ↔ phone sync, offline review, reconnect, integrity.
    #[tokio::test]
    async fn friday_sync_loop_desktop_phone_offline_reconnect() -> Result<()> {
        with_gre_sync_server(|auth, client| async move {
            let mut desktop = new_device("desktop")?;
            let mut phone = new_device("phone")?;

            // 1. Create review on desktop.
            let desktop_attempt = record_practice(&mut desktop.col, "desktop-correct", 700)?;

            // 2. Verify phone receives it.
            perform_sync(&mut desktop.col, &auth, &client).await?;
            perform_sync(&mut phone.col, &auth, &client).await?;
            assert_integrity(&mut phone.col, 1)?;
            let phone_attempts = all_attempts(&mut phone.col)?;
            assert_eq!(phone_attempts[0].answer, "desktop-correct");
            assert_eq!(
                phone_attempts[0].answered_at_secs,
                desktop_attempt.answered_at_secs
            );

            // 3. Review on phone.
            let phone_attempt = record_practice(&mut phone.col, "phone-correct", 800)?;

            // 4. Verify desktop receives it.
            perform_sync(&mut phone.col, &auth, &client).await?;
            perform_sync(&mut desktop.col, &auth, &client).await?;
            assert_integrity(&mut desktop.col, 2)?;
            let desktop_attempts = all_attempts(&mut desktop.col)?;
            let answers: HashSet<_> = desktop_attempts.iter().map(|a| a.answer.as_str()).collect();
            assert!(answers.contains("desktop-correct"));
            assert!(answers.contains("phone-correct"));

            // 5–6. Disconnect network; review cards offline on both devices.
            let offline_desktop =
                record_practice(&mut desktop.col, "offline-desktop-correct", 650)?;
            let offline_phone = record_practice(&mut phone.col, "offline-phone-correct", 750)?;

            assert_integrity(&mut desktop.col, 3)?;
            assert_integrity(&mut phone.col, 3)?;

            // 7–8. Reconnect and sync both devices.
            perform_sync(&mut desktop.col, &auth, &client).await?;
            perform_sync(&mut phone.col, &auth, &client).await?;
            perform_sync(&mut desktop.col, &auth, &client).await?;
            perform_sync(&mut phone.col, &auth, &client).await?;

            // 9. Verify no duplicates, no missing reviews, history preserved.
            assert_integrity(&mut desktop.col, 4)?;
            assert_integrity(&mut phone.col, 4)?;

            let desktop_final = all_attempts(&mut desktop.col)?;
            let phone_final = all_attempts(&mut phone.col)?;
            let desktop_identities: HashSet<_> =
                desktop_final.iter().map(attempt_identity).collect();
            let phone_identities: HashSet<_> = phone_final.iter().map(attempt_identity).collect();
            assert_eq!(desktop_identities, phone_identities);

            for label in [
                "desktop-correct",
                "phone-correct",
                "offline-desktop-correct",
                "offline-phone-correct",
            ] {
                assert!(
                    desktop_final.iter().any(|a| a.answer == label),
                    "desktop missing {label}"
                );
            }

            // Timestamps from offline reviews survive merge.
            assert!(desktop_final
                .iter()
                .any(|a| a.answered_at_secs == offline_desktop.answered_at_secs));
            assert!(phone_final
                .iter()
                .any(|a| a.answered_at_secs == offline_phone.answered_at_secs));
            assert!(desktop_final
                .iter()
                .any(|a| a.answered_at_secs == phone_attempt.answered_at_secs));

            Ok(())
        })
        .await
    }

    /// Both devices create attempts offline with colliding auto-increment ids.
    #[tokio::test]
    async fn offline_id_collision_survives_server_merge() -> Result<()> {
        with_gre_sync_server(|auth, client| async move {
            let mut desktop = new_device("desktop")?;
            let mut phone = new_device("phone")?;

            // Seed a common baseline so both devices diverge from the same state.
            record_practice(&mut desktop.col, "baseline-correct", 500)?;
            perform_sync(&mut desktop.col, &auth, &client).await?;
            perform_sync(&mut phone.col, &auth, &client).await?;

            // Offline: each device records independently (both get local id = 2).
            record_practice(&mut desktop.col, "desktop-offline-correct", 600)?;
            record_practice(&mut phone.col, "phone-offline-correct", 700)?;

            // Reconnect: desktop syncs first, then phone.
            perform_sync(&mut desktop.col, &auth, &client).await?;
            perform_sync(&mut phone.col, &auth, &client).await?;
            perform_sync(&mut desktop.col, &auth, &client).await?;

            assert_integrity(&mut desktop.col, 3)?;
            assert_integrity(&mut phone.col, 3)?;

            let desktop_attempts = all_attempts(&mut desktop.col)?;
            let answers: HashSet<_> = desktop_attempts.iter().map(|a| a.answer.as_str()).collect();
            assert!(answers.contains("baseline-correct"));
            assert!(answers.contains("desktop-offline-correct"));
            assert!(answers.contains("phone-offline-correct"));

            Ok(())
        })
        .await
    }
}
