// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use anki_proto::brainlift::CreateSessionResponse;
use anki_proto::brainlift::DashboardState;
use anki_proto::brainlift::GetPerformanceChartResponse;
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
use crate::sync::login::SyncAuth;

impl crate::services::BrainLiftService for Collection {
    fn list_questions(
        &mut self,
        input: anki_proto::brainlift::ListQuestionsRequest,
    ) -> error::Result<ListQuestionsResponse> {
        let limit = if input.limit == 0 { 10 } else { input.limit };
        let storage = gre_atlas_storage(self)?;
        let questions = storage
            .list_practice_bank_questions(&input.topic_prefix, limit)?
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
        crate::gre_atlas::topic_flashcard_release::gre_atlas_on_practice_attempt(
            self,
            &question.topic,
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

    fn get_performance_chart(
        &mut self,
        input: anki_proto::brainlift::GetPerformanceChartRequest,
    ) -> error::Result<GetPerformanceChartResponse> {
        self.gre_atlas_get_performance_chart(input)
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
        let status = GreStudyStatusResponse {
            deck_name: GRE_DECK_NAME.into(),
            deck_exists,
            new_count,
            learn_count,
            review_count,
            ..Default::default()
        };
        crate::gre_atlas::topic_flashcard_release::gre_atlas_process_flashcard_schedule(self)?;
        crate::gre_atlas::extra_study::enrich_gre_study_status(self, status)
    }

    fn start_gre_extra_study(
        &mut self,
    ) -> error::Result<anki_proto::brainlift::StartGreExtraStudyResponse> {
        crate::gre_atlas::extra_study::gre_atlas_start_extra_study(self)
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

    fn get_performance_eval(
        &mut self,
    ) -> error::Result<anki_proto::brainlift::PerformanceEvalResponse> {
        self.gre_atlas_get_performance_eval()
    }

    fn get_memory_eval(
        &mut self,
    ) -> error::Result<anki_proto::brainlift::MemoryEvalResponse> {
        self.gre_atlas_get_memory_eval()
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
        use crate::gre_atlas::questions::llm::GreAtlasAiConfig;

        let (json, markdown) = self.gre_atlas_generate_ai_eval_report()?;
        Ok(anki_proto::brainlift::BrainLiftAiEvalReportResponse {
            json,
            markdown,
            ai_enabled: GreAtlasAiConfig::from_env().is_some(),
        })
    }

    fn generate_question(
        &mut self,
        input: anki_proto::brainlift::GenerateQuestionRequest,
    ) -> error::Result<anki_proto::brainlift::GenerateQuestionResponse> {
        self.gre_atlas_generate_question(&input.topic_id, input.persist)
    }

    fn explain_answer(
        &mut self,
        input: anki_proto::brainlift::ExplainAnswerRequest,
    ) -> error::Result<anki_proto::brainlift::ExplainAnswerResponse> {
        self.gre_atlas_explain_answer(&input.question_id, &input.selected_answer)
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

    fn pull_brain_lift_sync_bundle(
        &mut self,
        input: anki_proto::brainlift::BrainLiftSyncPullRequest,
    ) -> error::Result<anki_proto::brainlift::BrainLiftSyncBundleResponse> {
        let limit = if input.limit == 0 { 5000 } else { input.limit };
        self.gre_atlas_pull_sync_bundle(input.after_usn, limit)
    }

    fn push_brain_lift_sync_bundle(
        &mut self,
        input: anki_proto::brainlift::BrainLiftSyncBundlePushRequest,
    ) -> error::Result<anki_proto::brainlift::BrainLiftSyncBundlePushResponse> {
        let bundle = input.bundle.or_invalid("missing sync bundle")?;
        self.gre_atlas_push_sync_bundle(bundle)
    }

    fn perform_gre_atlas_sync(
        &mut self,
        input: anki_proto::brainlift::PerformGreAtlasSyncRequest,
    ) -> error::Result<anki_proto::brainlift::PerformGreAtlasSyncResponse> {
        let Some(auth_proto) = input.auth else {
            return self.gre_atlas_perform_sync_offline();
        };
        let auth = SyncAuth {
            hkey: auth_proto.hkey,
            endpoint: auth_proto
                .endpoint
                .and_then(|e| reqwest::Url::parse(&e).ok()),
            io_timeout_secs: auth_proto.io_timeout_secs,
        };
        let client = reqwest::Client::new();
        let rt = tokio::runtime::Runtime::new().map_err(|e| error::AnkiError::InvalidInput {
            source: snafu::FromString::without_source(e.to_string()),
        })?;
        rt.block_on(self.gre_atlas_perform_sync(auth, client))
    }

    fn prepare_demo_collection(
        &mut self,
    ) -> error::Result<anki_proto::brainlift::PrepareDemoCollectionResponse> {
        self.gre_atlas_prepare_demo_collection()
    }

    fn get_gre_atlas_verification(
        &mut self,
        input: anki_proto::brainlift::GetGreAtlasVerificationRequest,
    ) -> error::Result<anki_proto::brainlift::GreAtlasVerificationResponse> {
        self.gre_atlas_get_verification(input.client())
    }

    fn get_gre_atlas_ai_settings(
        &mut self,
    ) -> error::Result<anki_proto::brainlift::GreAtlasAiSettings> {
        Ok(anki_proto::brainlift::GreAtlasAiSettings {
            ai_enabled: crate::gre_atlas::questions::gre_atlas_ai_enabled(self),
            ai_available: crate::gre_atlas::questions::gre_atlas_ai_available(),
        })
    }

    fn set_gre_atlas_ai_enabled(
        &mut self,
        input: anki_proto::brainlift::SetGreAtlasAiEnabledRequest,
    ) -> error::Result<anki_proto::brainlift::GreAtlasAiSettings> {
        self.set_config_json(
            crate::gre_atlas::questions::GRE_ATLAS_AI_ENABLED_KEY,
            &input.enabled,
            true,
        )?;
        Ok(anki_proto::brainlift::GreAtlasAiSettings {
            ai_enabled: input.enabled,
            ai_available: crate::gre_atlas::questions::gre_atlas_ai_available(),
        })
    }
}
