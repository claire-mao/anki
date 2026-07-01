// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::GetScoresResponse;
use anki_proto::brainlift::ScoreSnapshot;
use anki_proto::stats::TopicMasteryRequest;

use crate::brainlift::brainlift_storage;
use crate::brainlift::TOPIC_TAG_PREFIX;
use crate::collection::Collection;
use crate::error::Result;

impl Collection {
    pub fn brainlift_get_scores(&mut self) -> Result<GetScoresResponse> {
        let memory = self.brainlift_memory_score()?;
        let performance = self.brainlift_performance_score()?;
        let readiness = ScoreSnapshot {
            sufficient_data: false,
            detail: "Readiness model coming in Phase 2.".into(),
            value: None,
        };
        Ok(GetScoresResponse {
            memory: Some(memory),
            performance: Some(performance),
            readiness: Some(readiness),
        })
    }

    fn brainlift_memory_score(&mut self) -> Result<ScoreSnapshot> {
        let resp = self.compute_topic_mastery(TopicMasteryRequest {
            search: format!("deck:\"{}\"", super::GRE_DECK_NAME),
            topic_tag_prefix: TOPIC_TAG_PREFIX.into(),
            mastery_threshold: None,
            min_reviews: 1,
        })?;
        let summary = resp.summary.unwrap_or_default();
        if summary.sufficient_data {
            Ok(ScoreSnapshot {
                value: Some(summary.overall_avg_retrievability * 100.0),
                sufficient_data: true,
                detail: format!(
                    "{} studied cards · {}% topic coverage",
                    summary.studied_cards,
                    (summary.coverage_ratio * 100.0).round() as u32
                ),
            })
        } else {
            Ok(ScoreSnapshot {
                value: None,
                sufficient_data: false,
                detail: summary.abstain_reason,
            })
        }
    }

    fn brainlift_performance_score(&mut self) -> Result<ScoreSnapshot> {
        let storage = brainlift_storage(self)?;
        let (correct, total) = storage.performance_summary()?;
        if total == 0 {
            return Ok(ScoreSnapshot {
                value: None,
                sufficient_data: false,
                detail: "No GRE practice attempts yet.".into(),
            });
        }
        Ok(ScoreSnapshot {
            value: Some(correct as f32 / total as f32 * 100.0),
            sufficient_data: true,
            detail: format!("{correct}/{total} practice questions correct"),
        })
    }
}
