// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::GetRecentAttemptsResponse;
use anki_proto::brainlift::GreStudyStatusResponse;
use anki_proto::brainlift::ListQuestionsResponse;
use anki_proto::brainlift::PerformanceAttempt;
use anki_proto::brainlift::Question;
use anki_proto::brainlift::RecordAttemptResponse;

use crate::brainlift::brainlift_storage;
use crate::brainlift::GRE_DECK_NAME;
use crate::collection::Collection;
use crate::error;
use crate::error::OrInvalid;

impl crate::services::BrainLiftService for Collection {
    fn list_questions(
        &mut self,
        input: anki_proto::brainlift::ListQuestionsRequest,
    ) -> error::Result<ListQuestionsResponse> {
        let limit = if input.limit == 0 { 1 } else { input.limit };
        let storage = brainlift_storage(self)?;
        let questions = storage
            .list_questions(limit)?
            .into_iter()
            .map(|q| Question {
                id: q.id,
                topic: q.topic,
                section: q.section,
                format: q.format,
                stem: q.stem,
                choices: q.choices,
            })
            .collect();
        Ok(ListQuestionsResponse { questions })
    }

    fn record_attempt(
        &mut self,
        input: anki_proto::brainlift::RecordAttemptRequest,
    ) -> error::Result<RecordAttemptResponse> {
        let storage = brainlift_storage(self)?;
        let question = storage
            .get_question(&input.question_id)?
            .or_invalid("question not found")?;
        let correct = question.correct_answer.trim() == input.answer.trim();
        storage.record_attempt(
            &question.id,
            &question.topic,
            &input.answer,
            correct,
            input.response_time_ms,
            input.confidence,
            input.session_id.as_deref(),
        )?;
        Ok(RecordAttemptResponse {
            correct,
            explanation: question.explanation,
            topic: question.topic,
        })
    }

    fn get_scores(&mut self) -> error::Result<anki_proto::brainlift::GetScoresResponse> {
        self.brainlift_get_scores()
    }

    fn get_recent_attempts(
        &mut self,
        input: anki_proto::brainlift::GetRecentAttemptsRequest,
    ) -> error::Result<GetRecentAttemptsResponse> {
        let limit = if input.limit == 0 { 10 } else { input.limit };
        let storage = brainlift_storage(self)?;
        let attempts = storage
            .recent_attempts(limit)?
            .into_iter()
            .map(|a| PerformanceAttempt {
                question_id: a.question_id,
                topic: a.topic,
                answered_at_secs: a.answered_at_secs.0,
                answer: a.answer,
                correct: a.correct,
                response_time_ms: a.response_time_ms,
                confidence: a.confidence,
            })
            .collect();
        Ok(GetRecentAttemptsResponse { attempts })
    }

    fn get_gre_study_status(&mut self) -> error::Result<GreStudyStatusResponse> {
        let deck_id = self.get_deck_id(GRE_DECK_NAME)?;
        let deck_exists = deck_id.is_some();
        let mut new_count = 0;
        let mut learn_count = 0;
        let mut review_count = 0;
        if let Some(did) = deck_id {
            let timing = self.timing_today()?;
            let learn_cutoff = timing.now.0 as u32 + self.learn_ahead_secs();
            let counts_map = self.due_counts(timing.days_elapsed, learn_cutoff)?;
            if let Some(counts) = counts_map.get(&did) {
                new_count = counts.new;
                learn_count = counts.learning;
                review_count = counts.review;
            }
        }
        Ok(GreStudyStatusResponse {
            deck_name: GRE_DECK_NAME.into(),
            deck_exists,
            new_count,
            learn_count,
            review_count,
        })
    }
}
