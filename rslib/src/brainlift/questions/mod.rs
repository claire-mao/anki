// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::Question;

use crate::brainlift::storage::StoredQuestion;

/// Map a stored question row to the public protobuf (never includes answers).
pub(crate) fn stored_question_to_proto(q: StoredQuestion) -> Question {
    Question {
        id: q.id,
        topic: q.topic,
        section: q.section,
        format: q.format,
        stem: q.stem,
        choices: q.choices,
    }
}

/// Whether `topic` equals `prefix` or is a descendant in the GRE tag hierarchy.
pub fn topic_matches_prefix(topic: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }
    topic == prefix || topic.strip_prefix(prefix).is_some_and(|rest| rest.starts_with("::"))
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
        assert!(!topic_matches_prefix("gre::quant::algebra", "gre::quant::arithmetic"));
        assert!(topic_matches_prefix("gre::quant", ""));
    }
}
