use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::info;

//================================================================================
// AppState: Core application state
//================================================================================

/// Shared state for the application, including the API key pool.
struct AppState {
    api_keys: Vec<String>,
    counter: AtomicUsize,
}

impl AppState {
    /// Selects an API key from the pool in a round-robin fashion.
    fn get_next_api_key(&self) -> &str {
        let index = self.counter.fetch_add(1, Ordering::SeqCst);
        &self.api_keys[index % self.api_keys.len()]
    }
}

//================================================================================
// Error Handling
//================================================================================

/// A simple error type that can be converted into an HTTP response.
enum AppError {
    Internal(anyhow::Error),
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Internal(e) => {
                tracing::error!("Internal server error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Internal server error"})),
                )
                    .into_response()
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg}))).into_response(),
        }
    }
}

/// Allows converting anyhow::Error into AppError::Internal.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        AppError::Internal(err.into())
    }
}

//================================================================================
// OpenAI-Compatible Data Structures
//================================================================================

#[derive(Deserialize, Debug)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Debug)]
struct OpenAIChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
}

#[derive(Serialize, Debug)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: String,
}

#[derive(Serialize, Debug)]
struct ModelList {
    object: String,
    data: Vec<ModelObject>,
}

#[derive(Serialize, Debug)]
struct ModelObject {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
}

//--------------------------------------------------------------------------------
// Gemini-Specific Model List Structures
//--------------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
struct GeminiModelList {
    models: Vec<GeminiModelInfo>,
}

#[derive(Deserialize, Debug)]
struct GeminiModelInfo {
    name: String,
    #[serde(rename = "supportedGenerationMethods")]
    supported_generation_methods: Vec<String>,
}


//================================================================================
// Gemini API Data Structures
//================================================================================

#[derive(Serialize, Debug)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<SystemInstruction>,
}

#[derive(Serialize, Debug, Clone)]
struct SystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiContent {
    #[serde(default)]
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize, Debug)]
struct GeminiCandidate {
    content: GeminiContent,
}


//================================================================================
// API Handler and Logic
//================================================================================

/// Handles requests to the root path, listing available endpoints.
async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "Welcome to the Gemini API Pool!",
        "endpoints": [
            {
                "path": "/",
                "methods": ["GET"],
                "description": "Lists all available endpoints."
            },
            {
                "path": "/v1/chat/completions",
                "methods": ["POST"],
                "description": "OpenAI-compatible chat completions endpoint."
            },
            {
                "path": "/v1/models",
                "methods": ["GET"],
                "description": "Lists all available models."
            }
        ]
    }))
}

/// Lists the available models by fetching them from the Gemini API.
async fn list_models_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ModelList>, AppError> {
    info!("Received request to list models");
    let api_key = state.get_next_api_key();

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models?key={}",
        api_key
    );

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;

    // Handle potential errors from the Gemini API
    let status = response.status();
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        tracing::error!(
            "Gemini API returned an error on model list. Status: {}. Body: {}",
            status,
            error_body
        );
        return Err(AppError::Internal(anyhow::anyhow!(
            "Upstream Gemini API Error on model list: {} - {}",
            status,
            error_body
        )));
    }

    // Deserialize the successful response
    let gemini_model_list: GeminiModelList = response.json().await?;

    // Transform the Gemini model list to the OpenAI format
    let openai_models = gemini_model_list
        .models
        .into_iter()
        // We only care about models that can be used for chat completions
        .filter(|m| m.supported_generation_methods.contains(&"generateContent".to_string()))
        // We also filter for non-embedding models
        .filter(|m| !m.name.contains("embedding"))
        .map(|m| ModelObject {
            // Strip the "models/" prefix to get the clean model ID
            id: m.name.strip_prefix("models/").unwrap_or(&m.name).to_string(),
            object: "model".to_string(),
            created: 1, // Placeholder timestamp, as Gemini API doesn't provide it
            owned_by: "google".to_string(),
        })
        .collect();

    Ok(Json(ModelList {
        object: "list".to_string(),
        data: openai_models,
    }))
}

/// Handles the chat completions request.
async fn chat_completions_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OpenAIChatRequest>,
) -> Result<Json<OpenAIChatResponse>, AppError> {
    info!("Received OpenAI chat request for model: {}", payload.model);

    // 1. Select an API key from the pool
    let api_key = state.get_next_api_key();
    info!("Using API key ending with: ...{}", &api_key[api_key.len().saturating_sub(4)..]);

    // 2. Convert OpenAI request to Gemini request
    let model_name = payload.model.clone();
    let gemini_request = convert_to_gemini_request(payload)?;

    // 3. Send request to Gemini API
    let client = reqwest::Client::new();
    let gemini_url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model_name, api_key
    );

    let response = client
        .post(&gemini_url)
        .json(&gemini_request)
        .send()
        .await?;

    // Check if the response from Gemini is successful
    let status = response.status();
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        tracing::error!(
            "Gemini API returned an error. Status: {}. Body: {}",
            status,
            error_body
        );
        // Forward the error from Gemini to the client
        return Err(AppError::Internal(anyhow::anyhow!(
            "Upstream Gemini API Error: {} - {}",
            status,
            error_body
        )));
    }

    // Deserialize the successful response, with robust error handling
    let response_body = response.text().await?;
    let gemini_response: GeminiResponse = match serde_json::from_str(&response_body) {
        Ok(res) => res,
        Err(e) => {
            tracing::error!(
                "Failed to deserialize Gemini response. Status: {}. Error: {}. Body: {}",
                status,
                e,
                response_body
            );
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to deserialize upstream Gemini response"
            )));
        }
    };

    // 4. Convert Gemini response back to OpenAI response
    let openai_response = convert_to_openai_response(gemini_response, model_name)?;

    Ok(Json(openai_response))
}

/// Converts an OpenAI request payload into a Gemini request payload.
fn convert_to_gemini_request(req: OpenAIChatRequest) -> Result<GeminiRequest, AppError> {
    let mut gemini_contents = Vec::new();
    let mut system_instruction = None;

    for message in req.messages {
        match message.role.as_str() {
            "system" => {
                // The Gemini API currently only supports one system instruction.
                // If multiple are found, we'll use the first one.
                if system_instruction.is_none() {
                    system_instruction = Some(SystemInstruction {
                        parts: vec![GeminiPart { text: message.content }],
                    });
                }
            }
            "user" | "assistant" => {
                let role = if message.role == "assistant" { "model" } else { "user" };
                gemini_contents.push(GeminiContent {
                    role: role.to_string(),
                    parts: vec![GeminiPart { text: message.content }],
                });
            }
            _ => return Err(AppError::BadRequest(format!("Unsupported role: {}", message.role))),
        }
    }

    // The Gemini API requires that the conversation starts with a "user" role.
    // If the first message after a potential system prompt is from the "model",
    // it's an invalid sequence.
    if let Some(first_content) = gemini_contents.first() {
        if first_content.role == "model" {
            return Err(AppError::BadRequest("Conversation must start with a user message after the system prompt.".to_string()));
        }
    }


    Ok(GeminiRequest {
        contents: gemini_contents,
        system_instruction,
    })
}

/// Converts a Gemini response into an OpenAI-compatible response.
fn convert_to_openai_response(
    res: GeminiResponse,
    model_name: String,
) -> Result<OpenAIChatResponse, AppError> {
    let choice = res
        .candidates
        .into_iter()
        .next()
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|part| OpenAIChoice {
            index: 0,
            message: OpenAIMessage {
                role: "assistant".to_string(),
                content: part.text,
            },
            finish_reason: "stop".to_string(),
        })
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("No content found in Gemini response")))?;

    Ok(OpenAIChatResponse {
        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        model: model_name,
        choices: vec![choice],
    })
}


//================================================================================
// Main Function
//================================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load .env file
    dotenvy::dotenv().ok();

    // Load API keys from environment
    let api_keys_str = env::var("GEMINI_API_KEYS")
        .expect("GEMINI_API_KEYS must be set in .env file");
    let api_keys: Vec<String> = api_keys_str.split(',').map(|s| s.trim().to_string()).collect();
    if api_keys.is_empty() || api_keys.iter().any(|k| k.is_empty()) {
        panic!("GEMINI_API_KEYS must contain at least one non-empty key");
    }
    info!("Loaded {} API keys.", api_keys.len());

    // Create shared state
    let app_state = Arc::new(AppState {
        api_keys,
        counter: AtomicUsize::new(0),
    });

    // Create Axum router
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/v1/chat/completions", post(chat_completions_handler))
        .route("/v1/models", get(list_models_handler))
        .with_state(app_state);

    // Get listen address from environment or use default for containerized environments
    let listen_addr = env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    info!("Starting server on {}", listen_addr);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}