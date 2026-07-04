// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Manually authored foundation practice bank — exemplars for AI generation.

/// Attribution for seed / foundation questions.
pub const FOUNDATION_SOURCE_NAME: &str = "GRE Atlas Practice Bank";

/// One row in the foundation JSON seed files.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FoundationQuestion {
    pub id: String,
    pub topic: String,
    pub section: String,
    #[serde(default = "default_format")]
    pub format: String,
    /// Primary stem text (legacy field name used by existing seeds).
    #[serde(default)]
    pub stem: String,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub choices: Vec<String>,
    #[serde(default, alias = "answer_choices")]
    pub answer_choices: Vec<String>,
    pub correct_answer: String,
    pub explanation: String,
    #[serde(default)]
    pub difficulty: Option<f32>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub subtopic: Option<String>,
    #[serde(default, alias = "question_type")]
    pub question_type: Option<String>,
    #[serde(default)]
    pub concepts_tested: Vec<String>,
    #[serde(default)]
    pub estimated_time_seconds: Option<u32>,
}

fn default_format() -> String {
    "mcq".into()
}

impl FoundationQuestion {
    pub fn stem_text(&self) -> &str {
        if !self.prompt.is_empty() {
            &self.prompt
        } else {
            &self.stem
        }
    }

    pub fn choice_list(&self) -> &[String] {
        if !self.answer_choices.is_empty() {
            &self.answer_choices
        } else {
            &self.choices
        }
    }

    /// Explanation with embedded metadata for fields not stored in bl_question.
    pub fn stored_explanation(&self) -> String {
        let mut out = self.explanation.clone();
        let mut meta = serde_json::Map::new();
        if let Some(sub) = &self.subtopic {
            meta.insert("subtopic".into(), sub.clone().into());
        }
        if let Some(qt) = &self.question_type {
            meta.insert("question_type".into(), qt.clone().into());
        }
        if !self.concepts_tested.is_empty() {
            meta.insert(
                "concepts_tested".into(),
                self.concepts_tested.clone().into(),
            );
        }
        if let Some(secs) = self.estimated_time_seconds {
            meta.insert("estimated_time_seconds".into(), secs.into());
        }
        if !meta.is_empty() {
            let json = serde_json::Value::Object(meta);
            out.push_str("\n\n<!-- meta: ");
            out.push_str(&json.to_string());
            out.push_str(" -->");
        }
        out
    }

    pub fn source_name(&self) -> &str {
        self.source
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(FOUNDATION_SOURCE_NAME)
    }
}

/// All foundation questions from the split seed files.
pub fn load_foundation_bank() -> Vec<FoundationQuestion> {
    let mut all = Vec::new();
    all.extend(parse_foundation_file(include_str!("seed_gre_quant.json")));
    all.extend(parse_foundation_file(include_str!("seed_gre_verbal.json")));
    all.extend(parse_foundation_file(include_str!("seed_gre_awa.json")));
    all
}

fn parse_foundation_file(json: &str) -> Vec<FoundationQuestion> {
    serde_json::from_str(json).unwrap_or_else(|err| {
        panic!("foundation seed parse error: {err}");
    })
}

/// Minimum counts for the manually authored foundation bank.
pub const MIN_FOUNDATION_VERBAL: usize = 150;
pub const MIN_FOUNDATION_QUANT: usize = 150;
pub const MIN_FOUNDATION_AWA: usize = 25;

/// Return manually authored exemplars for a catalog leaf topic.
/// Future LLM generation should condition on these rows rather than invent
/// stems from scratch.
pub fn exemplars_for_topic(topic_id: &str) -> Vec<FoundationQuestion> {
    load_foundation_bank()
        .into_iter()
        .filter(|q| q.topic == topic_id)
        .collect()
}

/// One exemplar stem per leaf topic for lightweight prompt conditioning.
pub fn exemplar_stems_by_topic() -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    for q in load_foundation_bank() {
        out.entry(q.topic.clone())
            .or_insert_with(|| q.stem_text().to_string());
    }
    out
}

/// Target difficulty mix: 30% easy, 50% medium, 20% hard.
pub const DIFFICULTY_EASY_MAX: f32 = 0.35;
pub const DIFFICULTY_HARD_MIN: f32 = 0.65;

pub fn difficulty_bucket(d: Option<f32>) -> &'static str {
    match d.unwrap_or(0.5) {
        x if x <= DIFFICULTY_EASY_MAX => "easy",
        x if x >= DIFFICULTY_HARD_MIN => "hard",
        _ => "medium",
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::gre_atlas::domain::GreCatalog;
    use crate::gre_atlas::questions::variants::correct_answer_in_choices;
    use std::collections::HashMap;
    use std::collections::HashSet;

    #[test]
    fn foundation_bank_meets_minimum_counts() {
        let bank = load_foundation_bank();
        let verbal = bank.iter().filter(|q| q.section == "verbal").count();
        let quant = bank.iter().filter(|q| q.section == "quant").count();
        let awa = bank.iter().filter(|q| q.section == "awa").count();
        assert!(
            verbal >= MIN_FOUNDATION_VERBAL,
            "verbal: got {verbal}, want {}",
            MIN_FOUNDATION_VERBAL
        );
        assert!(
            quant >= MIN_FOUNDATION_QUANT,
            "quant: got {quant}, want {}",
            MIN_FOUNDATION_QUANT
        );
        assert!(
            awa >= MIN_FOUNDATION_AWA,
            "awa: got {awa}, want {}",
            MIN_FOUNDATION_AWA
        );
    }

    #[test]
    fn foundation_bank_is_valid() {
        let bank = load_foundation_bank();
        let mut ids = HashSet::new();
        for q in &bank {
            assert!(ids.insert(q.id.as_str()), "duplicate id: {}", q.id);
            assert!(
                !q.stem_text().is_empty(),
                "{} missing stem/prompt",
                q.id
            );
            let choices = q.choice_list();
            assert!(!choices.is_empty(), "{} has no choices", q.id);
            assert!(
                correct_answer_in_choices(&q.correct_answer, choices),
                "{}: correct_answer not in choices",
                q.id
            );
            assert!(
                GreCatalog::topic_by_id(&q.topic).is_some(),
                "{}: unknown topic {}",
                q.id,
                q.topic
            );
        }
    }

    #[test]
    fn foundation_covers_every_leaf_topic() {
        let bank = load_foundation_bank();
        for leaf in GreCatalog::leaf_topics() {
            assert!(
                bank.iter().any(|q| q.topic == leaf.id),
                "no foundation question for {}",
                leaf.id
            );
        }
    }

    #[test]
    fn foundation_difficulty_distribution() {
        let bank = load_foundation_bank();
        let mut buckets: HashMap<&str, u32> = HashMap::new();
        for q in &bank {
            *buckets
                .entry(difficulty_bucket(q.difficulty))
                .or_default() += 1;
        }
        let total = bank.len() as f32;
        let easy_pct = buckets.get("easy").copied().unwrap_or(0) as f32 / total;
        let hard_pct = buckets.get("hard").copied().unwrap_or(0) as f32 / total;
        // Allow ±8% tolerance on 30/20 targets.
        assert!(
            (0.22..=0.38).contains(&easy_pct),
            "easy share {easy_pct:.2}, want ~0.30"
        );
        assert!(
            (0.12..=0.28).contains(&hard_pct),
            "hard share {hard_pct:.2}, want ~0.20"
        );
    }

    #[test]
    fn foundation_has_unique_ids_across_files() {
        let bank = load_foundation_bank();
        let ids: HashSet<_> = bank.iter().map(|q| q.id.as_str()).collect();
        assert_eq!(ids.len(), bank.len());
    }
}
