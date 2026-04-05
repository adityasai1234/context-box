use axum::{
    extract::{Path, State, Multipart, Query},
    routing::{get, post, delete},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::storage::{SqliteStorage, SqliteDocument as StoredDocument, DocumentMeta};
use crate::crypto::load_key;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub storage: Arc<Mutex<SqliteStorage>>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize { 5 }

impl AppState {
    pub fn new(config: Config) -> Self {
        let key = load_key().expect("Encryption key not found. Run 'cb keygen' first.");
        
        let storage = SqliteStorage::new(
            &config.storage.data_dir.join("documents.db"),
            &key
        ).expect("Failed to initialize storage");
        
        Self {
            config,
            storage: Arc::new(Mutex::new(storage)),
        }
    }
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/config", get(get_config))
        .route("/api/documents", get(list_documents).post(upload_document))
        .route("/api/documents/:id", get(get_document).delete(delete_document))
        .route("/api/search", post(semantic_search))
        .route("/api/chat", post(rag_chat))
        .with_state(state)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "ContextBox" }))
}

async fn get_config(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "enable_mcp": state.config.features.enable_mcp,
        "enable_web_ui": state.config.features.enable_web_ui,
        "enable_cli": state.config.features.enable_cli,
        "enable_chat": state.config.features.enable_chat,
        "enable_url_crawl": state.config.features.enable_url_crawl,
    }))
}

async fn list_documents(State(state): State<AppState>) -> Json<Vec<DocumentMeta>> {
    let storage = state.storage.lock().await;
    Json(storage.list().unwrap_or_default())
}

async fn upload_document(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Json<serde_json::Value> {
    let mut content = String::new();
    let mut filename = "untitled".to_string();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            filename = field.file_name().unwrap_or("untitled").to_string();
            content = field.text().await.unwrap_or_default();
        } else if name == "content" {
            content = field.text().await.unwrap_or_default();
        } else if name == "name" {
            filename = field.text().await.unwrap_or_default();
        }
    }

    if content.is_empty() {
        return Json(serde_json::json!({
            "error": "No content provided"
        }));
    }

    let storage = state.storage.lock().await;
    match storage.add(filename.clone(), content, "api") {
        Ok(id) => Json(serde_json::json!({
            "id": id,
            "name": filename,
            "message": "Document uploaded successfully"
        })),
        Err(e) => Json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StoredDocument>, AppError> {
    let storage = state.storage.lock().await;
    match storage.get(&id) {
        Ok(doc) => Ok(Json(doc)),
        Err(e) => Err(AppError::NotFound(e.to_string())),
    }
}

async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let storage = state.storage.lock().await;
    match storage.delete(&id) {
        Ok(true) => Json(serde_json::json!({
            "message": "Document deleted successfully"
        })),
        Ok(false) => Json(serde_json::json!({
            "error": "Document not found"
        })),
        Err(e) => Json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

async fn semantic_search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<serde_json::Value> {
    let storage = state.storage.lock().await;
    match storage.search(&query.query) {
        Ok(results) => {
            let limit = query.limit;
            let results: Vec<_> = results.into_iter().take(limit).map(|r| {
                serde_json::json!({
                    "document_id": r.id,
                    "name": r.name,
                    "content": r.content,
                    "source": r.source,
                    "created_at": r.created_at
                })
            }).collect();
            Json(serde_json::json!({
                "query": query.query,
                "results": results
            }))
        },
        Err(e) => Json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

async fn rag_chat(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let message = payload.get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("");

    Json(serde_json::json!({
        "response": "Chat not fully implemented - requires OPENROUTER_API_KEY",
        "message": message
    }))
}
