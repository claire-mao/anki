// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

mod scores;
mod service;
pub mod storage;

use crate::brainlift::storage::BrainliftStorage;
use crate::collection::Collection;
use crate::error::Result;

pub const GRE_DECK_NAME: &str = "BrainLift GRE";
pub const TOPIC_TAG_PREFIX: &str = "gre::";

pub(crate) fn brainlift_storage(col: &mut Collection) -> Result<&mut BrainliftStorage> {
    if col.state.brainlift.is_none() {
        col.state.brainlift = Some(BrainliftStorage::open(&col.col_path)?);
    }
    Ok(col.state.brainlift.as_mut().unwrap())
}
