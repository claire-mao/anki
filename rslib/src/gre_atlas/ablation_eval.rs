// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;

use anki_proto::brainlift::StudyPlanRecommendation;
use anki_proto::stats::TopicMasteryEntry;
use anki_proto::stats::TopicMasteryRequest;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::Serialize;

use crate::collection::Collection;
use crate::error::Result;
use crate::gre_atlas::compute_coverage;
use crate::gre_atlas::gre_atlas_storage;
use crate::gre_atlas::gre_deck_search;
use crate::gre_atlas::readiness::compute_memory_score;
use crate::gre_atlas::readiness::compute_performance_score;
use crate::gre_atlas::readiness::compute_readiness_score;
use crate::gre_atlas::readiness::MemoryInputs;
use crate::gre_atlas::readiness::PerformanceInputs;
use crate::gre_atlas::signals::mastery_map;
use crate::gre_atlas::signals::observed_tags_from_mastery;
use crate::gre_atlas::study_plan::all_study_recommendations;
use crate::gre_atlas::study_plan::DAILY_FOCUS_TOPIC_COUNT;
use crate::gre_atlas::topic_insights::LOW_MEMORY_RETRIEVABILITY;
use crate::gre_atlas::topic_insights::LOW_PRACTICE_ACCURACY;
use crate::gre_atlas::TOPIC_TAG_PREFIX;
use crate::timestamp::TimestampMillis;

const RANDOM_POLICY_SEED: u64 = 42;
const MEMORY_FOCUS_BUMP_FRACTION: f32 = 0.25;
const FOCUS_PRACTICE_TARGET: u32 = 3;

const POLICY_GRE_ATLAS: &str = "gre_atlas_priority";
const POLICY_RANDOM: &str = "random_topic_order";
const POLICY_VANILLA: &str = "vanilla_anki_order";

const FACTOR_COVERAGE_GAP: &str = "coverage_gap";
const FACTOR_LOW_MASTERY: &str = "low_mastery";
const FACTOR_LOW_PERFORMANCE: &str = "low_performance";
const FACTOR_NO_PRACTICE: &str = "no_practice";

pub const ABLATION_MODEL_VERSION: &str = "gre_atlas-ablation-v1";

#[derive(Debug, Clone, Serialize)]
pub struct AblationEval {
    pub model_version: String,
    pub methodology: AblationMethodology,
    pub focus_topic_count: u32,
    pub collection: AblationScenarioResult,
    pub synthetic_reference: AblationScenarioResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct AblationMethodology {
    pub description: String,
    pub policies: Vec<AblationPolicyDoc>,
    pub metrics: Vec<AblationMetricDoc>,
    pub simulation_model: String,
    pub synthetic_label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AblationPolicyDoc {
    pub id: String,
    pub label: String,
    pub ordering_rule: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AblationMetricDoc {
    pub id: String,
    pub label: String,
    pub definition: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AblationScenarioResult {
    pub label: String,
    pub data_source: String,
    pub sufficient_data: bool,
    pub assessment: String,
    pub eligible_recommendation_count: u32,
    pub baseline: AblationBaseline,
    pub policies: Vec<AblationPolicyResult>,
    pub winners: AblationWinners,
}

#[derive(Debug, Clone, Serialize)]
pub struct AblationBaseline {
    pub coverage_weighted_ratio: f32,
    pub readiness_projected: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AblationPolicyResult {
    pub policy: String,
    pub policy_label: String,
    pub selected_topic_ids: Vec<String>,
    pub expected_learning_gain: f32,
    pub topic_coverage_gain: f32,
    pub readiness_improvement: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AblationWinners {
    pub expected_learning_gain: String,
    pub topic_coverage_gain: String,
    pub readiness_improvement: String,
}

struct AblationContext {
    recommendations: Vec<StudyPlanRecommendation>,
    observed_tags: Vec<String>,
    overall_avg_retrievability: f32,
    fsrs_enabled: bool,
    studied_cards: u32,
    mastery_topics: Vec<TopicMasteryEntry>,
    perf_correct: u32,
    perf_total: u32,
}

struct SimulationState {
    observed_tags: Vec<String>,
    overall_retrievability: f32,
    perf_correct: u32,
    perf_total: u32,
    mastery_topics: Vec<TopicMasteryEntry>,
    fsrs_enabled: bool,
    studied_cards: u32,
}

pub fn compute_ablation_eval(col: &mut Collection) -> Result<AblationEval> {
    let collection = load_collection_ablation(col).unwrap_or_else(|_| {
        insufficient_scenario(
            "Collection",
            "collection",
            "failed to load collection context",
        )
    });
    let synthetic_reference = evaluate_scenario(
        synthetic_reference_context(),
        "Synthetic reference scenario",
        "synthetic_reference",
    );

    Ok(AblationEval {
        model_version: ABLATION_MODEL_VERSION.into(),
        methodology: ablation_methodology(),
        focus_topic_count: DAILY_FOCUS_TOPIC_COUNT,
        collection,
        synthetic_reference,
    })
}

pub fn render_ablation_markdown(ablation: &AblationEval) -> String {
    let mut out = String::new();
    out.push_str("## Topic-priority ablation\n\n");
    out.push_str(&format!("- Model version: `{}`\n", ablation.model_version));
    out.push_str(&format!(
        "- Focus topics simulated per policy: {}\n",
        ablation.focus_topic_count
    ));
    out.push_str(&format!("- {}\n\n", ablation.methodology.description));

    out.push_str("### Policies\n\n");
    for policy in &ablation.methodology.policies {
        out.push_str(&format!(
            "- **{}** (`{}`): {}\n",
            policy.label, policy.id, policy.ordering_rule
        ));
    }
    out.push('\n');

    out.push_str("### Metrics\n\n");
    for metric in &ablation.methodology.metrics {
        out.push_str(&format!(
            "- **{}** (`{}`): {}\n",
            metric.label, metric.id, metric.definition
        ));
    }
    out.push('\n');

    out.push_str(&format!(
        "### Simulation model\n\n{}\n\n",
        ablation.methodology.simulation_model
    ));

    render_scenario_markdown(&mut out, &ablation.collection);
    render_scenario_markdown(&mut out, &ablation.synthetic_reference);

    out
}

fn render_scenario_markdown(out: &mut String, scenario: &AblationScenarioResult) {
    out.push_str(&format!(
        "### {} ({})\n\n",
        scenario.label, scenario.data_source
    ));
    out.push_str(&format!(
        "- Sufficient data: {}\n",
        scenario.sufficient_data
    ));
    out.push_str(&format!("- Assessment: {}\n", scenario.assessment));
    out.push_str(&format!(
        "- Eligible recommendations: {}\n",
        scenario.eligible_recommendation_count
    ));
    out.push_str(&format!(
        "- Baseline weighted coverage: {:.1}%\n",
        scenario.baseline.coverage_weighted_ratio * 100.0
    ));
    if let Some(readiness) = scenario.baseline.readiness_projected {
        out.push_str(&format!(
            "- Baseline readiness projected: {:.1}\n\n",
            readiness
        ));
    } else {
        out.push_str("- Baseline readiness projected: n/a (abstaining)\n\n");
    }

    if scenario.policies.is_empty() {
        out.push_str("_No policy results (insufficient eligible topics)._\n\n");
        return;
    }

    out.push_str("| Policy | Learning gain | Coverage gain | Readiness Δ | Focus topics |\n");
    out.push_str("| --- | ---: | ---: | ---: | --- |\n");
    for policy in &scenario.policies {
        let readiness = policy
            .readiness_improvement
            .map(|delta| format!("{delta:+.2}"))
            .unwrap_or_else(|| "n/a".into());
        let topics = policy.selected_topic_ids.join(", ");
        out.push_str(&format!(
            "| {} | {:.3} | {:+.1}% | {} | {} |\n",
            policy.policy_label,
            policy.expected_learning_gain,
            policy.topic_coverage_gain * 100.0,
            readiness,
            topics,
        ));
    }
    out.push('\n');
    out.push_str(&format!(
        "- Best expected learning gain: **{}**\n",
        scenario.winners.expected_learning_gain
    ));
    out.push_str(&format!(
        "- Best topic coverage gain: **{}**\n",
        scenario.winners.topic_coverage_gain
    ));
    out.push_str(&format!(
        "- Best readiness improvement: **{}**\n\n",
        scenario.winners.readiness_improvement
    ));
}

fn load_collection_ablation(col: &mut Collection) -> Result<AblationScenarioResult> {
    let context = match load_collection_context(col)? {
        Some(context) => context,
        None => {
            return Ok(insufficient_scenario(
                "Collection",
                "collection",
                "fewer than three eligible study recommendations",
            ));
        }
    };
    Ok(evaluate_scenario(context, "Collection", "collection"))
}

fn load_collection_context(col: &mut Collection) -> Result<Option<AblationContext>> {
    let mastery = col.compute_topic_mastery(TopicMasteryRequest {
        search: gre_deck_search(),
        topic_tag_prefix: TOPIC_TAG_PREFIX.into(),
        mastery_threshold: None,
        min_reviews: 1,
    })?;
    let summary = mastery.summary.clone().unwrap_or_default();
    let observed_tags = observed_tags_from_mastery(&mastery.topics);
    let observed_refs: Vec<&str> = observed_tags.iter().map(String::as_str).collect();
    let mastery_by_id = mastery_map(&mastery.topics);

    let storage = gre_atlas_storage(col)?;
    let practice_by_topic: HashMap<String, (u32, u32)> = storage
        .performance_by_topic()?
        .into_iter()
        .map(|(topic, correct, total)| (topic, (correct, total)))
        .collect();
    let (perf_correct, perf_total) = storage.performance_summary()?;

    let recommendations =
        all_study_recommendations(&mastery_by_id, &observed_refs, &practice_by_topic);
    if recommendations.len() < DAILY_FOCUS_TOPIC_COUNT as usize {
        return Ok(None);
    }

    Ok(Some(AblationContext {
        recommendations,
        observed_tags,
        overall_avg_retrievability: summary.overall_avg_retrievability,
        fsrs_enabled: mastery.fsrs_enabled,
        studied_cards: summary.studied_cards,
        mastery_topics: mastery.topics,
        perf_correct,
        perf_total,
    }))
}

fn synthetic_reference_context() -> AblationContext {
    let mastery_topics = vec![
        topic_entry("gre::quant::algebra::linear", "Linear equations", 10, 0.42),
        topic_entry("gre::quant::geometry::triangles", "Triangles", 8, 0.55),
        topic_entry("gre::verbal::reading::main_idea", "Main idea", 6, 0.78),
    ];
    let observed_tags = vec![
        "gre::quant::algebra::linear".into(),
        "gre::quant::geometry::triangles".into(),
        "gre::verbal::reading::main_idea".into(),
    ];
    let mastery_by_id = mastery_map(&mastery_topics);
    let observed_refs: Vec<&str> = observed_tags.iter().map(String::as_str).collect();
    let practice_by_topic = HashMap::from([
        ("gre::quant::algebra::linear".to_string(), (1_u32, 5_u32)),
        (
            "gre::quant::geometry::triangles".to_string(),
            (2_u32, 6_u32),
        ),
    ]);

    let recommendations =
        all_study_recommendations(&mastery_by_id, &observed_refs, &practice_by_topic);

    AblationContext {
        recommendations,
        observed_tags,
        overall_avg_retrievability: 0.58,
        fsrs_enabled: true,
        studied_cards: 24,
        mastery_topics,
        perf_correct: 12,
        perf_total: 30,
    }
}

fn topic_entry(id: &str, display_name: &str, studied_cards: u32, avg: f32) -> TopicMasteryEntry {
    TopicMasteryEntry {
        topic_id: id.into(),
        display_name: display_name.into(),
        total_cards: studied_cards,
        studied_cards,
        mastered_cards: 0,
        avg_retrievability: avg,
        avg_retrievability_low: (avg - 0.08).max(0.0),
        avg_retrievability_high: (avg + 0.08).min(1.0),
        total_reviews: studied_cards * 2,
        ..Default::default()
    }
}

#[cfg(test)]
pub(crate) fn synthetic_reference_scenario() -> AblationScenarioResult {
    evaluate_scenario(
        synthetic_reference_context(),
        "Synthetic reference scenario",
        "synthetic_reference",
    )
}

fn evaluate_scenario(
    context: AblationContext,
    label: &str,
    data_source: &str,
) -> AblationScenarioResult {
    let eligible_recommendation_count = context.recommendations.len() as u32;
    let baseline = baseline_metrics(&context);
    let sufficient_data = eligible_recommendation_count >= DAILY_FOCUS_TOPIC_COUNT;

    if !sufficient_data {
        return AblationScenarioResult {
            label: label.into(),
            data_source: data_source.into(),
            sufficient_data: false,
            assessment: "insufficient eligible recommendations for ablation".into(),
            eligible_recommendation_count,
            baseline,
            policies: Vec::new(),
            winners: empty_winners(),
        };
    }

    let policies = vec![
        evaluate_policy(POLICY_GRE_ATLAS, &context, order_gre_atlas),
        evaluate_policy(POLICY_RANDOM, &context, order_random),
        evaluate_policy(POLICY_VANILLA, &context, order_vanilla),
    ];
    let winners = compute_winners(&policies);

    AblationScenarioResult {
        label: label.into(),
        data_source: data_source.into(),
        sufficient_data: true,
        assessment: scenario_assessment(&policies, &winners),
        eligible_recommendation_count,
        baseline,
        policies,
        winners,
    }
}

pub(crate) fn insufficient_scenario(
    label: &str,
    data_source: &str,
    reason: &str,
) -> AblationScenarioResult {
    AblationScenarioResult {
        label: label.into(),
        data_source: data_source.into(),
        sufficient_data: false,
        assessment: reason.into(),
        eligible_recommendation_count: 0,
        baseline: AblationBaseline {
            coverage_weighted_ratio: 0.0,
            readiness_projected: None,
        },
        policies: Vec::new(),
        winners: empty_winners(),
    }
}

fn baseline_metrics(context: &AblationContext) -> AblationBaseline {
    let observed_refs: Vec<&str> = context.observed_tags.iter().map(String::as_str).collect();
    let coverage_weighted_ratio = compute_coverage(&observed_refs).weighted_ratio;
    let readiness_projected = projected_readiness(context);
    AblationBaseline {
        coverage_weighted_ratio,
        readiness_projected,
    }
}

fn evaluate_policy(
    policy_id: &str,
    context: &AblationContext,
    order: fn(&[StudyPlanRecommendation]) -> Vec<StudyPlanRecommendation>,
) -> AblationPolicyResult {
    let ordered = order(&context.recommendations);
    let selected: Vec<_> = ordered
        .iter()
        .take(DAILY_FOCUS_TOPIC_COUNT as usize)
        .cloned()
        .collect();

    let expected_learning_gain = selected.iter().map(|rec| rec.priority_score).sum();
    let observed_refs: Vec<&str> = context.observed_tags.iter().map(String::as_str).collect();
    let coverage_before = compute_coverage(&observed_refs).weighted_ratio;

    let mut state = simulation_state_from_context(context);
    for recommendation in &selected {
        apply_focus_topic(recommendation, &mut state);
    }
    let tag_refs: Vec<&str> = state.observed_tags.iter().map(String::as_str).collect();
    let coverage_after = compute_coverage(&tag_refs).weighted_ratio;
    let topic_coverage_gain = coverage_after - coverage_before;

    let readiness_before = projected_readiness(context);
    let readiness_after = projected_readiness_from_state(&state);
    let readiness_improvement = match (readiness_before, readiness_after) {
        (Some(before), Some(after)) => Some(after - before),
        _ => None,
    };

    AblationPolicyResult {
        policy: policy_id.into(),
        policy_label: policy_label(policy_id),
        selected_topic_ids: selected.iter().map(|rec| rec.topic_id.clone()).collect(),
        expected_learning_gain,
        topic_coverage_gain,
        readiness_improvement,
    }
}

fn order_gre_atlas(recommendations: &[StudyPlanRecommendation]) -> Vec<StudyPlanRecommendation> {
    recommendations.to_vec()
}

fn order_random(recommendations: &[StudyPlanRecommendation]) -> Vec<StudyPlanRecommendation> {
    let mut ordered = recommendations.to_vec();
    let mut rng = StdRng::seed_from_u64(RANDOM_POLICY_SEED);
    ordered.shuffle(&mut rng);
    ordered
}

fn order_vanilla(recommendations: &[StudyPlanRecommendation]) -> Vec<StudyPlanRecommendation> {
    let mut ordered = recommendations.to_vec();
    ordered.sort_by(|a, b| a.topic_id.cmp(&b.topic_id));
    ordered
}

fn simulation_state_from_context(context: &AblationContext) -> SimulationState {
    SimulationState {
        observed_tags: context.observed_tags.clone(),
        overall_retrievability: context.overall_avg_retrievability,
        perf_correct: context.perf_correct,
        perf_total: context.perf_total,
        mastery_topics: context.mastery_topics.clone(),
        fsrs_enabled: context.fsrs_enabled,
        studied_cards: context.studied_cards,
    }
}

fn apply_focus_topic(recommendation: &StudyPlanRecommendation, state: &mut SimulationState) {
    if recommendation
        .factors
        .contains(&FACTOR_COVERAGE_GAP.to_string())
        && !recommendation.covered
    {
        state.observed_tags.push(recommendation.topic_id.clone());
    }

    if recommendation
        .factors
        .contains(&FACTOR_LOW_MASTERY.to_string())
    {
        if let Some(memory_score) = recommendation.memory_score {
            let current = memory_score / 100.0;
            let target = LOW_MEMORY_RETRIEVABILITY;
            if current < target {
                let bump =
                    (target - current) * MEMORY_FOCUS_BUMP_FRACTION * recommendation.exam_weight;
                state.overall_retrievability = (state.overall_retrievability + bump).min(1.0);
            }
        }
    }

    if recommendation
        .factors
        .contains(&FACTOR_LOW_PERFORMANCE.to_string())
        || recommendation
            .factors
            .contains(&FACTOR_NO_PRACTICE.to_string())
    {
        let accuracy = if recommendation
            .factors
            .contains(&FACTOR_LOW_PERFORMANCE.to_string())
        {
            recommendation
                .practice_accuracy
                .map(|observed| (observed + LOW_PRACTICE_ACCURACY) / 2.0)
                .unwrap_or(LOW_PRACTICE_ACCURACY)
        } else {
            LOW_PRACTICE_ACCURACY
        };
        let correct = (FOCUS_PRACTICE_TARGET as f32 * accuracy).round() as u32;
        state.perf_correct += correct;
        state.perf_total += FOCUS_PRACTICE_TARGET;
    }
}

fn projected_readiness(context: &AblationContext) -> Option<f32> {
    projected_readiness_from_state(&SimulationState {
        observed_tags: context.observed_tags.clone(),
        overall_retrievability: context.overall_avg_retrievability,
        perf_correct: context.perf_correct,
        perf_total: context.perf_total,
        mastery_topics: context.mastery_topics.clone(),
        fsrs_enabled: context.fsrs_enabled,
        studied_cards: context.studied_cards,
    })
}

fn projected_readiness_from_state(state: &SimulationState) -> Option<f32> {
    let tag_refs: Vec<&str> = state.observed_tags.iter().map(String::as_str).collect();
    let coverage = compute_coverage(&tag_refs);
    let memory = compute_memory_score(&MemoryInputs {
        fsrs_enabled: state.fsrs_enabled,
        overall_retrievability: state.overall_retrievability,
        coverage_ratio: coverage.weighted_ratio,
        studied_cards: state.studied_cards,
        topics: &state.mastery_topics,
    });
    let performance = compute_performance_score(&PerformanceInputs {
        correct: state.perf_correct,
        total: state.perf_total,
    });
    let readiness = compute_readiness_score(&memory, &performance, TimestampMillis::now().0);
    readiness.projected_score
}

fn compute_winners(policies: &[AblationPolicyResult]) -> AblationWinners {
    AblationWinners {
        expected_learning_gain: winner_by(policies, |policy| policy.expected_learning_gain),
        topic_coverage_gain: winner_by(policies, |policy| policy.topic_coverage_gain),
        readiness_improvement: winner_by_optional(policies, |policy| policy.readiness_improvement),
    }
}

fn winner_by(
    policies: &[AblationPolicyResult],
    value: impl Fn(&AblationPolicyResult) -> f32,
) -> String {
    policies
        .iter()
        .max_by(|left, right| {
            value(left)
                .partial_cmp(&value(right))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|policy| policy.policy_label.clone())
        .unwrap_or_else(|| "n/a".into())
}

fn winner_by_optional(
    policies: &[AblationPolicyResult],
    value: impl Fn(&AblationPolicyResult) -> Option<f32>,
) -> String {
    policies
        .iter()
        .filter_map(|policy| value(policy).map(|metric| (policy, metric)))
        .max_by(|(_, left), (_, right)| {
            left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(policy, _)| policy.policy_label.clone())
        .unwrap_or_else(|| "n/a".into())
}

fn scenario_assessment(policies: &[AblationPolicyResult], winners: &AblationWinners) -> String {
    let gre_atlas = policies
        .iter()
        .find(|policy| policy.policy == POLICY_GRE_ATLAS);
    let random = policies
        .iter()
        .find(|policy| policy.policy == POLICY_RANDOM);
    let vanilla = policies
        .iter()
        .find(|policy| policy.policy == POLICY_VANILLA);

    match (gre_atlas, random, vanilla) {
        (Some(a), Some(b), Some(c)) => format!(
            "GRE Atlas priority sum {:.3} vs random {:.3} vs vanilla {:.3}; coverage winners: {}; readiness winners: {}",
            a.expected_learning_gain,
            b.expected_learning_gain,
            c.expected_learning_gain,
            winners.topic_coverage_gain,
            winners.readiness_improvement,
        ),
        _ => "insufficient policy results".into(),
    }
}

fn empty_winners() -> AblationWinners {
    AblationWinners {
        expected_learning_gain: "n/a".into(),
        topic_coverage_gain: "n/a".into(),
        readiness_improvement: "n/a".into(),
    }
}

fn policy_label(policy_id: &str) -> String {
    match policy_id {
        POLICY_GRE_ATLAS => "GRE Atlas priority".into(),
        POLICY_RANDOM => "Random topic order".into(),
        POLICY_VANILLA => "Vanilla Anki order".into(),
        _ => policy_id.into(),
    }
}

pub(crate) fn ablation_methodology() -> AblationMethodology {
    AblationMethodology {
        description: "Counterfactual comparison of daily focus-topic ordering policies. Each policy selects the same number of focus topics from the current eligible recommendation pool, then a documented one-session simulation estimates downstream metrics.".into(),
        policies: vec![
            AblationPolicyDoc {
                id: POLICY_GRE_ATLAS.into(),
                label: policy_label(POLICY_GRE_ATLAS),
                ordering_rule: "Production GRE Atlas ranking: priority_score desc, then exam_weight desc, then topic_id asc.".into(),
            },
            AblationPolicyDoc {
                id: POLICY_RANDOM.into(),
                label: policy_label(POLICY_RANDOM),
                ordering_rule: format!(
                    "Deterministic shuffle of eligible recommendations (StdRng seed {RANDOM_POLICY_SEED}), then take top {DAILY_FOCUS_TOPIC_COUNT}."
                ),
            },
            AblationPolicyDoc {
                id: POLICY_VANILLA.into(),
                label: policy_label(POLICY_VANILLA),
                ordering_rule: "Lexicographic topic_id order among eligible recommendations (no GRE Atlas topic-priority layer).".into(),
            },
        ],
        metrics: vec![
            AblationMetricDoc {
                id: "expected_learning_gain".into(),
                label: "Expected learning gain".into(),
                definition: "Sum of production priority_score values for the selected focus topics (GRE Atlas's ranking objective).".into(),
            },
            AblationMetricDoc {
                id: "topic_coverage_gain".into(),
                label: "Topic coverage gain".into(),
                definition: "Delta in GRE catalog weighted coverage ratio after simulating focus actions that close coverage gaps.".into(),
            },
            AblationMetricDoc {
                id: "readiness_improvement".into(),
                label: "Readiness improvement".into(),
                definition: "Delta in projected readiness score after the one-session simulation; n/a when readiness abstains before or after.".into(),
            },
        ],
        simulation_model: format!(
            "For each selected focus topic, apply one daily focus session: uncovered topics with a coverage_gap factor are marked covered; low_mastery topics bump overall retrievability by (target - current) × {MEMORY_FOCUS_BUMP_FRACTION} × exam_weight toward {LOW_MEMORY_RETRIEVABILITY:.0}; low_performance/no_practice topics add {FOCUS_PRACTICE_TARGET} simulated practice attempts. Readiness is recomputed with production weights using updated coverage, memory, and performance inputs."
        ),
        synthetic_label: "Synthetic reference scenario uses hand-authored mastery/practice inputs labeled synthetic_reference; collection results use live GRE deck data when at least three eligible recommendations exist.".into(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn synthetic_reference_produces_three_policies() {
        let scenario = evaluate_scenario(
            synthetic_reference_context(),
            "Synthetic reference scenario",
            "synthetic_reference",
        );
        assert!(scenario.sufficient_data);
        assert_eq!(scenario.policies.len(), 3);
        assert!(scenario.eligible_recommendation_count >= DAILY_FOCUS_TOPIC_COUNT);
    }

    #[test]
    fn gre_atlas_wins_learning_gain_on_synthetic_reference() {
        let scenario = evaluate_scenario(
            synthetic_reference_context(),
            "Synthetic reference scenario",
            "synthetic_reference",
        );
        assert_eq!(
            scenario.winners.expected_learning_gain,
            policy_label(POLICY_GRE_ATLAS)
        );
        let gre_atlas = scenario
            .policies
            .iter()
            .find(|policy| policy.policy == POLICY_GRE_ATLAS)
            .unwrap();
        let random = scenario
            .policies
            .iter()
            .find(|policy| policy.policy == POLICY_RANDOM)
            .unwrap();
        assert!(gre_atlas.expected_learning_gain >= random.expected_learning_gain);
    }

    #[test]
    fn random_order_is_deterministic() {
        let context = synthetic_reference_context();
        let first = order_random(&context.recommendations)
            .into_iter()
            .map(|rec| rec.topic_id)
            .collect::<Vec<_>>();
        let second = order_random(&context.recommendations)
            .into_iter()
            .map(|rec| rec.topic_id)
            .collect::<Vec<_>>();
        assert_eq!(first, second);
        assert_ne!(
            first,
            order_gre_atlas(&context.recommendations)
                .into_iter()
                .map(|rec| rec.topic_id)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn vanilla_order_is_lexicographic() {
        let context = synthetic_reference_context();
        let ordered = order_vanilla(&context.recommendations);
        let ids: Vec<_> = ordered.iter().map(|rec| rec.topic_id.as_str()).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn markdown_includes_ablation_sections() {
        let ablation = AblationEval {
            model_version: ABLATION_MODEL_VERSION.into(),
            methodology: ablation_methodology(),
            focus_topic_count: DAILY_FOCUS_TOPIC_COUNT,
            collection: insufficient_scenario("Collection", "collection", "test"),
            synthetic_reference: evaluate_scenario(
                synthetic_reference_context(),
                "Synthetic reference scenario",
                "synthetic_reference",
            ),
        };
        let md = render_ablation_markdown(&ablation);
        assert!(md.contains("Topic-priority ablation"));
        assert!(md.contains("Synthetic reference scenario"));
        assert!(md.contains("GRE Atlas priority"));
        assert!(md.contains("Random topic order"));
        assert!(md.contains("Vanilla Anki order"));
    }
}
