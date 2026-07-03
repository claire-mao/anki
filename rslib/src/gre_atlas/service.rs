// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::CreateSessionResponse;
use anki_proto::brainlift::DashboardState;
use anki_proto::brainlift::GetRecentAttemptsResponse;
use anki_proto::brainlift::GreStudyStatusResponse;
use anki_proto::brainlift::ListQuestionsResponse;
use anki_proto::brainlift::RecordAttemptResponse;
use anki_proto::brainlift::StudyPlanResponse;
use anki_proto::brainlift::TopicDetailsResponse;

use crate::collection::Collection;
use crate::error;
use crate::error::OrInvalid;
use crate::gre_atlas::gre_atlas_storage;
use crate::gre_atlas::questions::stored_question_to_proto;
use crate::gre_atlas::GRE_DECK_NAME;

impl crate::services::BrainLiftService for Collection {
    fn list_questions(
        &mut self,
        input: anki_proto::brainlift::ListQuestionsRequest,
    ) -> error::Result<ListQuestionsResponse> {
        let limit = if input.limit == 0 { 10 } else { input.limit };
        let storage = gre_atlas_storage(self)?;
        let questions = storage
            .list_questions(&input.topic_prefix, limit)?
            .into_iter()
            .map(stored_question_to_proto)
            .collect();
        Ok(ListQuestionsResponse { questions })
    }

    fn get_question(
        &mut self,
        input: anki_proto::brainlift::GetQuestionRequest,
    ) -> error::Result<anki_proto::brainlift::Question> {
        let storage = gre_atlas_storage(self)?;
        let question = storage
            .get_question(&input.question_id)?
            .or_invalid("question not found")?;
        Ok(stored_question_to_proto(question))
    }

    fn create_session(
        &mut self,
        input: anki_proto::brainlift::CreateSessionRequest,
    ) -> error::Result<CreateSessionResponse> {
        let storage = gre_atlas_storage(self)?;
        let session = storage.create_session(&input.source)?;
        Ok(CreateSessionResponse {
            session_id: session.id,
            started_at_secs: session.started_at_secs.0,
            source: session.source,
        })
    }

    fn record_attempt(
        &mut self,
        input: anki_proto::brainlift::RecordAttemptRequest,
    ) -> error::Result<RecordAttemptResponse> {
        let storage = gre_atlas_storage(self)?;
        let question = storage
            .get_question(&input.question_id)?
            .or_invalid("question not found")?;
        let correct = question.correct_answer.trim() == input.answer.trim();
        storage.record_attempt(
            &question.id,
            &question.topic,
            question.difficulty,
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
        self.gre_atlas_get_scores()
    }

    fn get_dashboard(
        &mut self,
        input: anki_proto::brainlift::GetDashboardRequest,
    ) -> error::Result<DashboardState> {
        self.gre_atlas_get_dashboard(input)
    }

    fn get_recent_attempts(
        &mut self,
        input: anki_proto::brainlift::GetRecentAttemptsRequest,
    ) -> error::Result<GetRecentAttemptsResponse> {
        let limit = if input.limit == 0 { 10 } else { input.limit };
        let storage = gre_atlas_storage(self)?;
        let attempts = storage
            .recent_attempts(&input.topic_prefix, limit)?
            .into_iter()
            .map(|a| crate::gre_atlas::signals::attempt_row_to_proto(&a))
            .collect();
        Ok(GetRecentAttemptsResponse { attempts })
    }

    fn get_gre_study_status(&mut self) -> error::Result<GreStudyStatusResponse> {
        let deck_id = crate::gre_atlas::gre_deck_id(self)?;
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

    fn get_study_plan(
        &mut self,
        input: anki_proto::brainlift::GetStudyPlanRequest,
    ) -> error::Result<StudyPlanResponse> {
        self.gre_atlas_get_study_plan(input)
    }

    fn get_readiness_calibration(
        &mut self,
    ) -> error::Result<anki_proto::brainlift::ReadinessCalibrationResponse> {
        self.gre_atlas_get_readiness_calibration()
    }

    fn generate_brain_lift_eval_report(
        &mut self,
    ) -> error::Result<anki_proto::brainlift::BrainLiftEvalReportResponse> {
        let (json, markdown, performance_markdown) = self.gre_atlas_generate_eval_report()?;
        Ok(anki_proto::brainlift::BrainLiftEvalReportResponse {
            json,
            markdown,
            performance_markdown,
        })
    }

    fn generate_brain_lift_ai_eval_report(
        &mut self,
    ) -> error::Result<anki_proto::brainlift::BrainLiftAiEvalReportResponse> {
        let (json, markdown) = self.gre_atlas_generate_ai_eval_report()?;
        Ok(anki_proto::brainlift::BrainLiftAiEvalReportResponse { json, markdown })
    }

    fn generate_question(
        &mut self,
        input: anki_proto::brainlift::GenerateQuestionRequest,
    ) -> error::Result<anki_proto::brainlift::GenerateQuestionResponse> {
        self.gre_atlas_generate_question(&input.topic_id, input.persist)
    }

    fn get_topic_details(
        &mut self,
        input: anki_proto::brainlift::GetTopicDetailsRequest,
    ) -> error::Result<TopicDetailsResponse> {
        self.gre_atlas_get_topic_details(input)
    }

    fn get_brain_lift_sync_status(
        &mut self,
    ) -> error::Result<anki_proto::brainlift::BrainLiftSyncStatus> {
        self.gre_atlas_sync_status()
    }

    fn pull_brain_lift_changes(
        &mut self,
        input: anki_proto::brainlift::BrainLiftSyncPullRequest,
    ) -> error::Result<anki_proto::brainlift::BrainLiftSyncPullResponse> {
        let limit = if input.limit == 0 { 100 } else { input.limit };
        self.gre_atlas_pull_changes(input.after_usn, limit)
    }

    fn push_brain_lift_changes(
        &mut self,
        input: anki_proto::brainlift::BrainLiftSyncPushRequest,
    ) -> error::Result<anki_proto::brainlift::BrainLiftSyncPushResponse> {
        self.gre_atlas_push_changes(input.attempts)
    }

    fn prepare_demo_collection(
        &mut self,
    ) -> error::Result<anki_proto::brainlift::PrepareDemoCollectionResponse> {
        self.gre_atlas_prepare_demo_collection()
    }
}
