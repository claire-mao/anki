// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Shared value types for AI vs offline-template question provenance and
//! evaluation status. These are deliberately dependency-free so the storage,
//! generator, eval-pipeline, and explanation layers can all reference them
//! without cycles.

/// Provenance flag distinguishing AI-generated questions from the deterministic
/// offline-template fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// Produced by the real, env-gated LLM path (and passed the eval gate).
    AiGenerated,
    /// Produced by the always-available deterministic template path.
    OfflineTemplate,
}

/// Stored as `bl_question.provenance` for real LLM output.
pub const PROVENANCE_AI: &str = "ai_generated";
/// Stored as `bl_question.provenance` for the deterministic template fallback.
pub const PROVENANCE_TEMPLATE: &str = "offline_template";

/// Exact user-facing note shown whenever the deterministic fallback is used.
/// Wording is contractual with the UI layer; do not change casually.
pub const OFFLINE_TEMPLATE_NOTE: &str = "Generated using offline templates.";

impl Provenance {
    pub fn as_str(self) -> &'static str {
        match self {
            Provenance::AiGenerated => PROVENANCE_AI,
            Provenance::OfflineTemplate => PROVENANCE_TEMPLATE,
        }
    }

    pub fn is_ai(self) -> bool {
        matches!(self, Provenance::AiGenerated)
    }
}

/// Outcome of the pre-exposure evaluation gate for a generated question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationStatus {
    /// Not yet evaluated.
    Pending,
    /// Passed every gate; safe to expose to learners.
    Approved,
    /// Answer not derivable / not among choices / malformed.
    RejectedHallucination,
    /// Near-duplicate of an existing bank question.
    RejectedDuplicate,
    /// Claims not grounded in the provided source material.
    RejectedUnsupported,
}

impl EvaluationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            EvaluationStatus::Pending => "pending",
            EvaluationStatus::Approved => "approved",
            EvaluationStatus::RejectedHallucination => "rejected_hallucination",
            EvaluationStatus::RejectedDuplicate => "rejected_duplicate",
            EvaluationStatus::RejectedUnsupported => "rejected_unsupported",
        }
    }

    pub fn is_approved(self) -> bool {
        matches!(self, EvaluationStatus::Approved)
    }

    pub fn is_rejected(self) -> bool {
        matches!(
            self,
            EvaluationStatus::RejectedHallucination
                | EvaluationStatus::RejectedDuplicate
                | EvaluationStatus::RejectedUnsupported
        )
    }
}

/// Metadata persisted alongside a generated question in `bl_question`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionMetadata {
    pub provenance: Provenance,
    pub model_version: String,
    pub source_document: String,
    pub evaluation_status: EvaluationStatus,
}

impl QuestionMetadata {
    /// Default metadata for a deterministic template question (the fallback the
    /// bank top-up uses on every DB open).
    pub fn offline_template(model_version: &str, source_document: impl Into<String>) -> Self {
        QuestionMetadata {
            provenance: Provenance::OfflineTemplate,
            model_version: model_version.to_string(),
            source_document: source_document.into(),
            evaluation_status: EvaluationStatus::Approved,
        }
    }
}
