// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::AbstentionRequirement;

pub(crate) const MIN_STUDIED_CARDS: u32 = 50;
pub(crate) const MIN_COVERAGE_RATIO: f32 = 0.5;
pub(crate) const MIN_PERFORMANCE_ATTEMPTS: u32 = 50;

pub(crate) const REQ_FSRS_ENABLED: &str = "fsrs_enabled";
pub(crate) const REQ_STUDIED_CARDS: &str = "studied_cards";
pub(crate) const REQ_TOPIC_COVERAGE: &str = "topic_coverage";
pub(crate) const REQ_PRACTICE_ATTEMPTS: &str = "practice_attempts";

pub(crate) fn memory_requirements(
    fsrs_enabled: bool,
    studied_cards: u32,
    coverage_ratio: f32,
) -> Vec<AbstentionRequirement> {
    vec![
        fsrs_requirement(fsrs_enabled),
        studied_cards_requirement(studied_cards),
        topic_coverage_requirement(coverage_ratio),
    ]
}

pub(crate) fn performance_requirements(attempt_count: u32) -> Vec<AbstentionRequirement> {
    vec![practice_attempts_requirement(attempt_count)]
}

pub(crate) fn readiness_requirements(
    memory: &[AbstentionRequirement],
    performance: &[AbstentionRequirement],
) -> Vec<AbstentionRequirement> {
    let mut requirements = memory.to_vec();
    requirements.extend_from_slice(performance);
    requirements
}

pub(crate) fn sufficient_from_requirements(requirements: &[AbstentionRequirement]) -> bool {
    requirements.iter().all(|req| req.met)
}

pub(crate) fn abstain_reason_from_requirements(requirements: &[AbstentionRequirement]) -> String {
    requirements
        .iter()
        .filter(|req| !req.met)
        .map(|req| format!("{}: {}", req.label, req.status))
        .collect::<Vec<_>>()
        .join("; ")
}

pub(crate) fn sufficient_data_and_reason(
    fsrs_enabled: bool,
    unique_studied: u32,
    coverage_ratio: f32,
) -> (bool, String) {
    let requirements = memory_requirements(fsrs_enabled, unique_studied, coverage_ratio);
    (
        sufficient_from_requirements(&requirements),
        abstain_reason_from_requirements(&requirements),
    )
}

fn fsrs_requirement(fsrs_enabled: bool) -> AbstentionRequirement {
    AbstentionRequirement {
        id: REQ_FSRS_ENABLED.into(),
        label: "FSRS scheduling".into(),
        status: if fsrs_enabled {
            "FSRS enabled".into()
        } else {
            "FSRS is disabled".into()
        },
        next_step: "Enable FSRS in deck options for the GRE Atlas deck.".into(),
        met: fsrs_enabled,
    }
}

fn studied_cards_requirement(studied_cards: u32) -> AbstentionRequirement {
    let met = studied_cards >= MIN_STUDIED_CARDS;
    AbstentionRequirement {
        id: REQ_STUDIED_CARDS.into(),
        label: "Studied GRE cards".into(),
        status: if met {
            format!("{studied_cards} studied cards (minimum {MIN_STUDIED_CARDS})")
        } else {
            format!("{studied_cards} of {MIN_STUDIED_CARDS} studied cards (each reviewed at least once)")
        },
        next_step: format!(
            "Review cards in the GRE Atlas deck until at least {MIN_STUDIED_CARDS} cards have one review."
        ),
        met,
    }
}

fn topic_coverage_requirement(coverage_ratio: f32) -> AbstentionRequirement {
    let met = coverage_ratio >= MIN_COVERAGE_RATIO;
    let current_pct = (coverage_ratio * 100.0).round() as u32;
    let required_pct = (MIN_COVERAGE_RATIO * 100.0).round() as u32;
    AbstentionRequirement {
        id: REQ_TOPIC_COVERAGE.into(),
        label: "Topic coverage".into(),
        status: if met {
            format!("{current_pct}% exam-weighted catalog covered (minimum {required_pct}%)")
        } else {
            format!("Only {current_pct}% of the GRE has evidence.")
        },
        next_step: "Tag and review cards across more GRE topics to reach 50% catalog coverage."
            .into(),
        met,
    }
}

fn practice_attempts_requirement(attempt_count: u32) -> AbstentionRequirement {
    let met = attempt_count >= MIN_PERFORMANCE_ATTEMPTS;
    AbstentionRequirement {
        id: REQ_PRACTICE_ATTEMPTS.into(),
        label: "GRE practice attempts".into(),
        status: if attempt_count == 0 {
            "No practice attempts yet".into()
        } else if met {
            format!("{attempt_count} practice attempts (minimum {MIN_PERFORMANCE_ATTEMPTS})")
        } else {
            format!("{attempt_count} of {MIN_PERFORMANCE_ATTEMPTS} practice attempts")
        },
        next_step: if attempt_count == 0 {
            "Open Practice and answer GRE-style questions.".into()
        } else {
            format!(
                "Keep practicing until you have at least {MIN_PERFORMANCE_ATTEMPTS} scored attempts."
            )
        },
        met,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn unmet_ids(requirements: &[AbstentionRequirement]) -> Vec<&str> {
        requirements
            .iter()
            .filter(|req| !req.met)
            .map(|req| req.id.as_str())
            .collect()
    }

    #[test]
    fn fsrs_disabled_abstains() {
        let reqs = memory_requirements(false, 300, 0.8);
        assert!(!sufficient_from_requirements(&reqs));
        assert_eq!(unmet_ids(&reqs), vec![REQ_FSRS_ENABLED]);
        let fsrs = reqs.first().unwrap();
        assert!(!fsrs.next_step.is_empty());
    }

    #[test]
    fn insufficient_studied_cards_abstains() {
        let reqs = memory_requirements(true, 45, 0.8);
        assert!(!sufficient_from_requirements(&reqs));
        assert_eq!(unmet_ids(&reqs), vec![REQ_STUDIED_CARDS]);
        assert!(reqs[1].status.contains("45"));
        assert!(reqs[1].status.contains("50"));
    }

    #[test]
    fn insufficient_topic_coverage_abstains() {
        let reqs = memory_requirements(true, 250, 0.35);
        assert!(!sufficient_from_requirements(&reqs));
        assert_eq!(unmet_ids(&reqs), vec![REQ_TOPIC_COVERAGE]);
        assert!(reqs[2].status.contains("35%"));
        assert!(reqs[2].status.contains("GRE has evidence"));
    }

    #[test]
    fn memory_sufficient_when_all_requirements_met() {
        let reqs = memory_requirements(true, 250, 0.6);
        assert!(sufficient_from_requirements(&reqs));
        assert!(abstain_reason_from_requirements(&reqs).is_empty());
    }

    #[test]
    fn no_practice_attempts_abstains() {
        let reqs = performance_requirements(0);
        assert!(!sufficient_from_requirements(&reqs));
        assert_eq!(unmet_ids(&reqs), vec![REQ_PRACTICE_ATTEMPTS]);
        assert_eq!(reqs[0].status, "No practice attempts yet");
        assert!(reqs[0].next_step.contains("Practice"));
    }

    #[test]
    fn insufficient_practice_attempts_abstains() {
        let reqs = performance_requirements(8);
        assert!(!sufficient_from_requirements(&reqs));
        assert_eq!(unmet_ids(&reqs), vec![REQ_PRACTICE_ATTEMPTS]);
        assert!(reqs[0].status.contains("8"));
        assert!(reqs[0].status.contains("50"));
    }

    #[test]
    fn performance_sufficient_at_minimum_attempts() {
        let reqs = performance_requirements(MIN_PERFORMANCE_ATTEMPTS);
        assert!(sufficient_from_requirements(&reqs));
    }

    #[test]
    fn readiness_lists_all_unmet_requirements() {
        let memory = memory_requirements(false, 10, 0.1);
        let performance = performance_requirements(3);
        let readiness = readiness_requirements(&memory, &performance);
        assert_eq!(readiness.len(), 4);
        assert!(!sufficient_from_requirements(&readiness));
        let unmet = unmet_ids(&readiness);
        assert_eq!(
            unmet,
            vec![
                REQ_FSRS_ENABLED,
                REQ_STUDIED_CARDS,
                REQ_TOPIC_COVERAGE,
                REQ_PRACTICE_ATTEMPTS,
            ]
        );
        let reason = abstain_reason_from_requirements(&readiness);
        assert!(reason.contains("FSRS scheduling"));
        assert!(reason.contains("Studied GRE cards"));
        assert!(reason.contains("Topic coverage"));
        assert!(reason.contains("GRE practice attempts"));
    }

    #[test]
    fn sufficient_data_and_reason_matches_memory_requirements() {
        let (sufficient, reason) = sufficient_data_and_reason(true, 250, 0.6);
        assert!(sufficient);
        assert!(reason.is_empty());

        let (sufficient, reason) = sufficient_data_and_reason(false, 250, 0.6);
        assert!(!sufficient);
        assert!(reason.contains("FSRS scheduling"));
    }
}
