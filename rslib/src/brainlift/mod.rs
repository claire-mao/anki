// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

pub(crate) mod abstention;
mod calibration;
mod dashboard;
mod estimated_gre;
mod readiness;
mod scores;
mod service;
mod signals;
mod study_plan;
mod sync;
mod topic_details;
mod topic_insights;
pub mod domain;
pub mod questions;
pub mod storage;

use crate::brainlift::storage::BrainliftStorage;
use crate::collection::Collection;
use crate::error::Result;

pub use domain::compute_coverage;
pub use domain::is_topic_covered;
pub use domain::GreCatalog;
pub use domain::GreCoverage;
pub use domain::GreSection;
pub use domain::TopicDef;
pub use domain::TOPIC_TAG_PREFIX;

pub const GRE_DECK_NAME: &str = "BrainLift GRE";

pub(crate) fn brainlift_storage(col: &mut Collection) -> Result<&mut BrainliftStorage> {
    if col.state.brainlift.is_none() {
        col.state.brainlift = Some(BrainliftStorage::open(&col.col_path)?);
    }
    Ok(col.state.brainlift.as_mut().unwrap())
}
