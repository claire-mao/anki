// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Before/after parity checks for GRE Atlas metrics after sidecar bundle sync.

#[cfg(test)]
mod test {
    use anki_proto::brainlift::GetDashboardRequest;
    use anki_proto::brainlift::GetStudyPlanRequest;

    use crate::collection::Collection;
    use crate::collection::CollectionBuilder;
    use crate::error::Result;
    use crate::gre_atlas::abstention::MIN_PERFORMANCE_ATTEMPTS;
    use crate::gre_atlas::calibration::OutcomeInputs;
    use crate::gre_atlas::calibration::READINESS_MODEL_VERSION;
    use crate::gre_atlas::gre_atlas_storage;
    use crate::gre_atlas::storage::SyncBundle;
    use crate::gre_atlas::storage::SyncPredictionRow;
    use crate::timestamp::TimestampSecs;

    /// Sidecar-derived fields that must match after a full bundle sync when
    /// collections are identical.
    #[derive(Debug, Clone, PartialEq)]
    struct SidecarMetricsSnapshot {
        performance_value: Option<f32>,
        performance_attempt_count: u32,
        performance_sufficient: bool,
        readiness_projected: Option<f32>,
        readiness_confidence: String,
        readiness_sufficient: bool,
        calibration_total_predictions: u32,
        calibration_resolved: u32,
        calibration_brier: Option<f32>,
        recent_activity_count: usize,
        daily_practice_target: u32,
        study_recommendations_count: usize,
    }

    /// Collection-derived fields; parity requires Anki collection sync (not
    /// available on iOS today).
    #[derive(Debug, Clone, PartialEq)]
    struct CollectionMetricsSnapshot {
        memory_studied_cards: u32,
        memory_coverage_ratio: f32,
        memory_sufficient: bool,
        coverage_observed_leaves: u32,
    }

    fn isolated_col() -> Result<(Collection, tempfile::TempDir)> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col = CollectionBuilder::new(dir.path().join("test.anki2")).build()?;
        Ok((col, dir))
    }

    fn seed_practice_attempts(col: &mut Collection, count: u32) -> Result<()> {
        let storage = gre_atlas_storage(col)?;
        let session = storage.create_session("practice")?;
        let questions: Vec<_> = storage.list_questions("", count.max(1))?;
        for i in 0..count {
            let q = &questions[(i as usize) % questions.len()];
            storage.record_attempt(
                &q.id,
                &q.topic,
                q.difficulty,
                &q.correct_answer,
                i % 5 != 0,
                800 + i,
                None,
                Some(&session.id),
            )?;
        }
        col.state.gre_atlas_signals_cache = None;
        Ok(())
    }

    fn capture_sidecar_metrics(col: &mut Collection) -> Result<SidecarMetricsSnapshot> {
        let scores = col.gre_atlas_get_scores()?;
        let dashboard = col.gre_atlas_get_dashboard(GetDashboardRequest {
            recent_activity_limit: 10,
            topic_insight_limit: 5,
        })?;
        let study_plan = col.gre_atlas_get_study_plan(GetStudyPlanRequest { limit: 10 })?;
        let calibration = col.gre_atlas_get_readiness_calibration()?;

        let performance = scores.performance.unwrap_or_default();
        let readiness = scores.readiness.unwrap_or_default();
        let stats = calibration.calibration.unwrap_or_default();
        let daily = study_plan.daily_plan.unwrap_or_default();

        Ok(SidecarMetricsSnapshot {
            performance_value: performance.value,
            performance_attempt_count: performance.attempt_count,
            performance_sufficient: performance.sufficient_data,
            readiness_projected: readiness.projected_score,
            readiness_confidence: readiness.confidence_level,
            readiness_sufficient: readiness.sufficient_data,
            calibration_total_predictions: stats.total_predictions,
            calibration_resolved: stats.resolved_outcomes,
            calibration_brier: stats.brier_score,
            recent_activity_count: dashboard.recent_activity.len(),
            daily_practice_target: daily
                .tasks
                .iter()
                .find(|task| task.id == "practice_questions")
                .map(|task| task.target_count)
                .unwrap_or(0),
            study_recommendations_count: study_plan.recommendations.len(),
        })
    }

    fn capture_collection_metrics(col: &mut Collection) -> Result<CollectionMetricsSnapshot> {
        let scores = col.gre_atlas_get_scores()?;
        let dashboard = col.gre_atlas_get_dashboard(Default::default())?;
        let memory = scores.memory.unwrap_or_default();
        let coverage = dashboard.coverage.unwrap_or_default();
        Ok(CollectionMetricsSnapshot {
            memory_studied_cards: memory.studied_cards,
            memory_coverage_ratio: memory.coverage_ratio,
            memory_sufficient: memory.sufficient_data,
            coverage_observed_leaves: coverage.covered_leaf_count,
        })
    }

    fn sync_desktop_bundle_to_mobile(
        desktop: &mut Collection,
        mobile: &mut Collection,
    ) -> Result<()> {
        let export = gre_atlas_storage(desktop)?.pull_sync_bundle(0, 5000)?;
        gre_atlas_storage(mobile)?.apply_sync_bundle(&export)?;
        mobile.state.gre_atlas_signals_cache = None;
        Ok(())
    }

    /// Desktop practice → mobile merge; sidecar metrics must match when
    /// collections start identical (simulates same profile, sidecar-only sync).
    #[test]
    fn bundle_sync_sidecar_metrics_match_after_desktop_to_mobile() -> Result<()> {
        let (mut desktop, _d_dir) = isolated_col()?;
        let (mut mobile, _m_dir) = isolated_col()?;

        seed_practice_attempts(&mut desktop, MIN_PERFORMANCE_ATTEMPTS)?;

        let desktop_before = capture_sidecar_metrics(&mut desktop)?;
        let mobile_before = capture_sidecar_metrics(&mut mobile)?;
        assert_ne!(
            desktop_before.performance_attempt_count,
            mobile_before.performance_attempt_count
        );

        sync_desktop_bundle_to_mobile(&mut desktop, &mut mobile)?;

        let desktop_after = capture_sidecar_metrics(&mut desktop)?;
        let mobile_after = capture_sidecar_metrics(&mut mobile)?;

        assert_eq!(desktop_after, mobile_after);
        assert!(mobile_after.performance_sufficient);
        assert_eq!(
            mobile_after.performance_attempt_count,
            MIN_PERFORMANCE_ATTEMPTS
        );

        let desktop_collection = capture_collection_metrics(&mut desktop)?;
        let mobile_collection = capture_collection_metrics(&mut mobile)?;
        assert_eq!(desktop_collection, mobile_collection);

        Ok(())
    }

    /// Readiness calibration rows (`bl_readiness_prediction`) merge through the
    /// bundle and drive identical calibration stats on both profiles.
    #[test]
    fn bundle_sync_predictions_match_calibration_stats() -> Result<()> {
        let (mut desktop, _d_dir) = isolated_col()?;
        let (mut mobile, _m_dir) = isolated_col()?;

        let prediction = SyncPredictionRow {
            id: 10,
            predicted_at_secs: TimestampSecs(1_700_000_000),
            projected_score: 72.0,
            projected_score_low: Some(67.0),
            projected_score_high: Some(77.0),
            memory_score: 65.0,
            performance_score: 70.0,
            coverage_ratio: 0.4,
            confidence_level: "medium".into(),
            model_version: READINESS_MODEL_VERSION.into(),
            outcome_score: Some(68.0),
            outcome_observed_at_secs: Some(TimestampSecs(1_700_100_000)),
            outcome_memory_score: Some(64.0),
            outcome_performance_score: Some(69.0),
            practice_correct: Some(40),
            practice_total: Some(50),
            usn: 1,
            mtime_secs: TimestampSecs(1_700_100_000),
        };

        gre_atlas_storage(&mut desktop)?.apply_sync_bundle(&SyncBundle {
            predictions: vec![prediction],
            ..Default::default()
        })?;

        sync_desktop_bundle_to_mobile(&mut desktop, &mut mobile)?;

        let desktop_stats = capture_sidecar_metrics(&mut desktop)?;
        let mobile_stats = capture_sidecar_metrics(&mut mobile)?;
        assert_eq!(desktop_stats.calibration_total_predictions, 1);
        assert_eq!(desktop_stats, mobile_stats);
        assert_eq!(mobile_stats.calibration_resolved, 1);

        Ok(())
    }

    /// Outcome resolution must bump prediction USN so resolved rows upload.
    #[test]
    fn outcome_resolution_marks_prediction_for_sync() -> Result<()> {
        let (mut col, _dir) = isolated_col()?;
        let storage = gre_atlas_storage(&mut col)?;

        let predicted_at = TimestampSecs(TimestampSecs::now().0 - 4 * 86_400);
        storage.apply_sync_bundle(&SyncBundle {
            predictions: vec![SyncPredictionRow {
                id: 5,
                predicted_at_secs: predicted_at,
                projected_score: 70.0,
                projected_score_low: Some(65.0),
                projected_score_high: Some(75.0),
                memory_score: 60.0,
                performance_score: 65.0,
                coverage_ratio: 0.3,
                confidence_level: "low".into(),
                model_version: READINESS_MODEL_VERSION.into(),
                outcome_score: None,
                outcome_observed_at_secs: None,
                outcome_memory_score: None,
                outcome_performance_score: None,
                practice_correct: None,
                practice_total: None,
                usn: 1,
                mtime_secs: predicted_at,
            }],
            ..Default::default()
        })?;
        storage.mark_synced_through(storage.sync_status()?.current_usn)?;

        let session = storage.create_session("practice")?;
        let q = storage.list_questions("", 1)?.pop().unwrap();
        for _ in 0..3 {
            storage.record_attempt(
                &q.id,
                &q.topic,
                q.difficulty,
                &q.correct_answer,
                true,
                900,
                None,
                Some(&session.id),
            )?;
        }

        storage.resolve_pending_outcomes(&OutcomeInputs {
            memory_score: 60.0,
            performance_score: 65.0,
            coverage_ratio: 0.3,
            practice_correct: 3,
            practice_total: 3,
        })?;

        let pending = storage.pull_sync_bundle(storage.last_pushed_usn()?, 5000)?;
        assert_eq!(pending.predictions.len(), 1);
        assert!(pending.predictions[0].outcome_score.is_some());

        Ok(())
    }

    /// When collections diverge, sidecar metrics still match but
    /// memory/coverage differ — documents iOS PARTIAL parity without
    /// collection sync.
    #[test]
    fn divergent_collections_sidecar_matches_collection_differs() -> Result<()> {
        use crate::config::BoolKey;

        let (mut desktop, _d_dir) = isolated_col()?;
        let (mut mobile, _m_dir) = isolated_col()?;

        desktop.set_config_bool(BoolKey::Fsrs, true, false)?;
        seed_practice_attempts(&mut desktop, MIN_PERFORMANCE_ATTEMPTS)?;
        sync_desktop_bundle_to_mobile(&mut desktop, &mut mobile)?;

        let desktop_sidecar = capture_sidecar_metrics(&mut desktop)?;
        let mobile_sidecar = capture_sidecar_metrics(&mut mobile)?;
        assert_eq!(desktop_sidecar, mobile_sidecar);

        let desktop_collection = capture_collection_metrics(&mut desktop)?;
        let mobile_collection = capture_collection_metrics(&mut mobile)?;
        assert_eq!(
            desktop_collection.memory_studied_cards,
            mobile_collection.memory_studied_cards
        );
        assert_eq!(
            desktop_collection.memory_coverage_ratio,
            mobile_collection.memory_coverage_ratio
        );

        Ok(())
    }
}
