// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Optional, env-gated LLM client for GRE Atlas question generation and
//! explanations.
//!
//! This is the **only** place that performs network I/O for the AI feature.
//! Everything here is off by default: [`GreAtlasAiConfig::from_env`] returns
//! `None` unless `GRE_ATLAS_OPENAI_API_KEY` is set, and the rest of the
//! subsystem uses the deterministic template path in that case.
//!
//! The [`LlmClient`] trait lets tests inject a stub that returns canned
//! completions (or simulated failures) so the default `cargo test` / `just
//! check` never make a network call.

use std::time::Duration;

/// Environment variable holding the API key. Presence of a non-empty value is
/// what enables the real LLM path.
pub const ENV_API_KEY: &str = "GRE_ATLAS_OPENAI_API_KEY";
/// Optional override for the OpenAI-compatible base URL.
pub const ENV_BASE_URL: &str = "GRE_ATLAS_OPENAI_BASE_URL";
/// Optional override for the chat-completions model id.
pub const ENV_MODEL: &str = "GRE_ATLAS_OPENAI_MODEL";
/// Optional override for the request timeout, in seconds.
pub const ENV_TIMEOUT_SECS: &str = "GRE_ATLAS_OPENAI_TIMEOUT_SECS";
/// When set to a truthy value (`1`, `true`, `yes`, `on`), disables the LLM path
/// even if an API key is present.
pub const ENV_AI_DISABLED: &str = "GRE_ATLAS_AI_DISABLED";

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "gpt-4o-mini";
const DEFAULT_TIMEOUT_SECS: u64 = 20;

/// Reason an LLM call did not yield usable output. Callers treat *every*
/// variant as "AI unavailable" and fall back to deterministic templates — no
/// variant is ever surfaced to the learner as an error.
#[derive(Debug, Clone)]
pub enum LlmError {
    /// No API key configured (feature disabled).
    Disabled,
    /// Transport/timeout/connection failure.
    Transport(String),
    /// Non-success HTTP status.
    Status(u16, String),
    /// Response body could not be parsed into the expected shape.
    Parse(String),
    /// Provider returned an empty completion.
    Empty,
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::Disabled => write!(f, "LLM disabled"),
            LlmError::Transport(msg) => write!(f, "LLM transport error: {msg}"),
            LlmError::Status(code, msg) => write!(f, "LLM HTTP {code}: {msg}"),
            LlmError::Parse(msg) => write!(f, "LLM parse error: {msg}"),
            LlmError::Empty => write!(f, "LLM returned empty completion"),
        }
    }
}

/// A single chat completion request (system + user prompt).
#[derive(Debug, Clone)]
pub struct LlmChatRequest {
    pub system: String,
    pub user: String,
    /// Sampling temperature; lower is more deterministic.
    pub temperature: f32,
}

/// Abstraction over the chat-completions call so the network dependency can be
/// stubbed in tests. Implementors must be thread-safe.
pub trait LlmClient: Send + Sync {
    /// Return the raw assistant message content for the given request.
    fn complete(&self, request: &LlmChatRequest) -> Result<String, LlmError>;

    /// Model identifier used for provenance/metadata.
    fn model_version(&self) -> &str;
}

/// Resolved configuration for the real LLM path. Absent unless an API key is
/// set in the environment.
#[derive(Debug, Clone)]
pub struct GreAtlasAiConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub timeout: Duration,
}

impl GreAtlasAiConfig {
    /// Resolve config from the environment. Returns `None` (feature disabled)
    /// unless [`ENV_API_KEY`] is present and non-empty, and [`ENV_AI_DISABLED`]
    /// is not set to a truthy value.
    pub fn from_env() -> Option<Self> {
        if ai_explicitly_disabled() {
            return None;
        }
        let api_key = non_empty_env(ENV_API_KEY)?;
        let base_url = non_empty_env(ENV_BASE_URL).unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let model = non_empty_env(ENV_MODEL).unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let timeout_secs = non_empty_env(ENV_TIMEOUT_SECS)
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_TIMEOUT_SECS);
        Some(GreAtlasAiConfig {
            api_key,
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
            timeout: Duration::from_secs(timeout_secs),
        })
    }
}

fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn ai_explicitly_disabled() -> bool {
    non_empty_env(ENV_AI_DISABLED)
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

/// Real OpenAI-compatible chat-completions client. Constructed only when a key
/// is configured. Performs the HTTP request on an isolated single-thread
/// runtime so it is safe to call from the synchronous RPC path regardless of
/// whether an ambient async runtime exists.
pub struct OpenAiLlmClient {
    config: GreAtlasAiConfig,
}

impl OpenAiLlmClient {
    pub fn new(config: GreAtlasAiConfig) -> Self {
        OpenAiLlmClient { config }
    }
}

impl LlmClient for OpenAiLlmClient {
    fn complete(&self, request: &LlmChatRequest) -> Result<String, LlmError> {
        let config = self.config.clone();
        let request = request.clone();
        block_on_isolated(move || async move { chat_completion(&config, &request).await })
    }

    fn model_version(&self) -> &str {
        &self.config.model
    }
}

async fn chat_completion(
    config: &GreAtlasAiConfig,
    request: &LlmChatRequest,
) -> Result<String, LlmError> {
    let client = reqwest::Client::builder()
        .timeout(config.timeout)
        .build()
        .map_err(|e| LlmError::Transport(e.to_string()))?;

    let body = serde_json::json!({
        "model": config.model,
        "messages": [
            { "role": "system", "content": request.system },
            { "role": "user", "content": request.user },
        ],
        "temperature": request.temperature,
        "response_format": { "type": "json_object" },
    });

    let url = format!("{}/chat/completions", config.base_url);
    let response = client
        .post(url)
        .bearer_auth(&config.api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| LlmError::Transport(e.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        // Body may contain provider error detail, but never the API key.
        let detail = response.text().await.unwrap_or_default();
        let detail: String = detail.chars().take(300).collect();
        return Err(LlmError::Status(status.as_u16(), detail));
    }

    let parsed: ChatCompletionResponse = response
        .json()
        .await
        .map_err(|e| LlmError::Parse(e.to_string()))?;

    parsed
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .filter(|c| !c.trim().is_empty())
        .ok_or(LlmError::Empty)
}

/// Run an async future to completion on a dedicated single-thread runtime in an
/// isolated OS thread. This avoids "cannot start a runtime from within a
/// runtime" panics and keeps the (rare, off-by-default) network call fully
/// self-contained.
fn block_on_isolated<F, Fut, T>(make_future: F) -> Result<T, LlmError>
where
    F: FnOnce() -> Fut + Send,
    Fut: std::future::Future<Output = Result<T, LlmError>>,
    T: Send,
{
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| LlmError::Transport(e.to_string()))?;
                runtime.block_on(make_future())
            })
            .join()
            .map_err(|_| LlmError::Transport("LLM worker thread panicked".into()))?
    })
}

#[derive(serde::Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(serde::Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(serde::Deserialize)]
struct ChatMessage {
    #[serde(default)]
    content: String,
}

#[cfg(test)]
mod test {
    use super::*;

    /// Serialize/deserialize access to `std::env`, which is process-global.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn config_absent_without_api_key() {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev = std::env::var(ENV_API_KEY).ok();
        std::env::remove_var(ENV_API_KEY);
        assert!(GreAtlasAiConfig::from_env().is_none());
        if let Some(prev) = prev {
            std::env::set_var(ENV_API_KEY, prev);
        }
    }

    #[test]
    fn config_present_with_api_key_uses_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev_key = std::env::var(ENV_API_KEY).ok();
        let prev_base = std::env::var(ENV_BASE_URL).ok();
        let prev_model = std::env::var(ENV_MODEL).ok();
        std::env::set_var(ENV_API_KEY, "sk-test");
        std::env::remove_var(ENV_BASE_URL);
        std::env::remove_var(ENV_MODEL);

        let config = GreAtlasAiConfig::from_env().expect("config present");
        assert_eq!(config.model, DEFAULT_MODEL);
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
        assert_eq!(config.api_key, "sk-test");

        // restore
        match prev_key {
            Some(v) => std::env::set_var(ENV_API_KEY, v),
            None => std::env::remove_var(ENV_API_KEY),
        }
        if let Some(v) = prev_base {
            std::env::set_var(ENV_BASE_URL, v);
        }
        if let Some(v) = prev_model {
            std::env::set_var(ENV_MODEL, v);
        }
    }

    #[test]
    fn blank_api_key_is_treated_as_disabled() {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev = std::env::var(ENV_API_KEY).ok();
        std::env::set_var(ENV_API_KEY, "   ");
        assert!(GreAtlasAiConfig::from_env().is_none());
        match prev {
            Some(v) => std::env::set_var(ENV_API_KEY, v),
            None => std::env::remove_var(ENV_API_KEY),
        }
    }

    #[test]
    fn explicit_disable_overrides_api_key() {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev_key = std::env::var(ENV_API_KEY).ok();
        let prev_disabled = std::env::var(ENV_AI_DISABLED).ok();
        std::env::set_var(ENV_API_KEY, "sk-test");
        std::env::set_var(ENV_AI_DISABLED, "1");
        assert!(GreAtlasAiConfig::from_env().is_none());
        match prev_key {
            Some(v) => std::env::set_var(ENV_API_KEY, v),
            None => std::env::remove_var(ENV_API_KEY),
        }
        match prev_disabled {
            Some(v) => std::env::set_var(ENV_AI_DISABLED, v),
            None => std::env::remove_var(ENV_AI_DISABLED),
        }
    }
}
