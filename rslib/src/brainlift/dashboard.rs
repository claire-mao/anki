// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::DashboardCoverage;
use anki_proto::brainlift::DashboardState;
use anki_proto::brainlift::GetDashboardRequest;

use crate::brainlift::signals::attempt_row_to_proto;
use crate::brainlift::signals::mastery_map;
use crate::brainlift::signals::observed_tags_from_mastery;
use crate::brainlift::topic_insights::build_candidate;
use crate::brainlift::topic_insights::select_recommended_topics;
use crate::brainlift::topic_insights::select_weak_topics;
use crate::brainlift::GreCatalog;
use crate::collection::Collection;
use crate::error::Result;

const DEFAULT_RECENT_ACTIVITY_LIMIT: u32 = 10;
const DEFAULT_TOPIC_INSIGHT_LIMIT: u32 = 5;

impl Collection {
    pub fn brainlift_get_dashboard(
        &mut self,
        req: GetDashboardRequest,
    ) -> Result<DashboardState> {
        let recent_limit = if req.recent_activity_limit == 0 {
            DEFAULT_RECENT_ACTIVITY_LIMIT
        } else {
            req.recent_activity_limit
        };
        let insight_limit = if req.topic_insight_limit == 0 {
            DEFAULT_TOPIC_INSIGHT_LIMIT
        } else {
            req.topic_insight_limit
        };

        let signals = self.load_brainlift_signals(recent_limit)?;
        let mastery_by_id = mastery_map(&signals.mastery.topics);
        let observed = observed_tags_from_mastery(&signals.mastery.topics);
        let observed_refs: Vec<&str> = observed.iter().map(String::as_str).collect();

        let mut candidates = Vec::new();
        for leaf in GreCatalog::leaf_topics() {
            candidates.push(build_candidate(
                leaf.id,
                leaf.display_name,
                leaf.section.slug(),
                leaf.exam_weight,
                &mastery_by_id,
                &observed_refs,
                &signals.practice_by_topic,
            ));
        }

        let weak_topics = select_weak_topics(&candidates, insight_limit);
        let recommended_topics = select_recommended_topics(&candidates, &weak_topics, insight_limit);

        Ok(DashboardState {
            memory: Some(signals.memory),
            performance: Some(signals.performance),
            readiness: Some(signals.readiness),
            coverage: Some(DashboardCoverage {
                weighted_ratio: signals.coverage.weighted_ratio,
                unweighted_ratio: signals.coverage.unweighted_ratio,
                catalog_leaf_count: signals.coverage.catalog_leaf_count,
                covered_leaf_count: signals.coverage.covered_leaf_count,
            }),
            weak_topics,
            recommended_topics,
            recent_activity: signals
                .recent_attempts
                .iter()
                .map(attempt_row_to_proto)
                .collect(),
            computed_at_millis: signals.computed_at_millis,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anki_proto::stats::TopicMasteryEntry;
    use crate::brainlift::topic_insights::build_candidate;
    use crate::collection::CollectionBuilder;
    use crate::error::Result;
    use std::collections::HashMap;

    #[test]
    fn weak_topics_prioritize_uncovered_and_low_memory() {
        let candidates = vec![
            build_candidate(
                "gre::quant::algebra::linear",
                "Linear",
                "quant",
                0.10,
                &HashMap::from([(
                    "gre::quant::algebra::linear".to_string(),
                    TopicMasteryEntry {
                        topic_id: "gre::quant::algebra::linear".into(),
                        display_name: "Linear".into(),
                        total_cards: 10,
                        studied_cards: 10,
                        mastered_cards: 0,
                        avg_retrievability: 0.4,
                        avg_retrievability_low: 0.3,
                        avg_retrievability_high: 0.5,
                        total_reviews: 10,
                    },
                )]),
                &["gre::quant::algebra::linear"],
                &HashMap::new(),
            ),
            build_candidate(
                "gre::quant::geometry::circles",
                "Circles",
                "quant",
                0.07,
                &HashMap::new(),
                &[],
                &HashMap::new(),
            ),
        ];
        let weak = select_weak_topics(&candidates, 5);
        assert!(!weak.is_empty());
        assert!(
            weak.iter()
                .any(|topic| topic.topic_id == "gre::quant::geometry::circles")
        );
        assert!(
            weak.iter()
                .any(|topic| topic.topic_id == "gre::quant::algebra::linear")
        );
    }

    #[test]
    fn get_dashboard_returns_state_for_empty_collection() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let mut col = CollectionBuilder::new(dir.path().join("test.anki2")).build()?;
        let state = col.brainlift_get_dashboard(Default::default())?;
        let coverage = state.coverage.unwrap();
        assert!(coverage.catalog_leaf_count > 0);
        assert!(!state.recommended_topics.is_empty());
        assert!(!state.weak_topics.is_empty());
        assert!(state.computed_at_millis > 0);
        Ok(())
    }
}
