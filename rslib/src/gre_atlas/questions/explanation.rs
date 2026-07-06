// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Post-answer explanations.
//!
//! Given a stored question and the learner's selected answer, produce a
//! structured explanation that (a) says why the correct answer is correct,
//! (b) says why *each* distractor is wrong, and (c) cites the grounding source.
//!
//! The optional LLM path generates this on demand; the deterministic fallback
//! builds it from the stored explanation plus per-distractor templated
//! reasoning. The fallback is always available and never errors — when it is
//! used, [`OFFLINE_TEMPLATE_NOTE`] is attached so the UI can surface exactly
//! "Generated using offline templates.".

use crate::gre_atlas::questions::foundation::FOUNDATION_SOURCE_NAME;
use crate::gre_atlas::questions::llm::LlmChatRequest;
use crate::gre_atlas::questions::llm::LlmClient;
use crate::gre_atlas::questions::metadata::Provenance;
use crate::gre_atlas::questions::metadata::OFFLINE_TEMPLATE_NOTE;
use crate::gre_atlas::questions::metadata::PROVENANCE_AI;
use crate::gre_atlas::questions::metadata::PROVENANCE_TEMPLATE;
use crate::gre_atlas::questions::source::source_section_for_topic;
use crate::gre_atlas::questions::source::GENERATION_SOURCE_NAME;
use crate::gre_atlas::storage::StoredQuestion;

/// Worked solution shown in the Explain Answer panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolutionExplanationData {
    pub concept: String,
    pub formula: String,
    pub steps: Vec<String>,
    pub final_answer: String,
    pub common_mistake: String,
    pub key_takeaways: Vec<String>,
    pub citation: String,
}

/// Reasoning for a single answer choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceExplanation {
    pub label: String,
    pub choice: String,
    pub is_correct: bool,
    pub reason: String,
    pub likely_misconception: String,
    pub student_reasoning: String,
    pub correct_reasoning: String,
    pub difference: String,
    /// Legacy one-line note; mirrors `reason` for backward-compatible clients.
    pub reasoning: String,
}

/// "Easy" / "Medium" / "Hard" from the stored 0–1 difficulty (grounded in data).
pub(crate) fn difficulty_label(difficulty: Option<f32>) -> String {
    match difficulty {
        Some(d) if d >= 0.66 => "Hard".to_string(),
        Some(d) if d >= 0.33 => "Medium".to_string(),
        Some(_) => "Easy".to_string(),
        None => String::new(),
    }
}

/// Suggested solving budget derived from section + difficulty (a template, not
/// a per-question fabrication).
pub(crate) fn estimated_time_label(question: &StoredQuestion) -> String {
    let base: f64 = match question.section.as_str() {
        "awa" => return "~30 min (essay)".to_string(),
        "verbal" => 1.5,
        _ => 2.0,
    };
    let factor: f64 = match question.difficulty {
        Some(d) if d >= 0.66 => 1.3,
        Some(d) if d < 0.33 => 0.7,
        _ => 1.0,
    };
    let minutes = base * factor;
    if (minutes - minutes.round()).abs() < 0.05 {
        format!("~{} min", minutes.round() as i64)
    } else {
        format!("~{minutes:.1} min")
    }
}

/// Related GRE areas derived from the topic path ancestry (humanized segments).
pub(crate) fn related_topics_for(topic: &str) -> Vec<String> {
    let mut segments: Vec<&str> = topic
        .split("::")
        .filter(|seg| !seg.is_empty() && *seg != "gre")
        .collect();
    segments.reverse();
    segments
        .into_iter()
        .skip(1) // drop the leaf itself; keep its ancestors as related areas
        .map(humanize_segment)
        .collect()
}

fn humanize_segment(seg: &str) -> String {
    let mut chars = seg.replace('_', " ");
    if let Some(first) = chars.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    chars
}

/// Grounded trap-recognition advice tied to the concept when no structured
/// template supplies a specific one.
pub(crate) fn generic_trap_recognition(concept: &str) -> String {
    let concept = concept.trim();
    if concept.is_empty() {
        return "Re-read exactly what the question asks and verify each step before choosing."
            .to_string();
    }
    format!(
        "On future {concept} questions, re-read what is being asked and check your result against the formula before selecting."
    )
}

/// A second solving approach for concepts that have a well-known alternative
/// (grounded), empty otherwise so the UI can hide it.
pub(crate) fn alternative_method_for(concept: &str) -> String {
    let c = concept.to_lowercase();
    if c.contains("circumference") {
        "Alternatively, use C = πd with the diameter (d = 2r).".to_string()
    } else if c.contains("area") && c.contains("circle") {
        "Alternatively, square the radius first, then multiply by π.".to_string()
    } else if c.contains("percent") {
        "Alternatively, multiply the base by (1 + rate) and compare with the new value."
            .to_string()
    } else {
        String::new()
    }
}

/// Humanized leaf-topic label (e.g. `gre::quant::geometry::circles` → "circles").
pub(crate) fn concept_from_topic(topic: &str) -> String {
    topic
        .rsplit("::")
        .next()
        .unwrap_or("problem solving")
        .replace('_', " ")
}

/// A complete, structured post-answer explanation.
#[derive(Debug, Clone)]
pub struct AnswerExplanationData {
    pub summary: String,
    pub solution: Option<SolutionExplanationData>,
    pub choices: Vec<ChoiceExplanation>,
    pub correct_answer: String,
    pub citation_source_name: String,
    pub citation_source_section: String,
    pub citation_excerpt: String,
    pub provenance: Provenance,
    pub provenance_note: String,
    pub model_version: String,
}

/// Build an explanation for `question` given the learner's `selected_answer`.
///
/// Tries the LLM path when `ai` is `Some`; on any failure falls back to the
/// deterministic templated explanation. Never returns an error.
pub fn build_answer_explanation(
    question: &StoredQuestion,
    selected_answer: &str,
    ai: Option<&dyn LlmClient>,
) -> AnswerExplanationData {
    if let Some(client) = ai {
        if let Some(explanation) = try_llm_explanation(question, selected_answer, client) {
            return explanation;
        }
    }
    build_template_explanation(question)
}

/// Deterministic explanation from stored data + per-distractor templates.
pub fn build_template_explanation(question: &StoredQuestion) -> AnswerExplanationData {
    let summary = clean_explanation(&question.explanation);
    let citation = citation_for(question);
    let citation_line = format_solution_citation(&citation.0, &citation.1);

    if let Some(structured) = build_structured_offline_explanation(question, &summary, &citation_line)
    {
        return structured;
    }

    let choices = question
        .choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            build_legacy_choice_explanation(question, choice, index, &summary)
        })
        .collect();

    let solution = generic_solution_from_summary(question, &summary, &citation_line);
    let display_summary = if summary.is_empty() {
        format!("The correct answer is \"{}\".", question.correct_answer)
    } else {
        summary.clone()
    };

    AnswerExplanationData {
        summary: display_summary,
        solution: Some(solution),
        choices,
        correct_answer: question.correct_answer.clone(),
        citation_source_name: citation.0,
        citation_source_section: citation.1,
        citation_excerpt: citation.2,
        provenance: Provenance::OfflineTemplate,
        provenance_note: OFFLINE_TEMPLATE_NOTE.to_string(),
        model_version: crate::gre_atlas::questions::ai_gen::AI_GENERATION_MODEL_VERSION.to_string(),
    }
}

fn format_solution_citation(source_name: &str, source_section: &str) -> String {
    if source_section.is_empty() {
        source_name.to_string()
    } else {
        format!("{source_name} — {source_section}")
    }
}

fn build_legacy_choice_explanation(
    question: &StoredQuestion,
    choice: &str,
    index: usize,
    summary: &str,
) -> ChoiceExplanation {
    let is_correct = choice.trim() == question.correct_answer.trim();
    let reason = if is_correct {
        correct_choice_reasoning(summary, &question.stem, choice)
    } else {
        distractor_reasoning(
            choice,
            &question.correct_answer,
            summary,
            &question.stem,
            index,
        )
    };
    let correct_reasoning = if is_correct {
        reason.clone()
    } else {
        correct_choice_reasoning(summary, &question.stem, &question.correct_answer)
    };
    choice_from_reason(
        choice_label(index),
        choice,
        is_correct,
        &reason,
        &correct_reasoning,
        summary,
        &question.correct_answer,
    )
}

fn choice_from_reason(
    label: String,
    choice: &str,
    is_correct: bool,
    reason: &str,
    correct_reasoning: &str,
    summary: &str,
    correct_answer: &str,
) -> ChoiceExplanation {
    let (likely_misconception, student_reasoning, difference) = if is_correct {
        (
            String::new(),
            String::new(),
            correct_reasoning.to_string(),
        )
    } else {
        (
            generic_misconception(choice, correct_answer, summary),
            generic_student_reasoning(choice, correct_answer),
            format!(
                "The correct approach leads to \"{correct_answer}\", not \"{choice}\"."
            ),
        )
    };
    ChoiceExplanation {
        label,
        choice: choice.to_string(),
        is_correct,
        reason: reason.to_string(),
        likely_misconception,
        student_reasoning,
        correct_reasoning: if is_correct {
            correct_reasoning.to_string()
        } else {
            correct_reasoning.to_string()
        },
        difference,
        reasoning: reason.to_string(),
    }
}

fn generic_misconception(choice: &str, correct: &str, summary: &str) -> String {
    if summary.is_empty() {
        return format!("Selecting \"{choice}\" without matching the question's logic.");
    }
    format!(
        "Treating \"{choice}\" as equivalent to the correct result \"{correct}\"."
    )
}

fn generic_student_reasoning(choice: &str, correct: &str) -> String {
    format!(
        "A student might pick \"{choice}\" after a quick estimate or partial calculation instead of verifying against \"{correct}\"."
    )
}

fn generic_solution_from_summary(
    question: &StoredQuestion,
    summary: &str,
    citation: &str,
) -> SolutionExplanationData {
    let concept = question
        .topic
        .rsplit("::")
        .next()
        .unwrap_or("problem solving")
        .replace('_', " ");
    let steps = if summary.is_empty() {
        vec![format!(
            "The correct answer is \"{}\".",
            question.correct_answer
        )]
    } else {
        vec![summary.to_string()]
    };
    SolutionExplanationData {
        concept: format!("GRE {concept}"),
        formula: String::new(),
        steps,
        final_answer: question.correct_answer.clone(),
        common_mistake: "Rushing to an answer without checking each step.".to_string(),
        key_takeaways: vec![if summary.is_empty() {
            format!("Confirm the result matches \"{}\".", question.correct_answer)
        } else {
            summary.to_string()
        }],
        citation: citation.to_string(),
    }
}

fn choice_label(index: usize) -> String {
    char::from(b'A' + (index as u8).min(25)).to_string()
}

/// Try topic-specific offline templates (circles, percent change, …).
fn build_structured_offline_explanation(
    question: &StoredQuestion,
    summary: &str,
    citation: &str,
) -> Option<AnswerExplanationData> {
    let meta = parse_explanation_meta(&question.explanation);
    let subtopic = meta
        .as_ref()
        .and_then(|m| m.get("subtopic"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let stem_l = question.stem.to_lowercase();

    if subtopic == "circumference"
        || (question.topic.contains("circles") && stem_l.contains("circumference"))
    {
        return build_circle_circumference_explanation(question, summary, citation);
    }
    if subtopic == "area" || (question.topic.contains("circles") && stem_l.contains("area")) {
        return build_circle_area_explanation(question, summary, citation);
    }
    if let Some(ctx) = parse_percent_change_context(summary, &question.stem) {
        return Some(build_percent_change_structured_explanation(
            question, summary, citation, &ctx,
        ));
    }
    None
}

fn parse_radius_from_stem(stem: &str) -> Option<f64> {
    let lower = stem.to_lowercase();
    let idx = lower.find("radius")?;
    let after = &stem[idx + "radius".len()..];
    let cleaned: String = after
        .trim_start()
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    cleaned.parse().ok()
}

fn parse_diameter_from_stem(stem: &str) -> Option<f64> {
    let lower = stem.to_lowercase();
    let idx = lower.find("diameter")?;
    let after = &stem[idx + "diameter".len()..];
    let cleaned: String = after
        .trim_start()
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    cleaned.parse().ok()
}

fn parse_pi_coefficient(raw: &str) -> Option<f64> {
    let trimmed = raw.trim();
    if trimmed == "π" {
        return Some(1.0);
    }
    let lower = trimmed.to_lowercase();
    if lower.ends_with('π') {
        let coeff: String = lower
            .trim_end_matches('π')
            .trim()
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect();
        if coeff.is_empty() || coeff == "." || coeff == "-" {
            return Some(1.0);
        }
        return coeff.parse().ok();
    }
    None
}

fn format_pi_expression(coeff: f64) -> String {
    if (coeff - 1.0).abs() < f64::EPSILON {
        "π".to_string()
    } else if (coeff - coeff.round()).abs() < f64::EPSILON {
        format!("{}π", coeff.round() as i64)
    } else {
        format!("{coeff}π")
    }
}

fn build_circle_circumference_explanation(
    question: &StoredQuestion,
    summary: &str,
    citation: &str,
) -> Option<AnswerExplanationData> {
    let radius = parse_radius_from_stem(&question.stem)?;
    let correct_coeff = 2.0 * radius;
    let correct_label = format_pi_expression(correct_coeff);
    if question.correct_answer.trim() != correct_label {
        return None;
    }

    let concept = "Circle circumference from radius".to_string();
    let formula = "C = 2πr".to_string();
    let radius_label = format_number(radius);
    let steps = vec![
        format!("Identify the radius: r = {radius_label}."),
        "Apply the circumference formula: C = 2πr.".to_string(),
        format!("Substitute: C = 2π({radius_label}) = {correct_label}."),
    ];
    let common_mistake =
        "Using πr (half the circumference) or πr² (the area formula) instead of 2πr.".to_string();
    let key_takeaways = vec![
        "Circumference scales linearly with radius: C = 2πr.".to_string(),
        "Do not confuse circumference (2πr) with area (πr²).".to_string(),
        "Read carefully whether the question asks for distance around (circumference) or space inside (area)."
            .to_string(),
    ];
    let solution = SolutionExplanationData {
        concept: concept.clone(),
        formula: formula.clone(),
        steps: steps.clone(),
        final_answer: correct_label.clone(),
        common_mistake: common_mistake.clone(),
        key_takeaways: key_takeaways.clone(),
        citation: citation.to_string(),
    };

    let correct_reasoning = format!(
        "Circumference = 2πr = 2π({radius_label}) = {correct_label}."
    );
    let choices = question
        .choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let is_correct = choice.trim() == correct_label;
            let label = choice_label(index);
            if is_correct {
                return ChoiceExplanation {
                    label,
                    choice: choice.clone(),
                    is_correct: true,
                    reason: correct_reasoning.clone(),
                    likely_misconception: String::new(),
                    student_reasoning: String::new(),
                    correct_reasoning: correct_reasoning.clone(),
                    difference: correct_reasoning.clone(),
                    reasoning: correct_reasoning.clone(),
                };
            }
            circle_circumference_distractor(
                choice,
                radius,
                &correct_label,
                &correct_reasoning,
                &label,
            )
        })
        .collect();

    let display_summary = if summary.is_empty() {
        steps.last().cloned().unwrap_or(correct_reasoning.clone())
    } else {
        summary.to_string()
    };

    let citation_triple = citation_for(question);
    Some(finish_structured_explanation(
        question,
        display_summary,
        solution,
        choices,
        citation_triple,
    ))
}

fn circle_circumference_distractor(
    choice: &str,
    radius: f64,
    correct_label: &str,
    correct_reasoning: &str,
    label: &str,
) -> ChoiceExplanation {
    let choice_trim = choice.trim();
    let area_coeff = radius * radius;
    let half_coeff = radius;
    let double_radius_coeff = 2.0 * (radius + 1.0);

    let (reason, misconception, student, difference) =
        if let Some(coeff) = parse_pi_coefficient(choice_trim) {
            if (coeff - area_coeff).abs() < f64::EPSILON {
                (
                    format!(
                        "{choice_trim} comes from the area formula πr² = π({})² = {}, not circumference.",
                        format_number(radius),
                        format_pi_expression(area_coeff)
                    ),
                    "Confusing circumference with area (using πr² instead of 2πr).".to_string(),
                    format!(
                        "Compute πr²: π × {}² = {}",
                        format_number(radius),
                        format_pi_expression(area_coeff)
                    ),
                    format!(
                        "Area uses πr²; circumference requires 2πr, which gives {correct_label}."
                    ),
                )
            } else if (coeff - half_coeff).abs() < f64::EPSILON {
                (
                    format!(
                        "{choice_trim} equals πr = π({}), which is only half the circumference.",
                        format_number(radius)
                    ),
                    "Using πr instead of 2πr (forgetting to multiply by 2).".to_string(),
                    format!(
                        "Multiply once by π: π × {} = {}",
                        format_number(radius),
                        format_pi_expression(half_coeff)
                    ),
                    format!(
                        "Circumference needs both diameters worth of π: 2πr = {correct_label}, not πr."
                    ),
                )
            } else if (coeff - double_radius_coeff).abs() < f64::EPSILON {
                (
                    format!(
                        "{choice_trim} would be 2π({}), using the wrong radius.",
                        format_number(radius + 1.0)
                    ),
                    "Plugging in the wrong radius value.".to_string(),
                    format!(
                        "Mistakenly use r = {}: 2π × {} = {}",
                        format_number(radius + 1.0),
                        format_number(radius + 1.0),
                        format_pi_expression(double_radius_coeff)
                    ),
                    format!(
                        "The stem gives r = {}; only {correct_label} matches 2πr.",
                        format_number(radius)
                    ),
                )
            } else {
                (
                    format!(
                        "{choice_trim} does not equal 2π({radius}) = {correct_label}."
                    ),
                    "Arithmetic slip or misapplied circle formula.".to_string(),
                    format!(
                        "A rough estimate or misapplied formula yields {choice_trim} instead of 2πr."
                    ),
                    format!(
                        "Recompute with C = 2πr to get {correct_label}, not {choice_trim}."
                    ),
                )
            }
        } else {
            (
                format!("{choice_trim} is not the circumference for r = {}.", format_number(radius)),
                "Selecting an expression that does not follow C = 2πr.".to_string(),
                format!("Pick {choice_trim} without verifying C = 2πr."),
                format!("Only {correct_label} satisfies C = 2πr."),
            )
        };

    ChoiceExplanation {
        label: label.to_string(),
        choice: choice.to_string(),
        is_correct: false,
        reason: reason.clone(),
        likely_misconception: misconception,
        student_reasoning: student,
        correct_reasoning: correct_reasoning.to_string(),
        difference,
        reasoning: reason,
    }
}

fn build_circle_area_explanation(
    question: &StoredQuestion,
    summary: &str,
    citation: &str,
) -> Option<AnswerExplanationData> {
    let radius = parse_radius_from_stem(&question.stem).or_else(|| {
        parse_diameter_from_stem(&question.stem).map(|d| d / 2.0)
    })?;
    let correct_coeff = radius * radius;
    let correct_label = format_pi_expression(correct_coeff);
    if question.correct_answer.trim() != correct_label {
        return None;
    }

    let radius_label = format_number(radius);
    let steps = vec![
        format!("Identify the radius: r = {radius_label}."),
        "Apply the area formula: A = πr².".to_string(),
        format!("Substitute: A = π({radius_label})² = {correct_label}."),
    ];
    let correct_reasoning = format!(
        "Area = πr² = π({radius_label})² = {correct_label}."
    );
    let solution = SolutionExplanationData {
        concept: "Circle area from radius".to_string(),
        formula: "A = πr²".to_string(),
        steps: steps.clone(),
        final_answer: correct_label.clone(),
        common_mistake:
            "Using 2πr (circumference) or πr (half the circumference) instead of πr².".to_string(),
        key_takeaways: vec![
            "Area depends on r², not r — doubling the radius quadruples the area.".to_string(),
            "Circumference is 2πr; area is πr² — check which the question asks for.".to_string(),
            "Square the radius before multiplying by π.".to_string(),
        ],
        citation: citation.to_string(),
    };

    let circumference_coeff = 2.0 * radius;
    let choices = question
        .choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let is_correct = choice.trim() == correct_label;
            let label = choice_label(index);
            if is_correct {
                return ChoiceExplanation {
                    label,
                    choice: choice.clone(),
                    is_correct: true,
                    reason: correct_reasoning.clone(),
                    likely_misconception: String::new(),
                    student_reasoning: String::new(),
                    correct_reasoning: correct_reasoning.clone(),
                    difference: correct_reasoning.clone(),
                    reasoning: correct_reasoning.clone(),
                };
            }
            let choice_trim = choice.trim();
            let (reason, misconception, student, difference) =
                if let Some(coeff) = parse_pi_coefficient(choice_trim) {
                    if (coeff - circumference_coeff).abs() < f64::EPSILON {
                        (
                            format!(
                                "{choice_trim} equals 2πr — that is circumference, not area."
                            ),
                            "Confusing area (πr²) with circumference (2πr).".to_string(),
                            format!(
                                "Compute 2πr = 2π × {} = {}",
                                radius_label,
                                format_pi_expression(circumference_coeff)
                            ),
                            format!(
                                "Area needs πr² = {correct_label}, not the distance around the circle."
                            ),
                        )
                    } else if (coeff - radius).abs() < f64::EPSILON {
                        (
                            format!("{choice_trim} equals πr, not πr²."),
                            "Multiplying by π once instead of squaring the radius first.".to_string(),
                            format!("Use πr = π × {radius_label} = {choice_trim}"),
                            format!(
                                "Square the radius first: π × {radius_label}² = {correct_label}."
                            ),
                        )
                    } else {
                        (
                            format!(
                                "{choice_trim} does not equal π({radius_label})² = {correct_label}."
                            ),
                            "Misapplied area formula or wrong radius.".to_string(),
                            format!(
                                "An incorrect r or skipped squaring yields {choice_trim}."
                            ),
                            format!("Only πr² with r = {radius_label} gives {correct_label}."),
                        )
                    }
                } else {
                    (
                        format!("{choice_trim} is not the area for r = {radius_label}."),
                        "Answer does not follow A = πr².".to_string(),
                        format!("Select {choice_trim} without computing πr²."),
                        format!("The area is {correct_label}."),
                    )
                };
            ChoiceExplanation {
                label,
                choice: choice.clone(),
                is_correct: false,
                reason: reason.clone(),
                likely_misconception: misconception,
                student_reasoning: student,
                correct_reasoning: correct_reasoning.clone(),
                difference,
                reasoning: reason,
            }
        })
        .collect();

    let display_summary = if summary.is_empty() {
        steps.last().cloned().unwrap_or(correct_reasoning.clone())
    } else {
        summary.to_string()
    };
    let citation_triple = citation_for(question);
    Some(finish_structured_explanation(
        question,
        display_summary,
        solution,
        choices,
        citation_triple,
    ))
}

fn build_percent_change_structured_explanation(
    question: &StoredQuestion,
    summary: &str,
    citation: &str,
    ctx: &PercentChangeContext,
) -> AnswerExplanationData {
    let change = format_number(ctx.change);
    let base = format_number(ctx.base);
    let correct = question.correct_answer.trim();
    let formula = "Percent change = (new − old) ÷ old × 100".to_string();
    let steps = vec![
        format!("Find the change: new − old = {change}."),
        format!("Divide by the original (old) value: {change} ÷ {base}."),
        format!(
            "Convert to a percent: ({change}/{base}) × 100 = {correct}."
        ),
    ];
    let solution = SolutionExplanationData {
        concept: "Percent increase from a base value".to_string(),
        formula,
        steps: steps.clone(),
        final_answer: correct.to_string(),
        common_mistake:
            "Using the raw change as the percent, or dividing by the new value instead of the original."
                .to_string(),
        key_takeaways: vec![
            "Percent change always uses the starting value as the denominator.".to_string(),
            "Convert the ratio to a percent by multiplying by 100.".to_string(),
            "Do not report the raw difference as a percent.".to_string(),
        ],
        citation: citation.to_string(),
    };

    let correct_reasoning = percent_change_correct_reasoning(ctx, correct);
    let choices = question
        .choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let is_correct = choice.trim() == correct;
            let label = choice_label(index);
            if is_correct {
                return ChoiceExplanation {
                    label,
                    choice: choice.clone(),
                    is_correct: true,
                    reason: correct_reasoning.clone(),
                    likely_misconception: String::new(),
                    student_reasoning: String::new(),
                    correct_reasoning: correct_reasoning.clone(),
                    difference: correct_reasoning.clone(),
                    reasoning: correct_reasoning.clone(),
                };
            }
            let reason = numeric_distractor_reasoning(
                parse_scalar_value(choice).unwrap_or(0.0),
                parse_scalar_value(correct).unwrap_or(0.0),
                choice,
                correct,
                summary,
                &question.stem,
                index,
            );
            choice_from_reason(
                label,
                choice,
                false,
                &reason,
                &correct_reasoning,
                summary,
                correct,
            )
        })
        .collect();

    let display_summary = if summary.is_empty() {
        correct_reasoning.clone()
    } else {
        summary.to_string()
    };
    let citation_triple = citation_for(question);
    finish_structured_explanation(
        question,
        display_summary,
        solution,
        choices,
        citation_triple,
    )
}

fn finish_structured_explanation(
    question: &StoredQuestion,
    summary: String,
    solution: SolutionExplanationData,
    choices: Vec<ChoiceExplanation>,
    citation: (String, String, String),
) -> AnswerExplanationData {
    AnswerExplanationData {
        summary,
        solution: Some(solution),
        choices,
        correct_answer: question.correct_answer.clone(),
        citation_source_name: citation.0,
        citation_source_section: citation.1,
        citation_excerpt: citation.2,
        provenance: Provenance::OfflineTemplate,
        provenance_note: OFFLINE_TEMPLATE_NOTE.to_string(),
        model_version: crate::gre_atlas::questions::ai_gen::AI_GENERATION_MODEL_VERSION.to_string(),
    }
}

/// Templated reasoning for a distractor. References the specific wrong option,
/// the correct answer, and the grounding explanation.
fn distractor_reasoning(
    choice: &str,
    correct_answer: &str,
    summary: &str,
    stem: &str,
    choice_index: usize,
) -> String {
    let choice_trim = choice.trim();
    let correct_trim = correct_answer.trim();

    if let (Some(wrong_val), Some(correct_val)) = (
        parse_scalar_value(choice_trim),
        parse_scalar_value(correct_trim),
    ) {
        return numeric_distractor_reasoning(
            wrong_val,
            correct_val,
            choice_trim,
            correct_trim,
            summary,
            stem,
            choice_index,
        );
    }

    text_distractor_reasoning(
        choice_trim,
        correct_trim,
        summary,
        stem,
        choice_index,
    )
}

fn text_distractor_reasoning(
    choice: &str,
    correct: &str,
    summary: &str,
    stem: &str,
    choice_index: usize,
) -> String {
    let choice_l = choice.to_lowercase();

    if let Some(reason) = restates_faulty_inference(choice, &choice_l, stem, summary) {
        return reason;
    }
    if let Some(reason) = absolutist_language_flaw(choice, &choice_l, summary, choice_index) {
        return reason;
    }
    if let Some(reason) = tangential_objection(choice, correct, summary, stem) {
        return reason;
    }

    varied_text_fallback(choice, correct, summary, choice_index)
}

fn restates_faulty_inference(
    choice: &str,
    choice_l: &str,
    stem: &str,
    summary: &str,
) -> Option<String> {
    let stem_l = stem.to_lowercase();
    let mirrors_argument_link = (stem_l.contains("download") && choice_l.contains("download")
        && choice_l.contains("satisfaction"))
        || (stem_l.contains("satisfaction") && choice_l.contains("satisfaction")
            && (choice_l.contains("download") || choice_l.contains("sale")))
        || (stem_l.contains("logo") && choice_l.contains("logo") && choice_l.contains("sales"))
        || (stem_l.contains("visitor") && choice_l.contains("customer")
            && choice_l.contains("satisfied"));

    let asserts_link = choice_l.contains("equals")
        || choice_l.contains("same as")
        || choice_l.contains("always")
        || choice_l.contains("must have")
        || choice_l.contains("must mean")
        || choice_l.contains("proves")
        || choice_l.contains("shows that");

    if mirrors_argument_link && asserts_link {
        return Some(format!(
            "\"{choice}\" repeats the argument's shaky link instead of challenging it. {}",
            summary_detail(summary, 0)
        ));
    }

    if choice_l.contains("always")
        && (choice_l.contains("equal") || choice_l.contains("same") || choice_l.contains("reflect"))
    {
        return Some(format!(
            "\"{choice}\" treats two separate ideas as permanently tied together. {}",
            summary_detail(summary, 1)
        ));
    }

    None
}

fn absolutist_language_flaw(
    choice: &str,
    choice_l: &str,
    summary: &str,
    choice_index: usize,
) -> Option<String> {
    let patterns: &[(&str, &str)] = &[
        (
            "never ",
            "states something can never happen, which is stronger than the argument requires and misses the real flaw",
        ),
        (
            "always ",
            "uses \"always,\" turning a possible relationship into a certainty",
        ),
        (
            "cannot ",
            "claims something is impossible instead of identifying the weak inference",
        ),
        (
            "can never",
            "rules out a whole category of outcomes rather than critiquing the reasoning",
        ),
        (
            "must ",
            "treats the conclusion as mandatory instead of showing why the evidence is insufficient",
        ),
        (
            "all ",
            "overgeneralizes with \"all\" instead of targeting the specific logical gap",
        ),
        (
            "none ",
            "denies every case outright rather than explaining what the argument gets wrong",
        ),
        (
            "every ",
            "applies to every case without addressing the actual weakness in the argument",
        ),
        (
            "no ",
            "makes a blanket denial that does not fix the reasoning problem",
        ),
    ];

    for (pattern, critique) in patterns {
        if choice_l.contains(pattern) {
            return Some(format!(
                "\"{choice}\" {critique}. {}",
                summary_detail(summary, choice_index)
            ));
        }
    }

    None
}

fn tangential_objection(choice: &str, correct: &str, summary: &str, stem: &str) -> Option<String> {
    let mut focus = content_tokens(stem);
    focus.extend(content_tokens(correct));
    focus.extend(content_tokens(summary));
    let choice_tokens = content_tokens(choice);
    if choice_tokens.is_empty() {
        return None;
    }
    let overlap = choice_tokens.intersection(&focus).count();
    if overlap == 0 {
        return Some(format!(
            "\"{choice}\" raises a side issue instead of the flaw in the argument. {}",
            summary_detail(summary, 2)
        ));
    }
    None
}

fn varied_text_fallback(choice: &str, correct: &str, summary: &str, choice_index: usize) -> String {
    if summary.is_empty() {
        let templates = [
            "\"{choice}\" does not follow from the question. The correct answer is \"{correct}\".",
            "\"{choice}\" is not supported by the passage. Choose \"{correct}\" instead.",
            "The evidence points to \"{correct}\", not \"{choice}\".",
        ];
        let template = templates[choice_index % templates.len()];
        return template
            .replace("{choice}", choice)
            .replace("{correct}", correct);
    }

    let templates = [
        "\"{choice}\" does not expose the gap in the argument. {detail}",
        "\"{choice}\" sounds related but misses the flaw. {detail}",
        "Look for an objection like \"{correct}\", not \"{choice}\". {detail}",
        "\"{choice}\" does not explain why the reasoning breaks down. {detail}",
        "\"{choice}\" may be tempting, but it does not fit the passage. {detail}",
    ];
    let detail = summary_detail(summary, choice_index);
    templates[choice_index % templates.len()]
        .replace("{choice}", choice)
        .replace("{correct}", correct)
        .replace("{detail}", &detail)
}

fn summary_detail(summary: &str, variant: usize) -> String {
    if summary.is_empty() {
        return String::new();
    }
    match variant % 4 {
        0 => summary.to_string(),
        1 => format!("The real issue: {summary}"),
        2 => format!("Focus on this instead: {summary}"),
        _ => format!("{summary}"),
    }
}

const STOP_WORDS: &[&str] = &[
    "a", "an", "the", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
    "do", "does", "did", "will", "would", "could", "should", "may", "might", "must", "shall",
    "can", "not", "nor", "so", "if", "or", "and", "but", "in", "on", "at", "to", "for", "of",
    "with", "by", "from", "as", "that", "this", "it", "its", "than", "then", "what", "which",
    "who", "whom", "when", "where", "why", "how", "our", "your", "their", "his", "her", "its",
];

fn content_tokens(text: &str) -> std::collections::HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| word.len() > 2 && !STOP_WORDS.contains(word))
        .map(str::to_string)
        .collect()
}

fn correct_choice_reasoning(summary: &str, stem: &str, correct_label: &str) -> String {
    if let Some(ctx) = parse_percent_change_context(summary, stem) {
        return percent_change_correct_reasoning(&ctx, correct_label);
    }
    if summary.is_empty() {
        return "This is the correct answer.".to_string();
    }
    summary.to_string()
}

#[derive(Debug, Clone, Copy)]
struct PercentChangeContext {
    change: f64,
    base: f64,
    new_value: Option<f64>,
}

fn parse_percent_change_context(summary: &str, stem: &str) -> Option<PercentChangeContext> {
    parse_increase_on_base_summary(summary).or_else(|| parse_percent_change_from_stem(stem))
}

fn parse_increase_on_base_summary(summary: &str) -> Option<PercentChangeContext> {
    let lower = summary.to_lowercase();
    if !lower.contains("increase") || !lower.contains("base") {
        return None;
    }
    let numbers = extract_numbers(summary);
    if numbers.len() >= 2 {
        let change = numbers[0];
        let base = numbers[1];
        if base > 0.0 {
            return Some(PercentChangeContext {
                change,
                base,
                new_value: Some(base + change),
            });
        }
    }
    None
}

fn parse_percent_change_from_stem(stem: &str) -> Option<PercentChangeContext> {
    let stem_l = stem.to_lowercase();
    if !stem_l.contains("percent change") {
        return None;
    }
    let values: Vec<f64> = stem
        .split('|')
        .filter_map(|segment| {
            segment
                .rsplit_once(':')
                .and_then(|(_, raw)| extract_numbers(raw).first().copied())
        })
        .collect();
    if values.len() >= 2 {
        let base = values[values.len() - 2];
        let new_value = values[values.len() - 1];
        let change = new_value - base;
        if base > 0.0 && change >= 0.0 {
            return Some(PercentChangeContext {
                change,
                base,
                new_value: Some(new_value),
            });
        }
    }
    None
}

fn extract_numbers(text: &str) -> Vec<f64> {
    text.split(|c: char| !c.is_ascii_digit() && c != '.')
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .collect()
}

fn percent_change_correct_reasoning(ctx: &PercentChangeContext, correct_label: &str) -> String {
    let change = format_number(ctx.change);
    let base = format_number(ctx.base);
    if let Some(new_value) = ctx.new_value {
        let new_label = format_number(new_value);
        return format!(
            "Percent change = (new − old) ÷ old × 100 = ({new_label} − {base}) ÷ {base} = {change}/{base} = {correct_label}."
        );
    }
    format!(
        "Percent change = change ÷ original × 100 = {change}/{base} = {correct_label}."
    )
}

fn format_number(value: f64) -> String {
    if (value - value.round()).abs() < f64::EPSILON {
        format!("{}", value.round() as i64)
    } else {
        format!("{value}")
    }
}

fn numeric_distractor_reasoning(
    wrong: f64,
    correct: f64,
    wrong_label: &str,
    correct_label: &str,
    summary: &str,
    stem: &str,
    _choice_index: usize,
) -> String {
    let is_percent = wrong_label.contains('%') || correct_label.contains('%');
    if is_percent {
        if let Some(ctx) = parse_percent_change_context(summary, stem) {
            return percent_distractor_reasoning(wrong, correct, wrong_label, correct_label, &ctx);
        }
    }

    let direction = if wrong > correct {
        format!("{wrong_label} is larger than the correct result ({correct_label})")
    } else if wrong < correct {
        format!("{wrong_label} is smaller than the correct result ({correct_label})")
    } else {
        format!("{wrong_label} is not the correct result ({correct_label})")
    };

    if summary.is_empty() {
        format!("{direction}. Recheck the arithmetic.")
    } else if is_terse_worked_hint(summary) {
        format!("{direction}. Work through the steps instead of guessing.")
    } else {
        format!("{direction}. {summary}")
    }
}

fn is_terse_worked_hint(summary: &str) -> bool {
    let lower = summary.to_lowercase();
    lower.contains("increase") && lower.contains("base")
}

fn percent_distractor_reasoning(
    wrong_pct: f64,
    correct_pct: f64,
    wrong_label: &str,
    correct_label: &str,
    ctx: &PercentChangeContext,
) -> String {
    let change = ctx.change;
    let base = ctx.base;

    if (wrong_pct - change).abs() < 0.5 {
        return format!(
            "{wrong_label} treats the raw change ({}) as the percent. Divide by the starting value: {}/{} = {}.",
            format_number(change),
            format_number(change),
            format_number(base),
            correct_label
        );
    }

    if let Some(new_value) = ctx.new_value {
        let pct_of_new = (change / new_value) * 100.0;
        if (wrong_pct - pct_of_new).abs() < 2.0 {
            return format!(
                "{wrong_label} divides by the new value ({}) instead of the original ({}). Percent change uses the starting amount as the base.",
                format_number(new_value),
                format_number(base)
            );
        }
    }

    if wrong_pct > correct_pct {
        format!(
            "{wrong_label} is too high. Percent increase = {} ÷ {} × 100 = {}.",
            format_number(change),
            format_number(base),
            correct_label
        )
    } else {
        format!(
            "{wrong_label} is too low. Recompute: {} ÷ {} = {}.",
            format_number(change),
            format_number(base),
            correct_label
        )
    }
}

fn parse_scalar_value(raw: &str) -> Option<f64> {
    let cleaned: String = raw
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '.' || *ch == '-')
        .collect();
    if cleaned.is_empty() || cleaned == "." || cleaned == "-" {
        return None;
    }
    cleaned.parse().ok()
}

fn try_llm_explanation(
    question: &StoredQuestion,
    selected_answer: &str,
    client: &dyn LlmClient,
) -> Option<AnswerExplanationData> {
    let prompt = build_explanation_prompt(question, selected_answer);
    let request = LlmChatRequest {
        system: EXPLANATION_SYSTEM_PROMPT.to_string(),
        user: prompt,
        temperature: 0.3,
    };
    let content = client.complete(&request).ok()?;
    let parsed = parse_explanation_json(&content)?;

    // Require that the model addressed the correct answer; otherwise fall back.
    if parsed.summary.trim().is_empty() || parsed.choices.is_empty() {
        return None;
    }

    let choices = question
        .choices
        .iter()
        .enumerate()
        .map(|(index, choice)| {
            let is_correct = choice.trim() == question.correct_answer.trim();
            let parsed_choice = parsed
                .choices
                .iter()
                .find(|c| c.choice.trim() == choice.trim());
            let reason = parsed_choice
                .and_then(|c| c.reason.clone().or_else(|| {
                    if c.reasoning.trim().is_empty() {
                        None
                    } else {
                        Some(c.reasoning.clone())
                    }
                }))
                .filter(|r| !r.trim().is_empty())
                .unwrap_or_else(|| {
                    if is_correct {
                        correct_choice_reasoning(
                            parsed.summary.trim(),
                            &question.stem,
                            choice,
                        )
                    } else {
                        distractor_reasoning(
                            choice,
                            &question.correct_answer,
                            parsed.summary.trim(),
                            &question.stem,
                            index,
                        )
                    }
                });
            let correct_reasoning = if is_correct {
                reason.clone()
            } else {
                correct_choice_reasoning(
                    parsed.summary.trim(),
                    &question.stem,
                    &question.correct_answer,
                )
            };
            let label = parsed_choice
                .and_then(|c| {
                    let l = c.label.trim();
                    if l.is_empty() {
                        None
                    } else {
                        Some(l.to_string())
                    }
                })
                .unwrap_or_else(|| choice_label(index));
            ChoiceExplanation {
                label,
                choice: choice.clone(),
                is_correct,
                reason: reason.clone(),
                likely_misconception: parsed_choice
                    .map(|c| c.likely_misconception.clone())
                    .unwrap_or_default(),
                student_reasoning: parsed_choice
                    .map(|c| c.student_reasoning.clone())
                    .unwrap_or_default(),
                correct_reasoning: parsed_choice
                    .and_then(|c| {
                        let r = c.correct_reasoning.trim();
                        if r.is_empty() {
                            None
                        } else {
                            Some(r.to_string())
                        }
                    })
                    .unwrap_or(correct_reasoning),
                difference: parsed_choice
                    .map(|c| c.difference.clone())
                    .unwrap_or_default(),
                reasoning: reason,
            }
        })
        .collect();

    let citation = citation_for(question);
    let citation_line = format_solution_citation(&citation.0, &citation.1);
    let solution = parsed.solution.as_ref().map(|s| SolutionExplanationData {
        concept: s.concept.clone(),
        formula: s.formula.clone(),
        steps: s.steps.clone(),
        final_answer: if s.final_answer.trim().is_empty() {
            question.correct_answer.clone()
        } else {
            s.final_answer.clone()
        },
        common_mistake: s.common_mistake.clone(),
        key_takeaways: s.key_takeaways.clone(),
        citation: if s.citation.trim().is_empty() {
            citation_line.clone()
        } else {
            s.citation.clone()
        },
    });

    Some(AnswerExplanationData {
        summary: parsed.summary.trim().to_string(),
        solution,
        choices,
        correct_answer: question.correct_answer.clone(),
        citation_source_name: citation.0,
        citation_source_section: citation.1,
        citation_excerpt: citation.2,
        provenance: Provenance::AiGenerated,
        provenance_note: format!("Explained by {}.", client.model_version()),
        model_version: client.model_version().to_string(),
    })
}

/// Whether this row was produced by the template/LLM generation pipeline and
/// should cite the bundled ETS excerpts — not manually authored foundation
/// items.
fn question_uses_ets_grounding(question: &StoredQuestion) -> bool {
    matches!(
        question.provenance.as_deref(),
        Some(PROVENANCE_AI) | Some(PROVENANCE_TEMPLATE)
    ) || question.source_name.as_deref() == Some(GENERATION_SOURCE_NAME)
}

/// Citation triple: (source_name, source_section, excerpt).
///
/// Generated items cite the bundled ETS excerpt they were grounded in.
/// Foundation bank items cite their stored named source and never fabricate an
/// ETS excerpt.
fn citation_for(question: &StoredQuestion) -> (String, String, String) {
    if question_uses_ets_grounding(question) {
        if let Some(source) = source_section_for_topic(&question.topic) {
            let section = question
                .source_section
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(source.section);
            return (
                GENERATION_SOURCE_NAME.to_string(),
                section.to_string(),
                source.excerpt.to_string(),
            );
        }
    }

    let name = question
        .source_name
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| FOUNDATION_SOURCE_NAME.to_string());
    let section = question
        .source_section
        .clone()
        .or_else(|| question.source_document.clone())
        .unwrap_or_default();
    (name, section, String::new())
}

/// Strip the embedded `<!-- meta: ... -->` block foundation questions append to
/// their explanation, plus surrounding whitespace.
fn clean_explanation(explanation: &str) -> String {
    let without_meta = match explanation.find("<!-- meta:") {
        Some(idx) => &explanation[..idx],
        None => explanation,
    };
    without_meta.trim().to_string()
}

pub(crate) fn parse_explanation_meta(explanation: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
    let idx = explanation.find("<!-- meta:")?;
    let rest = explanation.get(idx + "<!-- meta:".len()..)?;
    let end = rest.find("-->")?;
    let json_str = rest[..end].trim();
    let value: serde_json::Value = serde_json::from_str(json_str).ok()?;
    value.as_object().cloned()
}

pub(crate) fn question_type_key(format: &str, explanation: &str) -> String {
    parse_explanation_meta(explanation)
        .and_then(|meta| {
            meta.get("question_type")
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| format.to_string())
}

const EXPLANATION_SYSTEM_PROMPT: &str = "You are a patient GRE tutor. Explain why the correct \
    answer is correct and why every other option is wrong with tutor-style reasoning — never just \
    \"Incorrect.\" Ground explanations in the provided source excerpt. Reply with strict JSON and \
    nothing else.";

fn build_explanation_prompt(question: &StoredQuestion, selected_answer: &str) -> String {
    let (source_name, source_section, excerpt) = citation_for(question);
    let choices = question
        .choices
        .iter()
        .map(|c| format!("- {c}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Source: {source_name}\nSection: {source_section}\nExcerpt: {excerpt}\n\n\
         Question: {stem}\nChoices:\n{choices}\nCorrect answer: {correct}\n\
         Learner selected: {selected}\n\n\
         Return JSON: {{\"summary\": string, \"solution\": {{\"concept\": string, \"formula\": \
         string, \"steps\": [string], \"final_answer\": string, \"common_mistake\": string, \
         \"key_takeaways\": [string], \"citation\": string}}, \"choices\": [{{\"label\": string, \
         \"choice\": string, \"is_correct\": boolean, \"reason\": string, \"likely_misconception\": \
         string, \"student_reasoning\": string, \"correct_reasoning\": string, \"difference\": \
         string}}]}}. Include one entry per choice with labels A, B, C, …",
        stem = question.stem,
        correct = question.correct_answer,
        selected = selected_answer,
    )
}

#[derive(serde::Deserialize)]
struct ExplanationJson {
    #[serde(default)]
    summary: String,
    #[serde(default)]
    solution: Option<SolutionJson>,
    #[serde(default)]
    choices: Vec<ExplanationChoiceJson>,
}

#[derive(serde::Deserialize)]
struct SolutionJson {
    #[serde(default)]
    concept: String,
    #[serde(default)]
    formula: String,
    #[serde(default)]
    steps: Vec<String>,
    #[serde(default)]
    final_answer: String,
    #[serde(default)]
    common_mistake: String,
    #[serde(default)]
    key_takeaways: Vec<String>,
    #[serde(default)]
    citation: String,
}

#[derive(serde::Deserialize)]
struct ExplanationChoiceJson {
    #[serde(default)]
    label: String,
    #[serde(default)]
    choice: String,
    #[serde(default)]
    reasoning: String,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    likely_misconception: String,
    #[serde(default)]
    student_reasoning: String,
    #[serde(default)]
    correct_reasoning: String,
    #[serde(default)]
    difference: String,
}

fn parse_explanation_json(content: &str) -> Option<ExplanationJson> {
    let start = content.find('{')?;
    let end = content.rfind('}')?;
    if end < start {
        return None;
    }
    serde_json::from_str(&content[start..=end]).ok()
}

#[cfg(test)]
mod test {
    use super::*;

    fn stored(explanation: &str) -> StoredQuestion {
        StoredQuestion {
            id: "q1".into(),
            topic: "gre::quant::algebra::linear".into(),
            section: "quant".into(),
            format: "mcq".into(),
            stem: "If 4x + 8 = 20, what is x?".into(),
            choices: vec!["1".into(), "2".into(), "3".into(), "4".into()],
            correct_answer: "3".into(),
            explanation: explanation.into(),
            difficulty: Some(0.4),
            source_name: None,
            source_section: None,
            generated_at_secs: None,
            generation_confidence: None,
            source_document: None,
            model_version: None,
            provenance: None,
            evaluation_status: None,
        }
    }

    #[test]
    fn template_explanation_covers_every_choice_and_cites_source() {
        let mut q = stored("Subtract 8 then divide by 4: x = 3.");
        q.source_name = Some(GENERATION_SOURCE_NAME.to_string());
        q.provenance = Some(PROVENANCE_TEMPLATE.to_string());
        let out = build_template_explanation(&q);
        assert_eq!(out.provenance, Provenance::OfflineTemplate);
        assert_eq!(out.provenance_note, OFFLINE_TEMPLATE_NOTE);
        assert_eq!(out.choices.len(), 4);
        let correct: Vec<_> = out.choices.iter().filter(|c| c.is_correct).collect();
        assert_eq!(correct.len(), 1);
        assert_eq!(correct[0].choice, "3");
        assert!(correct[0].reasoning.contains("Subtract 8"));
        // every distractor names itself and the correct answer with distinct text
        let wrong: Vec<_> = out.choices.iter().filter(|c| !c.is_correct).collect();
        let unique_reasoning: std::collections::HashSet<_> =
            wrong.iter().map(|c| c.reasoning.as_str()).collect();
        assert_eq!(unique_reasoning.len(), wrong.len());
        for c in wrong {
            assert!(c.reasoning.contains(&c.choice), "{}", c.reasoning);
            assert!(c.reasoning.contains('3'), "{}", c.reasoning);
        }
        assert_eq!(out.citation_source_name, GENERATION_SOURCE_NAME);
        assert!(!out.citation_excerpt.is_empty());
    }

    #[test]
    fn template_explanation_strips_meta_comment() {
        let q = stored("x = 3.\n\n<!-- meta: {\"subtopic\":\"linear\"} -->");
        let out = build_template_explanation(&q);
        assert!(!out.summary.contains("meta"));
        assert_eq!(out.summary, "x = 3.");
    }

    #[test]
    fn build_uses_template_when_ai_absent() {
        let q = stored("x = 3.");
        let out = build_answer_explanation(&q, "1", None);
        assert_eq!(out.provenance, Provenance::OfflineTemplate);
    }

    #[test]
    fn foundation_question_cites_practice_bank_not_ets() {
        let mut q = stored("20% of $120 is $24; $120 − $24 = $96.");
        q.source_name = Some(FOUNDATION_SOURCE_NAME.to_string());
        q.source_section = Some("percent_discount".into());
        let out = build_template_explanation(&q);
        assert_eq!(out.citation_source_name, FOUNDATION_SOURCE_NAME);
        assert_eq!(out.citation_source_section, "percent_discount");
        assert!(out.citation_excerpt.is_empty());
        assert_ne!(out.citation_source_name, GENERATION_SOURCE_NAME);
    }

    #[test]
    fn generated_question_cites_ets_excerpt() {
        let mut q = stored("Subtract 8 then divide by 4: x = 3.");
        q.source_name = Some(GENERATION_SOURCE_NAME.to_string());
        q.source_section = Some("Quantitative Reasoning — Linear equations".into());
        q.provenance = Some(PROVENANCE_TEMPLATE.to_string());
        let out = build_template_explanation(&q);
        assert_eq!(out.citation_source_name, GENERATION_SOURCE_NAME);
        assert!(!out.citation_excerpt.is_empty());
    }

    #[test]
    fn argument_distractors_get_distinct_reasoning() {
        let mut q = stored("The argument equates usage metrics with satisfaction without evidence.");
        q.stem = "Argument: \"Our app downloads increased, so customer satisfaction must have improved.\" What is the main flaw?".into();
        q.choices = vec![
            "Downloads may not reflect satisfaction.".into(),
            "Satisfaction always equals downloads.".into(),
            "Apps cannot be measured.".into(),
            "Customers never use downloaded apps.".into(),
        ];
        q.correct_answer = "Downloads may not reflect satisfaction.".into();
        let out = build_template_explanation(&q);
        let wrong: Vec<_> = out.choices.iter().filter(|c| !c.is_correct).collect();
        assert_eq!(wrong.len(), 3);
        let unique_reasoning: std::collections::HashSet<_> =
            wrong.iter().map(|c| c.reasoning.as_str()).collect();
        assert_eq!(
            unique_reasoning.len(),
            wrong.len(),
            "expected distinct reasoning: {:?}",
            wrong.iter().map(|c| &c.reasoning).collect::<Vec<_>>()
        );
        for c in &wrong {
            assert!(
                !c.reasoning.contains("does not match the reasoning for"),
                "{}",
                c.reasoning
            );
        }
        assert!(wrong[0].reasoning.contains("always"));
        assert!(wrong[1].reasoning.contains("impossible") || wrong[1].reasoning.contains("cannot"));
        assert!(wrong[2].reasoning.contains("never"));
    }

    #[test]
    fn percent_change_explanations_name_mistakes_without_repeating_hint() {
        let mut q = stored("Increase 15 on base 50.");
        q.topic = "gre::quant::data_interpretation".into();
        q.stem = "Sales by period — 2019: 40 | 2020: 50 | 2021: 65. What is the percent change from 2020 to 2021?".into();
        q.choices = vec![
            "40%".into(),
            "15%".into(),
            "25%".into(),
            "30%".into(),
        ];
        q.correct_answer = "30%".into();
        let out = build_template_explanation(&q);

        let correct = out.choices.iter().find(|c| c.is_correct).unwrap();
        assert!(
            correct.reasoning.contains("50") && correct.reasoning.contains("30%"),
            "{}",
            correct.reasoning
        );
        assert!(
            !correct.reasoning.starts_with("Increase 15"),
            "{}",
            correct.reasoning
        );

        let fifteen = out
            .choices
            .iter()
            .find(|c| c.choice == "15%")
            .unwrap();
        assert!(
            fifteen.reasoning.contains("raw change") || fifteen.reasoning.contains("15/50"),
            "{}",
            fifteen.reasoning
        );
        assert!(
            !fifteen.reasoning.contains("The real issue"),
            "{}",
            fifteen.reasoning
        );

        let order: Vec<_> = out.choices.iter().map(|c| c.choice.as_str()).collect();
        assert_eq!(order, vec!["40%", "15%", "25%", "30%"]);
    }
}
