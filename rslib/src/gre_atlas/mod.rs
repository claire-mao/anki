// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

mod ablation_eval;
pub(crate) mod abstention;
mod ai_eval;
mod calibration;
mod coverage_report;
mod dashboard;
mod demo;
pub mod domain;
mod estimated_gre;
mod eval;
mod memory_eval;
mod performance_eval;
pub mod questions;
mod readiness;
mod scores;
mod service;
pub(crate) mod signals;
pub mod storage;
mod study_plan;
mod sync;
mod topic_details;
mod topic_insights;

use std::sync::LazyLock;

pub use domain::compute_coverage;
pub use domain::is_topic_covered;
pub use domain::GreCatalog;
pub use domain::GreCoverage;
pub use domain::GreSection;
pub use domain::TopicDef;
pub use domain::TOPIC_TAG_PREFIX;

use crate::collection::Collection;
use crate::error::Result;
use crate::gre_atlas::storage::GreAtlasStorage;

pub const GRE_DECK_NAME: &str = "GRE Atlas";
/// Previous product deck name; kept for collection compatibility.
pub const LEGACY_GRE_DECK_NAME: &str = "BrainLift GRE";

pub(crate) fn gre_deck_id(col: &Collection) -> Result<Option<crate::decks::DeckId>> {
    if let Some(id) = col.get_deck_id(GRE_DECK_NAME)? {
        return Ok(Some(id));
    }
    col.get_deck_id(LEGACY_GRE_DECK_NAME)
}

static GRE_DECK_SEARCH: LazyLock<String> = LazyLock::new(|| {
    format!(
        r#"deck:"{}" OR deck:"{}""#,
        GRE_DECK_NAME, LEGACY_GRE_DECK_NAME
    )
});

pub(crate) fn gre_deck_search_str() -> &'static str {
    GRE_DECK_SEARCH.as_str()
}

pub fn gre_deck_search() -> String {
    GRE_DECK_SEARCH.clone()
}

pub(crate) fn gre_atlas_storage(col: &mut Collection) -> Result<&mut GreAtlasStorage> {
    if col.state.gre_atlas.is_none() {
        col.state.gre_atlas = Some(GreAtlasStorage::open(&col.col_path)?);
    }
    Ok(col.state.gre_atlas.as_mut().unwrap())
}
