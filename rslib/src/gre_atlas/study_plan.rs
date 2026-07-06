// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use anki_proto::brainlift::DailyStudyPlan;
use anki_proto::brainlift::DashboardCoverage;
use anki_proto::brainlift::GetStudyPlanRequest;
use anki_proto::brainlift::StudyPlanDailyTask;
use anki_proto::brainlift::StudyPlanRecommendation;
use anki_proto::brainlift::StudyPlanResponse;
use anki_proto::stats::TopicMasteryEntry;

use super::GRE_DECK_NAME;
use crate::collection::Collection;
use crate::error::Result;
use crate::gre_atlas::abstention::MIN_PERFORMANCE_ATTEMPTS;
use crate::gre_atlas::coverage_report::build_dashboard_coverage;
use crate::gre_atlas::gre_deck_id;
use crate::gre_atlas::signals::mastery_map;
use crate::gre_atlas::signals::observed_tags_from_mastery;
use crate::gre_atlas::signals::GreAtlasSignals;
use crate::gre_atlas::topic_insights::build_candidate;
use crate::gre_atlas::topic_insights::leaf_metrics;
use crate::gre_atlas::topic_insights::HIGH_EXAM_WEIGHT;
use crate::gre_atlas::topic_insights::LOW_MEMORY_RETRIEVABILITY;
use crate::gre_atlas::topic_insights::LOW_PRACTICE_ACCURACY;
use crate::gre_atlas::topic_insights::MIN_PRACTICE_ATTEMPTS_FOR_WEAK;
use crate::gre_atlas::topic_flashcard_release::topic_flashcard_schedule_for_topic;
use crate::gre_atlas::GreCatalog;

const DEFAULT_STUDY_PLAN_LIMIT: u32 = 10;
const DAILY_REVIEW_TARGET: u32 = 30;
const DAILY_REVIEW_BOOTSTRAP: u32 = 40;
const DAILY_REVIEW_CAP: u32 = 60;
const DAILY_PRACTICE_TARGET: u32 = 5;
const DAILY_PRACTICE_BOOTSTRAP: u32 = 10;
const FOCUS_ADD_CARDS_TARGET: u32 = 5;
const FOCUS_REVIEW_CARDS_TARGET: u32 = 20;
const FOCUS_PRACTICE_TARGET: u32 = 3;
const FOCUS_PRACTICE_UNTRIED: u32 = 2;

const TASK_REVIEW_CARDS: &str = "review_cards";
const TASK_PRACTICE_QUESTIONS: &str = "practice_questions";
const TASK_FOCUS_TOPIC: &str = "focus_topic";

const FACTOR_COVERAGE_GAP: &str = "coverage_gap";
const FACTOR_LOW_MASTERY: &str = "low_mastery";
const FACTOR_LOW_PERFORMANCE: &str = "low_performance";
const FACTOR_NO_PRACTICE: &str = "no_practice";
const FACTOR_HIGH_IMPORTANCE: &str = "high_importance";

pub(crate) const DAILY_FOCUS_TOPIC_COUNT: u32 = 3;

pub(crate) fn all_study_recommendations(
    mastery_by_id: &HashMap<String, TopicMasteryEntry>,
    observed_tags: &[&str],
    practice_by_topic: &HashMap<String, (u32, u32)>,
) -> Vec<StudyPlanRecommendation> {
    let mut recommendations = Vec::new();
    for leaf in GreCatalog::leaf_topics() {
        if let Some(recommendation) = score_study_recommendation(
            leaf.id,
            leaf.display_name,
            leaf.section.slug(),
            leaf.exam_weight,
            mastery_by_id,
            observed_tags,
            practice_by_topic,
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
            .then_with(|| a.topic_id.cmp(&b.topic_id))
    });
    recommendations
}

impl Collection {
    pub fn gre_atlas_get_study_plan(
        &mut self,
        req: GetStudyPlanRequest,
    ) -> Result<StudyPlanResponse> {
        let limit = if req.limit == 0 {
            DEFAULT_STUDY_PLAN_LIMIT
        } else {
            req.limit
        };

        let signals = self.load_gre_atlas_signals(1)?;
        let mastery_by_id = mastery_map(&signals.mastery.topics);
        let observed = observed_tags_from_mastery(&signals.mastery.topics);
        let observed_refs: Vec<&str> = observed.iter().map(String::as_str).collect();

        let mut recommendations =
            all_study_recommendations(&mastery_by_id, &observed_refs, &signals.practice_by_topic);

        let (new_due, learn_due, review_due) = gre_deck_due_counts(self)?;
        let daily_plan = build_daily_study_plan(
            self,
            &signals,
            &recommendations,
            new_due,
            learn_due,
            review_due,
        );

        recommendations.truncate(limit as usize);

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

        let coverage = build_dashboard_coverage(&signals.coverage, &candidates);

        let summary = build_study_plan_summary(&coverage, recommendations.len());

        Ok(StudyPlanResponse {
            coverage: Some(coverage),
            recommendations,
            computed_at_millis: signals.computed_at_millis,
            summary,
            daily_plan: Some(daily_plan),
        })
    }
}

fn gre_deck_due_counts(col: &mut Collection) -> Result<(u32, u32, u32)> {
    let deck_id = gre_deck_id(col)?;
    let mut new_count = 0;
    let mut learn_count = 0;
    let mut review_count = 0;
    if let Some(did) = deck_id {
        let timing = col.timing_today()?;
        let learn_cutoff = timing.now.0 as u32 + col.learn_ahead_secs();
        let counts_map = col.due_counts(timing.days_elapsed, learn_cutoff)?;
        if let Some(counts) = counts_map.get(&did) {
            new_count = counts.new;
            learn_count = counts.learning;
            review_count = counts.review;
        }
    }
    Ok((new_count, learn_count, review_count))
}

fn build_daily_study_plan(
    col: &mut Collection,
    signals: &GreAtlasSignals,
    recommendations: &[StudyPlanRecommendation],
    new_due: u32,
    learn_due: u32,
    review_due: u32,
) -> DailyStudyPlan {
    let due_total = new_due + learn_due + review_due;
    let review_target = daily_review_target(&signals.memory, due_total);
    let practice_target = daily_practice_target(&signals.performance, &signals.readiness);

    let mut tasks = Vec::new();

    if review_target > 0 {
        tasks.push(StudyPlanDailyTask {
            id: TASK_REVIEW_CARDS.into(),
            title: "Review GRE flashcards".into(),
            detail: if !signals.memory.sufficient_data {
                format!(
                    "{review_target} cards due in {GRE_DECK_NAME} ({new_due} new, {learn_due} learning, {review_due} review) to build memory evidence"
                )
            } else {
                format!(
                    "Clear up to {review_target} due cards ({new_due} new, {learn_due} learning, {review_due} review)"
                )
            },
            target_count: review_target,
            topic_id: None,
            topic_display_name: None,
            flashcard_schedule_hint: None,
            flashcards_due_now: None,
            flashcards_next_due_in_days: None,
        });
    } else {
        tasks.push(StudyPlanDailyTask {
            id: TASK_REVIEW_CARDS.into(),
            title: "Review GRE flashcards".into(),
            detail: format!(
                "No cards due in {GRE_DECK_NAME} right now. Add cards for focus topics below."
            ),
            target_count: 0,
            topic_id: None,
            topic_display_name: None,
            flashcard_schedule_hint: None,
            flashcards_due_now: None,
            flashcards_next_due_in_days: None,
        });
    }

    tasks.push(StudyPlanDailyTask {
        id: TASK_PRACTICE_QUESTIONS.into(),
        title: "GRE practice questions".into(),
        detail: if !signals.performance.sufficient_data {
            format!(
                "Answer {practice_target} questions today ({}/{MIN_PERFORMANCE_ATTEMPTS} attempts logged toward performance score)",
                signals.performance.attempt_count
            )
        } else {
            format!("Answer {practice_target} exam-style questions, prioritizing focus topics below")
        },
        target_count: practice_target,
        topic_id: None,
        topic_display_name: None,
        flashcard_schedule_hint: None,
        flashcards_due_now: None,
        flashcards_next_due_in_days: None,
    });

    for recommendation in recommendations
        .iter()
        .take(DAILY_FOCUS_TOPIC_COUNT as usize)
    {
        let mut task = focus_topic_task(recommendation);
        if let Some(topic_id) = &task.topic_id {
            if let Ok(schedule) = topic_flashcard_schedule_for_topic(col, topic_id) {
                task.flashcard_schedule_hint = Some(schedule.hint());
                task.flashcards_due_now = Some(schedule.due_now);
                task.flashcards_next_due_in_days = schedule
                    .next_batch_in_days
                    .or(schedule.next_due_in_days);
            }
        }
        tasks.push(task);
    }

    let headline = build_daily_headline(
        signals,
        review_target,
        practice_target,
        recommendations.len(),
    );
    let rationale = build_daily_rationale(signals, due_total);

    DailyStudyPlan {
        headline,
        tasks,
        rationale,
    }
}

fn daily_review_target(memory: &anki_proto::brainlift::MemoryScore, due_total: u32) -> u32 {
    if due_total == 0 {
        return 0;
    }
    let cap = if memory.sufficient_data {
        DAILY_REVIEW_TARGET
    } else {
        DAILY_REVIEW_BOOTSTRAP
    };
    due_total.min(cap).min(DAILY_REVIEW_CAP)
}

fn daily_practice_target(
    performance: &anki_proto::brainlift::PerformanceScore,
    readiness: &anki_proto::brainlift::ReadinessScore,
) -> u32 {
    if !performance.sufficient_data {
        return DAILY_PRACTICE_BOOTSTRAP;
    }
    if readiness.sufficient_data && readiness.projected_score.is_some_and(|score| score < 70.0) {
        return DAILY_PRACTICE_TARGET + DAILY_PRACTICE_BOOTSTRAP;
    }
    DAILY_PRACTICE_TARGET
}

fn focus_topic_task(recommendation: &StudyPlanRecommendation) -> StudyPlanDailyTask {
    if recommendation
        .factors
        .contains(&FACTOR_COVERAGE_GAP.to_string())
    {
        return StudyPlanDailyTask {
            id: TASK_FOCUS_TOPIC.into(),
            title: format!("Cover {}", recommendation.display_name),
            detail: format!(
                "Answer {FOCUS_ADD_CARDS_TARGET} practice questions on {} ({:.0}% exam weight)",
                recommendation.topic_id,
                recommendation.exam_weight * 100.0
            ),
            target_count: FOCUS_ADD_CARDS_TARGET,
            topic_id: Some(recommendation.topic_id.clone()),
            topic_display_name: Some(recommendation.display_name.clone()),
            flashcard_schedule_hint: None,
            flashcards_due_now: None,
            flashcards_next_due_in_days: None,
        };
    }

    if recommendation
        .factors
        .contains(&FACTOR_LOW_MASTERY.to_string())
    {
        let review_target = recommendation
            .studied_cards
            .clamp(10, FOCUS_REVIEW_CARDS_TARGET);
        return StudyPlanDailyTask {
            id: TASK_FOCUS_TOPIC.into(),
            title: format!("Strengthen {}", recommendation.display_name),
            detail: recommendation.explanation.clone(),
            target_count: review_target,
            topic_id: Some(recommendation.topic_id.clone()),
            topic_display_name: Some(recommendation.display_name.clone()),
            flashcard_schedule_hint: None,
            flashcards_due_now: None,
            flashcards_next_due_in_days: None,
        };
    }

    let practice_target = if recommendation
        .factors
        .contains(&FACTOR_LOW_PERFORMANCE.to_string())
    {
        FOCUS_PRACTICE_TARGET
    } else {
        FOCUS_PRACTICE_UNTRIED
    };
    StudyPlanDailyTask {
        id: TASK_FOCUS_TOPIC.into(),
        title: format!("Practice {}", recommendation.display_name),
        detail: recommendation.explanation.clone(),
        target_count: practice_target,
        topic_id: Some(recommendation.topic_id.clone()),
        topic_display_name: Some(recommendation.display_name.clone()),
        flashcard_schedule_hint: None,
        flashcards_due_now: None,
        flashcards_next_due_in_days: None,
    }
}

fn build_daily_headline(
    signals: &GreAtlasSignals,
    review_target: u32,
    practice_target: u32,
    recommendation_count: usize,
) -> String {
    if recommendation_count == 0 {
        return "Maintain your streak: keep reviewing and practicing across the GRE catalog."
            .into();
    }

    if !signals.readiness.sufficient_data {
        return format!(
            "Build readiness evidence today: {review_target} reviews and {practice_target} practice questions, plus {DAILY_FOCUS_TOPIC_COUNT} focus topics"
        );
    }

    format!(
        "Today's plan: {review_target} reviews, {practice_target} practice questions, focusing on top {DAILY_FOCUS_TOPIC_COUNT} ranked topics"
    )
}

fn build_daily_rationale(signals: &GreAtlasSignals, due_total: u32) -> String {
    let mut parts = Vec::new();

    if signals.readiness.sufficient_data {
        parts.push(format!(
            "Readiness {:.0}% ({})",
            signals.readiness.projected_score.unwrap_or_default(),
            signals.readiness.confidence_level
        ));
    } else if !signals.readiness.abstain_reason.is_empty() {
        parts.push(format!(
            "Readiness gated: {}",
            signals.readiness.abstain_reason
        ));
    }

    parts.push(signals.memory.detail.clone());
    parts.push(signals.performance.detail.clone());
    parts.push(format!(
        "{due_total} cards due in {GRE_DECK_NAME} · {:.0}% weighted catalog coverage",
        signals.coverage.weighted_ratio * 100.0
    ));

    parts.join(" · ")
}

pub(crate) fn score_study_recommendation(
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
                explanation_parts
                    .push(format!("Low practice accuracy ({correct}/{total} correct)"));
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
    use crate::collection::CollectionBuilder;
    use crate::error::Result;
    use crate::gre_atlas::topic_insights::build_candidate;

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
        assert!(data_interp
            .factors
            .contains(&FACTOR_COVERAGE_GAP.to_string()));
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
                ..Default::default()
            },
        )]);
        let observed = vec!["gre::quant::algebra::linear"];
        let practice_by_topic =
            HashMap::from([("gre::quant::algebra::linear".to_string(), (1_u32, 4_u32))]);

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

        assert!(recommendation
            .factors
            .contains(&FACTOR_LOW_MASTERY.to_string()));
        assert!(recommendation
            .factors
            .contains(&FACTOR_LOW_PERFORMANCE.to_string()));
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
                ..Default::default()
            },
        )]);
        let observed = vec!["gre::quant::algebra::linear"];
        let practice_by_topic =
            HashMap::from([("gre::quant::algebra::linear".to_string(), (8_u32, 10_u32))]);

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
        let plan = col.gre_atlas_get_study_plan(Default::default())?;
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
        let daily = plan.daily_plan.unwrap();
        assert!(!daily.headline.is_empty());
        assert!(!daily.rationale.is_empty());
        assert!(daily.tasks.len() >= 2);
        assert!(daily.tasks.iter().any(|task| task.id == TASK_REVIEW_CARDS));
        assert!(daily
            .tasks
            .iter()
            .any(|task| task.id == TASK_PRACTICE_QUESTIONS));
        Ok(())
    }

    #[test]
    fn daily_review_target_caps_due_cards() {
        let memory = anki_proto::brainlift::MemoryScore {
            sufficient_data: true,
            ..Default::default()
        };
        assert_eq!(daily_review_target(&memory, 0), 0);
        assert_eq!(daily_review_target(&memory, 12), 12);
        assert_eq!(daily_review_target(&memory, 100), DAILY_REVIEW_TARGET);
    }

    #[test]
    fn daily_practice_target_bootstraps_before_performance_unlocks() {
        let sparse = anki_proto::brainlift::PerformanceScore {
            sufficient_data: false,
            attempt_count: 4,
            ..Default::default()
        };
        let readiness = anki_proto::brainlift::ReadinessScore::default();
        assert_eq!(
            daily_practice_target(&sparse, &readiness),
            DAILY_PRACTICE_BOOTSTRAP
        );

        let healthy = anki_proto::brainlift::PerformanceScore {
            sufficient_data: true,
            attempt_count: 25,
            ..Default::default()
        };
        assert_eq!(
            daily_practice_target(&healthy, &readiness),
            DAILY_PRACTICE_TARGET
        );
    }

    #[test]
    fn focus_topic_task_maps_factors_to_actions() {
        let coverage = StudyPlanRecommendation {
            topic_id: "gre::quant::data_interpretation".into(),
            display_name: "Data interpretation".into(),
            section: "quant".into(),
            exam_weight: 0.15,
            memory_score: None,
            practice_accuracy: None,
            studied_cards: 0,
            covered: false,
            priority_score: 0.2,
            explanation: "Not covered".into(),
            factors: vec![FACTOR_COVERAGE_GAP.to_string()],
        };
        let task = focus_topic_task(&coverage);
        assert_eq!(task.target_count, FOCUS_ADD_CARDS_TARGET);
        assert_eq!(task.topic_id.as_deref(), Some(coverage.topic_id.as_str()));

        let weak_memory = StudyPlanRecommendation {
            factors: vec![FACTOR_LOW_MASTERY.to_string()],
            studied_cards: 25,
            topic_id: "gre::quant::algebra::linear".into(),
            display_name: "Linear equations".into(),
            section: "quant".into(),
            exam_weight: 0.10,
            memory_score: Some(40.0),
            practice_accuracy: None,
            covered: true,
            priority_score: 0.1,
            explanation: "Low FSRS retrievability".into(),
        };
        let task = focus_topic_task(&weak_memory);
        assert_eq!(task.target_count, FOCUS_REVIEW_CARDS_TARGET);
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
