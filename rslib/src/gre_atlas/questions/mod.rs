// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

pub mod ai_gen;
pub mod source;

use anki_proto::brainlift::GenerateQuestionResponse;
use anki_proto::brainlift::Question;
use anki_proto::brainlift::QuestionAttribution;

use crate::collection::Collection;
use crate::error::Result;
use crate::gre_atlas::gre_atlas_storage;
use crate::gre_atlas::questions::ai_gen::generate_question_for_topic;
use crate::gre_atlas::questions::ai_gen::GenerationOutcome;
use crate::gre_atlas::storage::StoredQuestion;
use crate::timestamp::TimestampSecs;

/// Map a stored question row to the public protobuf (never includes answers).
pub(crate) fn stored_question_to_proto(q: StoredQuestion) -> Question {
    let attribution = match (
        q.source_name,
        q.source_section,
        q.generated_at_secs,
        q.generation_confidence,
    ) {
        (Some(source_name), Some(source_section), Some(generated_at_secs), confidence) => {
            Some(QuestionAttribution {
                source_name,
                source_section,
                generated_at_secs,
                confidence,
            })
        }
        _ => None,
    };
    Question {
        id: q.id,
        topic: q.topic,
        section: q.section,
        format: q.format,
        stem: q.stem,
        choices: q.choices,
        attribution,
    }
}

impl Collection {
    pub fn gre_atlas_generate_question(
        &mut self,
        topic_id: &str,
        persist: bool,
    ) -> Result<GenerateQuestionResponse> {
        let now = TimestampSecs::now();
        let outcome = generate_question_for_topic(topic_id, now);
        match outcome {
            GenerationOutcome::Accepted(draft) => {
                if persist {
                    let storage = gre_atlas_storage(self)?;
                    storage.insert_generated_question(&draft)?;
                }
                Ok(GenerateQuestionResponse {
                    accepted: true,
                    confidence: draft.confidence,
                    rejection_reason: String::new(),
                    question: Some(stored_question_to_proto(StoredQuestion {
                        id: draft.id.clone(),
                        topic: draft.topic.clone(),
                        section: draft.section.clone(),
                        format: draft.format.clone(),
                        stem: draft.stem.clone(),
                        choices: draft.choices.clone(),
                        correct_answer: draft.correct_answer.clone(),
                        explanation: draft.explanation.clone(),
                        difficulty: draft.difficulty,
                        source_name: Some(draft.attribution.source_name.clone()),
                        source_section: Some(draft.attribution.source_section.clone()),
                        generated_at_secs: Some(draft.attribution.generated_at_secs),
                        generation_confidence: Some(draft.confidence),
                    })),
                    attribution: Some(QuestionAttribution {
                        source_name: draft.attribution.source_name,
                        source_section: draft.attribution.source_section,
                        generated_at_secs: draft.attribution.generated_at_secs,
                        confidence: Some(draft.confidence),
                    }),
                })
            }
            GenerationOutcome::Rejected {
                confidence,
                reason,
                attribution,
            } => Ok(GenerateQuestionResponse {
                accepted: false,
                confidence,
                rejection_reason: reason,
                question: None,
                attribution: Some(QuestionAttribution {
                    source_name: attribution.source_name,
                    source_section: attribution.source_section,
                    generated_at_secs: attribution.generated_at_secs,
                    confidence: Some(confidence),
                }),
            }),
        }
    }
}

/// Whether `topic` equals `prefix` or is a descendant in the GRE tag hierarchy.
pub fn topic_matches_prefix(topic: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }
    topic == prefix
        || topic
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with("::"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn topic_prefix_matching() {
        assert!(topic_matches_prefix(
            "gre::quant::algebra::linear",
            "gre::quant"
        ));
        assert!(topic_matches_prefix(
            "gre::quant::algebra::linear",
            "gre::quant::algebra::linear"
        ));
        assert!(!topic_matches_prefix(
            "gre::quant::algebra::linear",
            "gre::verbal"
        ));
        assert!(!topic_matches_prefix(
            "gre::quant::algebra",
            "gre::quant::arithmetic"
        ));
        assert!(topic_matches_prefix("gre::quant", ""));
    }

    #[test]
    fn generate_question_persists_attribution() -> Result<()> {
        use crate::collection::CollectionBuilder;

        let dir = tempfile::tempdir().map_err(|e| crate::error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        let col_path = dir.path().join("test.anki2");
        let mut col = CollectionBuilder::new(&col_path).build()?;
        let resp = col.gre_atlas_generate_question("gre::quant::algebra::linear", true)?;
        assert!(resp.accepted);
        let attr = resp.attribution.unwrap();
        assert!(!attr.source_name.is_empty());
        assert!(!attr.source_section.is_empty());
        assert!(attr.generated_at_secs > 0);

        let storage = gre_atlas_storage(&mut col)?;
        let stored = storage.get_question(&resp.question.unwrap().id)?.unwrap();
        assert_eq!(
            stored.source_name.as_deref(),
            Some(attr.source_name.as_str())
        );
        assert_eq!(
            stored.source_section.as_deref(),
            Some(attr.source_section.as_str())
        );
        assert_eq!(stored.generated_at_secs, Some(attr.generated_at_secs));
        Ok(())
    }
}
