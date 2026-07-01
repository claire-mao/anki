// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use anki_proto::brainlift::DashboardCoverage;
use anki_proto::brainlift::GetStudyPlanRequest;
use anki_proto::brainlift::StudyPlanRecommendation;
use anki_proto::brainlift::StudyPlanResponse;
use anki_proto::stats::TopicMasteryEntry;

use crate::brainlift::signals::mastery_map;
use crate::brainlift::signals::observed_tags_from_mastery;
use crate::brainlift::topic_insights::leaf_metrics;
use crate::brainlift::topic_insights::HIGH_EXAM_WEIGHT;
use crate::brainlift::topic_insights::LOW_MEMORY_RETRIEVABILITY;
use crate::brainlift::topic_insights::LOW_PRACTICE_ACCURACY;
use crate::brainlift::topic_insights::MIN_PRACTICE_ATTEMPTS_FOR_WEAK;
use crate::brainlift::GreCatalog;
use crate::collection::Collection;
use crate::error::Result;

const DEFAULT_STUDY_PLAN_LIMIT: u32 = 10;

const FACTOR_COVERAGE_GAP: &str = "coverage_gap";
const FACTOR_LOW_MASTERY: &str = "low_mastery";
const FACTOR_LOW_PERFORMANCE: &str = "low_performance";
const FACTOR_NO_PRACTICE: &str = "no_practice";
const FACTOR_HIGH_IMPORTANCE: &str = "high_importance";

impl Collection {
    pub fn brainlift_get_study_plan(
        &mut self,
        req: GetStudyPlanRequest,
    ) -> Result<StudyPlanResponse> {
        let limit = if req.limit == 0 {
            DEFAULT_STUDY_PLAN_LIMIT
        } else {
            req.limit
        };

        let signals = self.load_brainlift_signals(1)?;
        let mastery_by_id = mastery_map(&signals.mastery.topics);
        let observed = observed_tags_from_mastery(&signals.mastery.topics);
        let observed_refs: Vec<&str> = observed.iter().map(String::as_str).collect();

        let mut recommendations = Vec::new();
        for leaf in GreCatalog::leaf_topics() {
            if let Some(recommendation) = score_study_recommendation(
                leaf.id,
                leaf.display_name,
                leaf.section.slug(),
                leaf.exam_weight,
                &mastery_by_id,
                &observed_refs,
                &signals.practice_by_topic,
            ) {
                recommendations.push(recommendation);
            }
        }

        recommendations.sort_by(|a, b| {
            b.priority_score
                .partial_cmp(&a.priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    b.exam_weight
                        .partial_cmp(&a.exam_weight)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        recommendations.truncate(limit as usize);

        let coverage = DashboardCoverage {
            weighted_ratio: signals.coverage.weighted_ratio,
            unweighted_ratio: signals.coverage.unweighted_ratio,
            catalog_leaf_count: signals.coverage.catalog_leaf_count,
            covered_leaf_count: signals.coverage.covered_leaf_count,
        };

        let summary = build_study_plan_summary(&coverage, recommendations.len());

        Ok(StudyPlanResponse {
            coverage: Some(coverage),
            recommendations,
            computed_at_millis: signals.computed_at_millis,
            summary,
        })
    }
}

fn score_study_recommendation(
    topic_id: &str,
    display_name: &str,
    section: &str,
    exam_weight: f32,
    mastery_by_id: &HashMap<String, TopicMasteryEntry>,
    observed_tags: &[&str],
    practice_by_topic: &HashMap<String, (u32, u32)>,
) -> Option<StudyPlanRecommendation> {
    let metrics = leaf_metrics(topic_id, mastery_by_id, observed_tags, practice_by_topic);

    let mut priority_score = 0.0f32;
    let mut factors = Vec::new();
    let mut explanation_parts = Vec::new();

    if !metrics.covered {
        priority_score += exam_weight * 1.5;
        factors.push(FACTOR_COVERAGE_GAP.to_string());
        explanation_parts.push(format!(
            "Not covered in your GRE deck ({:.0}% exam weight)",
            exam_weight * 100.0
        ));
    }

    if let Some(retrievability) = metrics.memory_retrievability {
        if retrievability < LOW_MEMORY_RETRIEVABILITY {
            let gap = 1.0 - retrievability;
            priority_score += exam_weight * gap;
            factors.push(FACTOR_LOW_MASTERY.to_string());
            explanation_parts.push(format!(
                "Low FSRS retrievability ({:.0}%)",
                retrievability * 100.0
            ));
        }
    }

    if let (Some(correct), Some(total)) = (metrics.practice_correct, metrics.practice_total) {
        if total >= MIN_PRACTICE_ATTEMPTS_FOR_WEAK {
            let accuracy = correct as f32 / total as f32;
            if accuracy < LOW_PRACTICE_ACCURACY {
                priority_score += exam_weight * (1.0 - accuracy);
                factors.push(FACTOR_LOW_PERFORMANCE.to_string());
                explanation_parts.push(format!("Low practice accuracy ({correct}/{total} correct)"));
            }
        }
    } else if metrics.covered {
        priority_score += exam_weight * 0.25;
        factors.push(FACTOR_NO_PRACTICE.to_string());
        explanation_parts.push("No GRE practice attempts for this topic yet".into());
    }

    if exam_weight >= HIGH_EXAM_WEIGHT && !explanation_parts.is_empty() {
        factors.push(FACTOR_HIGH_IMPORTANCE.to_string());
        explanation_parts.push(format!(
            "High exam importance ({:.0}% of its section)",
            exam_weight * 100.0
        ));
    }

    if priority_score <= 0.0 || factors.is_empty() {
        return None;
    }

    Some(StudyPlanRecommendation {
        topic_id: topic_id.to_string(),
        display_name: display_name.to_string(),
        section: section.to_string(),
        exam_weight,
        memory_score: metrics.memory_score,
        practice_accuracy: metrics.practice_accuracy,
        studied_cards: metrics.studied_cards,
        covered: metrics.covered,
        priority_score,
        explanation: explanation_parts.join(" · "),
        factors,
    })
}

fn build_study_plan_summary(coverage: &DashboardCoverage, recommendation_count: usize) -> String {
    if recommendation_count == 0 {
        return format!(
            "All {} catalog topics meet current memory, practice, and coverage targets.",
            coverage.catalog_leaf_count
        );
    }

    format!(
        "{} of {} catalog topics covered ({:.0}% weighted coverage). {} ranked topics to study next.",
        coverage.covered_leaf_count,
        coverage.catalog_leaf_count,
        coverage.weighted_ratio * 100.0,
        recommendation_count
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::brainlift::topic_insights::build_candidate;
    use crate::collection::CollectionBuilder;
    use crate::error::Result;

    #[test]
    fn study_plan_ranks_uncovered_high_yield_first() {
        let mastery_by_id = HashMap::new();
        let practice_by_topic = HashMap::new();
        let observed: Vec<&str> = vec![];

        let data_interp = score_study_recommendation(
            "gre::quant::data_interpretation",
            "Data interpretation",
            "quant",
            0.15,
            &mastery_by_id,
            &observed,
            &practice_by_topic,
        )
        .unwrap();
        assert!(data_interp.factors.contains(&FACTOR_COVERAGE_GAP.to_string()));
        assert!(data_interp.explanation.contains("15%"));

        let plan = [
            score_study_recommendation(
                "gre::quant::geometry::circles",
                "Circles",
                "quant",
                0.07,
                &mastery_by_id,
                &observed,
                &practice_by_topic,
            )
            .unwrap(),
            data_interp,
        ];
        assert!(plan[1].priority_score > plan[0].priority_score);
    }

    #[test]
    fn study_plan_includes_low_memory_and_performance() {
        let mastery_by_id = HashMap::from([(
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
        )]);
        let observed = vec!["gre::quant::algebra::linear"];
        let practice_by_topic = HashMap::from([(
            "gre::quant::algebra::linear".to_string(),
            (1_u32, 4_u32),
        )]);

        let recommendation = score_study_recommendation(
            "gre::quant::algebra::linear",
            "Linear equations",
            "quant",
            0.10,
            &mastery_by_id,
            &observed,
            &practice_by_topic,
        )
        .unwrap();

        assert!(recommendation.factors.contains(&FACTOR_LOW_MASTERY.to_string()));
        assert!(recommendation.factors.contains(&FACTOR_LOW_PERFORMANCE.to_string()));
        assert!(recommendation.explanation.contains("FSRS"));
        assert!(recommendation.explanation.contains("practice accuracy"));
    }

    #[test]
    fn study_plan_skips_topics_on_track() {
        let mastery_by_id = HashMap::from([(
            "gre::quant::algebra::linear".to_string(),
            TopicMasteryEntry {
                topic_id: "gre::quant::algebra::linear".into(),
                display_name: "Linear".into(),
                total_cards: 10,
                studied_cards: 10,
                mastered_cards: 10,
                avg_retrievability: 0.92,
                avg_retrievability_low: 0.88,
                avg_retrievability_high: 0.95,
                total_reviews: 20,
            },
        )]);
        let observed = vec!["gre::quant::algebra::linear"];
        let practice_by_topic = HashMap::from([(
            "gre::quant::algebra::linear".to_string(),
            (8_u32, 10_u32),
        )]);

        assert!(score_study_recommendation(
            "gre::quant::algebra::linear",
            "Linear equations",
            "quant",
            0.10,
            &mastery_by_id,
            &observed,
            &practice_by_topic,
        )
        .is_none());
    }

    #[test]
    fn get_study_plan_returns_ranked_recommendations() -> Result<()> {
        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let mut col = CollectionBuilder::new(dir.path().join("test.anki2")).build()?;
        let plan = col.brainlift_get_study_plan(Default::default())?;
        let coverage = plan.coverage.unwrap();
        assert!(coverage.catalog_leaf_count > 0);
        assert!(!plan.recommendations.is_empty());
        assert!(!plan.summary.is_empty());
        assert!(plan.computed_at_millis > 0);
        for topic in &plan.recommendations {
            assert!(!topic.explanation.is_empty());
            assert!(!topic.factors.is_empty());
            assert!(topic.priority_score > 0.0);
        }
        Ok(())
    }

    #[test]
    fn dashboard_and_study_plan_share_candidate_metrics() {
        let mastery_by_id = HashMap::new();
        let observed: Vec<&str> = vec![];
        let practice_by_topic = HashMap::new();

        let candidate = build_candidate(
            "gre::quant::geometry::circles",
            "Circles",
            "quant",
            0.07,
            &mastery_by_id,
            &observed,
            &practice_by_topic,
        );
        let recommendation = score_study_recommendation(
            "gre::quant::geometry::circles",
            "Circles",
            "quant",
            0.07,
            &mastery_by_id,
            &observed,
            &practice_by_topic,
        )
        .unwrap();

        assert_eq!(candidate.covered, recommendation.covered);
        assert_eq!(candidate.memory_score, recommendation.memory_score);
    }
}
