// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki::backend::Backend;
use anki_proto::brainlift::PrepareDemoCollectionResponse;
use anki_proto::generic::Empty;
use prost::Message;

use crate::backend_method::invoke_proto;

const GRE_ATLAS_SERVICE: &str = "BackendBrainLiftService";

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreDemoCollectionView {
    pub deck_name: String,
    pub deck_created: bool,
    pub cards_added: u32,
    pub practice_attempts_added: u32,
    pub due_new: u32,
    pub due_learn: u32,
    pub due_review: u32,
    pub due_total: u32,
}

impl From<PrepareDemoCollectionResponse> for GreDemoCollectionView {
    fn from(response: PrepareDemoCollectionResponse) -> Self {
        Self {
            deck_name: response.deck_name,
            deck_created: response.deck_created,
            cards_added: response.cards_added,
            practice_attempts_added: response.practice_attempts_added,
            due_new: response.due_new,
            due_learn: response.due_learn,
            due_review: response.due_review,
            due_total: response.due_new + response.due_learn + response.due_review,
        }
    }
}

pub fn prepare_demo_collection(backend: &Backend) -> Result<GreDemoCollectionView, Vec<u8>> {
    let response = invoke_proto::<PrepareDemoCollectionResponse>(
        backend,
        GRE_ATLAS_SERVICE,
        "prepare_demo_collection",
        &Empty::default().encode_to_vec(),
    )?;
    Ok(response.into())
}
