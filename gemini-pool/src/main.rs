use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;
use tracing::info;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

//================================================================================
// Database Models
//================================================================================

#[derive(Debug, Serialize, Clone)]
struct ApiKey {
    id: Uuid,
    key_name: String,
    api_key: String,
    is_active: bool,
    created_at: DateTime<Utc>,
    total_requests: i64,
    total_input_tokens: i64,
    total_output_tokens: i64,
}

#[derive(Debug, Serialize)]
struct UsageLog {
    id: Uuid,
    api_key_id: Uuid,
    timestamp: DateTime<Utc>,
    endpoint: String,
    model: String,
    input_tokens: i32,
    output_tokens: i32,
    success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
}

#[derive(Debug, Serialize)]
struct DashboardStats {
    total_api_keys: i64,
    total_requests: i64,
    total_tokens: i64,
    active_keys: i64,
}

#[derive(Debug, Serialize)]
struct ApiKeysResponse {
    api_keys: Vec<ApiKey>,
}

#[derive(Debug, Deserialize)]
struct CreateApiKeyRequest {
    key_name: String,
    api_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateApiKeyResponse {
    id: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct UpdateApiKeyRequest {
    key_name: String,
    is_active: bool,
}

//================================================================================
// AppState: Core application state
//================================================================================

/// Shared state for the application, including the API key pool.
struct AppState {
    api_keys: Vec<String>,
    counter: AtomicUsize,
    db_pool: SqlitePool,
    jwt_secret: String,
    admin_username: String,
    admin_password: String,
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
// Database Functions
//================================================================================

async fn initialize_database(pool: &SqlitePool) -> anyhow::Result<()> {
    // Create api_keys table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS api_keys (
            id TEXT PRIMARY KEY,
            key_name TEXT NOT NULL,
            api_key TEXT NOT NULL UNIQUE,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            total_requests INTEGER NOT NULL DEFAULT 0,
            total_input_tokens INTEGER NOT NULL DEFAULT 0,
            total_output_tokens INTEGER NOT NULL DEFAULT 0
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create usage_logs table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS usage_logs (
            id TEXT PRIMARY KEY,
            api_key_id TEXT NOT NULL,
            timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            endpoint TEXT NOT NULL,
            model TEXT NOT NULL,
            input_tokens INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            success BOOLEAN NOT NULL DEFAULT TRUE,
            FOREIGN KEY (api_key_id) REFERENCES api_keys (id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    info!("Database initialized successfully");
    Ok(())
}

async fn get_or_create_api_key(pool: &SqlitePool, api_key: &str) -> anyhow::Result<Uuid> {
    // Try to find existing API key
    if let Some(row) = sqlx::query("SELECT id FROM api_keys WHERE api_key = ?")
        .bind(api_key)
        .fetch_optional(pool)
        .await?
    {
        let id_str: String = row.get("id");
        return Ok(Uuid::parse_str(&id_str)?);
    }

    // Create new API key entry
    let id = Uuid::new_v4();
    let key_name = format!("API Key {}", &api_key[..8.min(api_key.len())]);
    
    sqlx::query(
        "INSERT INTO api_keys (id, key_name, api_key) VALUES (?, ?, ?)"
    )
    .bind(id.to_string())
    .bind(&key_name)
    .bind(api_key)
    .execute(pool)
    .await?;

    Ok(id)
}

async fn log_usage(
    pool: &SqlitePool,
    api_key_id: Uuid,
    endpoint: &str,
    model: &str,
    input_tokens: i32,
    output_tokens: i32,
    success: bool,
) -> anyhow::Result<()> {
    let log_id = Uuid::new_v4();
    
    // Insert usage log
    sqlx::query(
        "INSERT INTO usage_logs (id, api_key_id, endpoint, model, input_tokens, output_tokens, success) 
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(log_id.to_string())
    .bind(api_key_id.to_string())
    .bind(endpoint)
    .bind(model)
    .bind(input_tokens)
    .bind(output_tokens)
    .bind(success)
    .execute(pool)
    .await?;

    // Update totals in api_keys table
    sqlx::query(
        "UPDATE api_keys 
         SET total_requests = total_requests + 1,
             total_input_tokens = total_input_tokens + ?,
             total_output_tokens = total_output_tokens + ?
         WHERE id = ?"
    )
    .bind(input_tokens)
    .bind(output_tokens)
    .bind(api_key_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

//================================================================================
// Authentication Middleware
//================================================================================

/// Middleware to verify API key in Authorization header
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, AppError> {
    // Check Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("Missing Authorization header".to_string()))?;

    // Expect "Bearer <api_key>" format
    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::BadRequest("Invalid Authorization header format. Expected: Bearer <api_key>".to_string()));
    }

    let provided_key = auth_header.strip_prefix("Bearer ").unwrap();
    
    // Verify the API key exists in database and is active
    let api_key_result = sqlx::query("SELECT id FROM api_keys WHERE api_key = ? AND is_active = TRUE")
        .bind(provided_key)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let api_key_id = api_key_result
        .ok_or_else(|| AppError::BadRequest("Invalid or inactive API key".to_string()))?
        .get::<String, _>("id");

    // Store the API key ID in request extensions for later use
    request.extensions_mut().insert(api_key_id);

    // If authentication successful, continue to the handler
    Ok(next.run(request).await)
}

//================================================================================
// Admin API Handlers
//================================================================================

async fn admin_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // Verify credentials
    if payload.username != state.admin_username || payload.password != state.admin_password {
        return Err(AppError::BadRequest("Invalid credentials".to_string()));
    }

    // Create JWT token
    let claims = Claims {
        sub: payload.username,
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_ref()),
    )
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(LoginResponse { token }))
}

async fn admin_verify_token(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let auth_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("Missing Authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::BadRequest("Invalid Authorization header format".to_string()));
    }

    let token = auth_header.strip_prefix("Bearer ").unwrap();
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::BadRequest("Invalid token".to_string()))?;

    Ok(Json(serde_json::json!({"status": "valid"})))
}

async fn admin_dashboard(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DashboardStats>, AppError> {
    let total_api_keys = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM api_keys")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(0);

    let total_requests = sqlx::query_scalar::<_, i64>("SELECT SUM(total_requests) FROM api_keys")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(0);

    let total_input_tokens = sqlx::query_scalar::<_, i64>("SELECT SUM(total_input_tokens) FROM api_keys")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(0);

    let total_output_tokens = sqlx::query_scalar::<_, i64>("SELECT SUM(total_output_tokens) FROM api_keys")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(0);

    let active_keys = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM api_keys WHERE is_active = TRUE")
        .fetch_one(&state.db_pool)
        .await
        .unwrap_or(0);

    Ok(Json(DashboardStats {
        total_api_keys,
        total_requests,
        total_tokens: total_input_tokens + total_output_tokens,
        active_keys,
    }))
}

async fn admin_list_api_keys(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiKeysResponse>, AppError> {
    let rows = sqlx::query(
        "SELECT id, key_name, api_key, is_active, created_at, total_requests, total_input_tokens, total_output_tokens 
         FROM api_keys ORDER BY created_at DESC"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    let api_keys: Vec<ApiKey> = rows
        .into_iter()
        .map(|row| ApiKey {
            id: Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
            key_name: row.get("key_name"),
            api_key: row.get("api_key"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            total_requests: row.get("total_requests"),
            total_input_tokens: row.get("total_input_tokens"),
            total_output_tokens: row.get("total_output_tokens"),
        })
        .collect();

    Ok(Json(ApiKeysResponse { api_keys }))
}

async fn admin_get_api_key(
    State(state): State<Arc<AppState>>,
    Path(key_id): Path<String>,
) -> Result<Json<ApiKey>, AppError> {
    let row = sqlx::query(
        "SELECT id, key_name, api_key, is_active, created_at, total_requests, total_input_tokens, total_output_tokens 
         FROM api_keys WHERE id = ?"
    )
    .bind(&key_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or_else(|| AppError::BadRequest("API Key not found".to_string()))?;

    let api_key = ApiKey {
        id: Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
        key_name: row.get("key_name"),
        api_key: row.get("api_key"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        total_requests: row.get("total_requests"),
        total_input_tokens: row.get("total_input_tokens"),
        total_output_tokens: row.get("total_output_tokens"),
    };

    Ok(Json(api_key))
}

async fn admin_create_api_key(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, AppError> {
    let api_key = payload.api_key.unwrap_or_else(|| {
        // Generate a random API key
        format!("gp_{}", uuid::Uuid::new_v4().to_string().replace("-", ""))
    });

    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO api_keys (id, key_name, api_key) VALUES (?, ?, ?)"
    )
    .bind(id.to_string())
    .bind(&payload.key_name)
    .bind(&api_key)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(CreateApiKeyResponse {
        id: id.to_string(),
        api_key,
    }))
}

async fn admin_update_api_key(
    State(state): State<Arc<AppState>>,
    Path(key_id): Path<String>,
    Json(payload): Json<UpdateApiKeyRequest>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query(
        "UPDATE api_keys SET key_name = ?, is_active = ? WHERE id = ?"
    )
    .bind(&payload.key_name)
    .bind(payload.is_active)
    .bind(&key_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::BadRequest("API Key not found".to_string()));
    }

    Ok(StatusCode::OK)
}

async fn admin_delete_api_key(
    State(state): State<Arc<AppState>>,
    Path(key_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM api_keys WHERE id = ?")
        .bind(&key_id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::BadRequest("API Key not found".to_string()));
    }

    // Also delete related usage logs
    sqlx::query("DELETE FROM usage_logs WHERE api_key_id = ?")
        .bind(&key_id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok(StatusCode::OK)
}

async fn admin_auth_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, AppError> {
    let auth_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("Missing Authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::BadRequest("Invalid Authorization header format".to_string()));
    }

    let token = auth_header.strip_prefix("Bearer ").unwrap();
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::BadRequest("Invalid or expired token".to_string()))?;

    Ok(next.run(request).await)
}

//================================================================================
// Token Counting Functions
//================================================================================

/// Simple token counting function (approximation)
fn count_tokens(text: &str) -> i32 {
    // Simple approximation: roughly 1 token per 4 characters for English text
    // This is a rough estimate; for production use, you'd want a proper tokenizer
    (text.len() as f32 / 4.0).ceil() as i32
}

fn count_tokens_in_messages(messages: &[OpenAIMessage]) -> i32 {
    messages.iter().map(|msg| count_tokens(&msg.content)).sum()
}

//================================================================================
// API Handler and Logic
//================================================================================

/// Handles requests to the root path, serves index.html
async fn root_handler() -> impl IntoResponse {
    match std::fs::read_to_string("web/index.html") {
        Ok(html) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
            html
        ).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            "Index page not found"
        ).into_response()
    }
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

    // Load admin credentials
    let admin_username = env::var("ADMIN_USERNAME")
        .unwrap_or_else(|_| "admin".to_string());
    let admin_password = env::var("ADMIN_PASSWORD")
        .expect("ADMIN_PASSWORD must be set in .env file for admin authentication");
    if admin_password.is_empty() {
        panic!("ADMIN_PASSWORD must not be empty");
    }
    
    // Load JWT secret
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default_jwt_secret_change_in_production".to_string());
    
    info!("Admin authentication configured.");

    // Initialize database
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./gemini_pool.db".to_string());
    
    let db_pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    initialize_database(&db_pool)
        .await
        .expect("Failed to initialize database");

    // Create shared state
    let app_state = Arc::new(AppState {
        api_keys,
        counter: AtomicUsize::new(0),
        db_pool,
        jwt_secret,
        admin_username,
        admin_password,
    });

    // Create protected API routes that require client API key authentication
    let protected_api_routes = Router::new()
        .route("/v1/chat/completions", post(chat_completions_handler))
        .route("/v1/models", get(list_models_handler))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Create admin routes that require admin JWT authentication
    let admin_routes = Router::new()
        .route("/admin/api/dashboard", get(admin_dashboard))
        .route("/admin/api/api-keys", get(admin_list_api_keys))
        .route("/admin/api/api-keys", post(admin_create_api_key))
        .route("/admin/api/api-keys/{id}", get(admin_get_api_key))
        .route("/admin/api/api-keys/{id}", put(admin_update_api_key))
        .route("/admin/api/api-keys/{id}", delete(admin_delete_api_key))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            admin_auth_middleware,
        ));

    // Create public admin auth routes
    let auth_routes = Router::new()
        .route("/admin/api/auth/login", post(admin_login))
        .route("/admin/api/auth/verify", get(admin_verify_token));

    // Create static file service for web interface
    let static_service = ServeDir::new("web");

    // Create main Axum router
    let app = Router::new()
        .route("/", get(root_handler))
        .merge(protected_api_routes)
        .merge(admin_routes)  // API routes must come first to have higher priority
        .merge(auth_routes)   // Auth routes must come before static service
        .nest_service("/admin", static_service)  // Static files come last
        .fallback_service(ServeDir::new("web"))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Get listen address from environment or use default for containerized environments
    let listen_addr = env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    info!("Starting server on {}", listen_addr);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}