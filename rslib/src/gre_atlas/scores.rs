// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::GetScoresResponse;

use crate::collection::Collection;
use crate::error::Result;

impl Collection {
    pub fn gre_atlas_get_scores(&mut self) -> Result<GetScoresResponse> {
        let signals = self.load_gre_atlas_signals(10)?;
        Ok(GetScoresResponse {
            memory: Some(signals.memory),
            performance: Some(signals.performance),
            readiness: Some(signals.readiness),
            estimated_gre: Some(signals.estimated_gre),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::collection::Collection;
    use crate::collection::CollectionBuilder;
    use crate::config::BoolKey;
    use crate::gre_atlas::gre_atlas_storage;

    fn isolated_col() -> Result<(Collection, tempfile::TempDir)> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col = CollectionBuilder::new(dir.path().join("test.anki2")).build()?;
        Ok((col, dir))
    }

    #[test]
    fn get_scores_abstains_readiness_with_sparse_data() -> Result<()> {
        let (mut col, _dir) = isolated_col()?;
        let scores = col.gre_atlas_get_scores()?;
        let memory = scores.memory.unwrap();
        let performance = scores.performance.unwrap();
        let readiness = scores.readiness.unwrap();
        assert!(!memory.sufficient_data);
        assert!(!performance.sufficient_data);
        assert!(!readiness.sufficient_data);
        assert!(readiness.projected_score.is_none());
        assert!(!readiness.abstain_reason.is_empty());
        assert_eq!(memory.abstention_requirements.len(), 3);
        assert_eq!(performance.abstention_requirements.len(), 1);
        assert_eq!(readiness.abstention_requirements.len(), 4);
        for req in readiness
            .abstention_requirements
            .iter()
            .filter(|req| !req.met)
        {
            assert!(!req.next_step.is_empty());
        }
        Ok(())
    }

    #[test]
    fn get_scores_readiness_with_practice_attempts_only() -> Result<()> {
        let (mut col, _dir) = isolated_col()?;
        let storage = gre_atlas_storage(&mut col)?;
        let q = storage.list_questions("", 1)?.pop().unwrap();
        let session = storage.create_session("practice")?;
        for _ in 0..20 {
            storage.record_attempt(
                &q.id,
                &q.topic,
                q.difficulty,
                &q.correct_answer,
                true,
                1000,
                None,
                Some(&session.id),
            )?;
        }
        let scores = col.gre_atlas_get_scores()?;
        assert!(scores.performance.unwrap().sufficient_data);
        let readiness = scores.readiness.unwrap();
        assert!(!readiness.sufficient_data);
        assert!(readiness.projected_score.is_none());
        assert!(
            readiness.abstain_reason.contains("studied cards")
                || readiness.abstain_reason.contains("FSRS")
                || readiness.abstain_reason.contains("coverage")
        );
        assert!(readiness
            .abstention_requirements
            .iter()
            .any(|req| !req.met && !req.next_step.is_empty()));
        Ok(())
    }

    #[test]
    fn get_scores_includes_evidence_summary_when_abstaining() -> Result<()> {
        let (mut col, _dir) = isolated_col()?;
        col.set_config_bool(BoolKey::Fsrs, true, false)?;
        let storage = gre_atlas_storage(&mut col)?;
        let q = storage.list_questions("", 1)?.pop().unwrap();
        let session = storage.create_session("practice")?;
        storage.record_attempt(
            &q.id,
            &q.topic,
            q.difficulty,
            &q.correct_answer,
            true,
            1000,
            None,
            Some(&session.id),
        )?;
        let scores = col.gre_atlas_get_scores()?;
        let readiness = scores.readiness.unwrap();
        assert!(!readiness.evidence_summary.is_empty());
        assert!(readiness.last_updated_millis > 0);
        let estimated = scores.estimated_gre.unwrap();
        assert!(!estimated.abstain_reason.is_empty() || estimated.combined_score.is_some());
        Ok(())
    }
}
