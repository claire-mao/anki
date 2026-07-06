// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki::backend::Backend;
use anki_proto::brainlift::GreStudyStatusResponse;
use anki_proto::card_rendering::RenderCardResponse;
use anki_proto::card_rendering::RenderExistingCardRequest;
use anki_proto::card_rendering::RenderedTemplateNode;
use anki_proto::cards::CardId as ProtoCardId;
use anki_proto::decks::DeckId;
use anki_proto::generic::Empty;
use anki_proto::generic::String as ProtoString;
use anki_proto::generic::StringList;
use anki_proto::scheduler::card_answer::Rating;
use anki_proto::scheduler::queued_cards::Queue;
use anki_proto::scheduler::CardAnswer;
use anki_proto::scheduler::GetQueuedCardsRequest;
use anki_proto::scheduler::QueuedCards;
use anki_proto::scheduler::SchedulingStates;
use prost::Message;
use serde::Deserialize;
use serde::Serialize;

use crate::backend_method::invoke;
use crate::backend_method::invoke_proto;

const GRE_DECK_NAME: &str = "GRE Atlas";

const GRE_ATLAS_SERVICE: &str = "BackendBrainLiftService";
const DECKS_SERVICE: &str = "DecksService";
const SCHEDULER_SERVICE: &str = "SchedulerService";
const CARD_RENDERING_SERVICE: &str = "CardRenderingService";

const REVIEW_FETCH_LIMIT: u32 = 1;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreStudyGradeButtonView {
    pub rating: u32,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreStudyCardView {
    pub card_id: i64,
    pub queue: String,
    pub question_html: String,
    pub answer_html: String,
    pub css: String,
    pub buttons: Vec<GreStudyGradeButtonView>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GreStudyReviewView {
    pub deck_exists: bool,
    pub deck_name: String,
    pub due_new: u32,
    pub due_learn: u32,
    pub due_review: u32,
    pub due_total: u32,
    pub session_active: bool,
    pub session_complete: bool,
    pub card: Option<GreStudyCardView>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GreStudyAnswerInput {
    pub card_id: i64,
    pub rating: u32,
    pub milliseconds_taken: u32,
}

pub fn load_study_review(backend: &Backend) -> Result<GreStudyReviewView, Vec<u8>> {
    load_study_review_inner(backend, false)
}

pub fn load_study_extra_review(backend: &Backend) -> Result<GreStudyReviewView, Vec<u8>> {
    load_study_review_inner(backend, true)
}

fn load_study_review_inner(
    backend: &Backend,
    unlock_extra: bool,
) -> Result<GreStudyReviewView, Vec<u8>> {
    let status = fetch_gre_study_status(backend)?;
    if !status.deck_exists {
        return Ok(GreStudyReviewView::idle(status, false, true));
    }
    ensure_gre_deck_selected(backend)?;
    if unlock_extra {
        invoke(
            backend,
            GRE_ATLAS_SERVICE,
            "start_gre_extra_study",
            &Empty::default().encode_to_vec(),
        )?;
    }
    let status = if unlock_extra {
        fetch_gre_study_status(backend)?
    } else {
        status
    };
    build_study_review_view(backend, status, true)
}

pub fn answer_study_card(
    backend: &Backend,
    input: GreStudyAnswerInput,
) -> Result<GreStudyReviewView, Vec<u8>> {
    let status = fetch_gre_study_status(backend)?;
    if !status.deck_exists {
        return Ok(GreStudyReviewView::idle(status, false, true));
    }
    ensure_gre_deck_selected(backend)?;

    let rating = rating_from_u32(input.rating)?;
    let states = invoke_proto::<SchedulingStates>(
        backend,
        SCHEDULER_SERVICE,
        "get_scheduling_states",
        &ProtoCardId { cid: input.card_id }.encode_to_vec(),
    )?;
    let new_state = new_state_for_rating(&states, rating);
    let answered_at_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0);
    invoke(
        backend,
        SCHEDULER_SERVICE,
        "answer_card",
        &CardAnswer {
            card_id: input.card_id,
            current_state: states.current,
            new_state,
            rating: rating as i32,
            answered_at_millis,
            milliseconds_taken: input.milliseconds_taken,
        }
        .encode_to_vec(),
    )?;

    build_study_review_view(backend, status, true)
}

fn fetch_gre_study_status(backend: &Backend) -> Result<GreStudyStatusResponse, Vec<u8>> {
    invoke_proto(
        backend,
        GRE_ATLAS_SERVICE,
        "get_gre_study_status",
        &Empty::default().encode_to_vec(),
    )
}

fn ensure_gre_deck_selected(backend: &Backend) -> Result<(), Vec<u8>> {
    let deck = invoke_proto::<DeckId>(
        backend,
        DECKS_SERVICE,
        "get_deck_id_by_name",
        &ProtoString {
            val: GRE_DECK_NAME.into(),
        }
        .encode_to_vec(),
    )?;
    invoke(
        backend,
        DECKS_SERVICE,
        "set_current_deck",
        &deck.encode_to_vec(),
    )?;
    Ok(())
}

fn fetch_queued_cards(backend: &Backend) -> Result<QueuedCards, Vec<u8>> {
    invoke_proto(
        backend,
        SCHEDULER_SERVICE,
        "get_queued_cards",
        &GetQueuedCardsRequest {
            fetch_limit: REVIEW_FETCH_LIMIT,
            intraday_learning_only: false,
        }
        .encode_to_vec(),
    )
}

fn build_study_review_view(
    backend: &Backend,
    status: GreStudyStatusResponse,
    session_active: bool,
) -> Result<GreStudyReviewView, Vec<u8>> {
    let queued = fetch_queued_cards(backend)?;
    let due_total = queued.new_count + queued.learning_count + queued.review_count;
    let Some(top) = queued.cards.first() else {
        return Ok(GreStudyReviewView {
            deck_exists: status.deck_exists,
            deck_name: status.deck_name,
            due_new: queued.new_count,
            due_learn: queued.learning_count,
            due_review: queued.review_count,
            due_total,
            session_active,
            session_complete: session_active,
            card: None,
        });
    };

    let card = top
        .card
        .clone()
        .ok_or_else(|| b"missing card in queued card response".to_vec())?;
    let states = top
        .states
        .clone()
        .ok_or_else(|| b"missing scheduling states".to_vec())?;
    let rendered = invoke_proto::<RenderCardResponse>(
        backend,
        CARD_RENDERING_SERVICE,
        "render_existing_card",
        &RenderExistingCardRequest {
            card_id: card.id,
            browser: false,
            partial_render: false,
        }
        .encode_to_vec(),
    )?;
    let labels = invoke_proto::<StringList>(
        backend,
        SCHEDULER_SERVICE,
        "describe_next_states",
        &states.encode_to_vec(),
    )?;
    let buttons = grade_buttons(&labels.vals);
    Ok(GreStudyReviewView {
        deck_exists: status.deck_exists,
        deck_name: status.deck_name,
        due_new: queued.new_count,
        due_learn: queued.learning_count,
        due_review: queued.review_count,
        due_total,
        session_active,
        session_complete: false,
        card: Some(GreStudyCardView {
            card_id: card.id,
            queue: queue_label(top.queue),
            question_html: nodes_to_html(&rendered.question_nodes),
            answer_html: nodes_to_html(&rendered.answer_nodes),
            css: rendered.css,
            buttons,
        }),
    })
}

impl GreStudyReviewView {
    fn idle(status: GreStudyStatusResponse, session_active: bool, session_complete: bool) -> Self {
        let due_total = status.new_count + status.learn_count + status.review_count;
        Self {
            deck_exists: status.deck_exists,
            deck_name: status.deck_name,
            due_new: status.new_count,
            due_learn: status.learn_count,
            due_review: status.review_count,
            due_total,
            session_active,
            session_complete,
            card: None,
        }
    }
}

fn queue_label(queue: i32) -> String {
    match Queue::try_from(queue).unwrap_or(Queue::Review) {
        Queue::New => "new".into(),
        Queue::Learning => "learning".into(),
        Queue::Review => "review".into(),
    }
}

fn grade_buttons(labels: &[String]) -> Vec<GreStudyGradeButtonView> {
    labels
        .iter()
        .enumerate()
        .map(|(index, label)| GreStudyGradeButtonView {
            rating: index as u32,
            label: label.clone(),
        })
        .collect()
}

fn nodes_to_html(nodes: &[RenderedTemplateNode]) -> String {
    nodes
        .iter()
        .filter_map(|node| match &node.value {
            Some(anki_proto::card_rendering::rendered_template_node::Value::Text(text)) => {
                Some(text.as_str())
            }
            _ => None,
        })
        .collect()
}

fn rating_from_u32(value: u32) -> Result<Rating, Vec<u8>> {
    match value {
        0 => Ok(Rating::Again),
        1 => Ok(Rating::Hard),
        2 => Ok(Rating::Good),
        3 => Ok(Rating::Easy),
        _ => Err(b"invalid study rating".to_vec()),
    }
}

fn new_state_for_rating(
    states: &SchedulingStates,
    rating: Rating,
) -> Option<anki_proto::scheduler::SchedulingState> {
    match rating {
        Rating::Again => states.again.clone(),
        Rating::Hard => states.hard.clone(),
        Rating::Good => states.good.clone(),
        Rating::Easy => states.easy.clone(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn grade_buttons_follow_scheduler_labels() {
        let buttons = grade_buttons(&["1m".into(), "6m".into(), "10m".into(), "4d".into()]);
        assert_eq!(buttons.len(), 4);
        assert_eq!(buttons[0].rating, 0);
        assert_eq!(buttons[0].label, "1m");
    }
}
