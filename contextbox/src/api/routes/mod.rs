use axum::{
    extract::{Path, State, Multipart},
    routing::{get, post, delete},
    Json, Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::storage::{Document, DocumentListItem, VectorStore};
use crate::parser::DocumentParser;
use crate::ai::{EmbeddingClient, ChatClient};

pub struct AppState {
    pub config: Config,
    pub vector_store: Arc<Mutex<VectorStore>>,
    pub documents: Arc<Mutex<Vec<Document>>>,
    pub embedding_client: Option<EmbeddingClient>,
    pub chat_client: Option<ChatClient>,
    pub parser: DocumentParser,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let vector_store = VectorStore::new(Some(config.storage.vector_db_path.as_path()))
            .expect("Failed to initialize vector store");
        
        let embedding_client = config.api.openrouter_api_key
            .clone()
            .map(EmbeddingClient::new);
        
        let chat_client = config.api.openrouter_api_key
            .clone()
            .map(ChatClient::new);

        Self {
            config,
            vector_store: Arc::new(Mutex::new(vector_store)),
            documents: Arc::new(Mutex::new(Vec::new())),
            embedding_client,
            chat_client,
            parser: DocumentParser::new(),
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
    Json(serde_json::json!({ "status": "ok" }))
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

async fn list_documents(State(state): State<AppState>) -> Json<Vec<DocumentListItem>> {
    let docs = state.documents.lock().await;
    let list: Vec<DocumentListItem> = docs.iter()
        .map(|d| DocumentListItem {
            id: d.id.clone(),
            name: d.name.clone(),
            source: d.source.clone(),
            created_at: d.created_at,
            size: d.content.len() as u64,
        })
        .collect();
    Json(list)
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

    let doc = Document::new(filename.clone(), content);
    let id = doc.id.clone();
    
    let mut docs = state.documents.lock().await;
    docs.push(doc);

    Json(serde_json::json!({
        "id": id,
        "name": filename,
        "message": "Document uploaded successfully"
    }))
}

async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Document> {
    let docs = state.documents.lock().await;
    let doc = docs.iter()
        .find(|d| d.id == id)
        .cloned()
        .ok_or_else(|| AppError::NotFound(format!("Document {} not found", id)))
        .unwrap();
    Json(doc)
}

async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    let mut docs = state.documents.lock().await;
    let initial_len = docs.len();
    docs.retain(|d| d.id != id);
    
    if docs.len() == initial_len {
        return Json(serde_json::json!({
            "error": "Document not found"
        }));
    }

    let mut vector_store = state.vector_store.lock().await;
    let _ = vector_store.delete_by_document(&id);

    Json(serde_json::json!({
        "message": "Document deleted successfully"
    }))
}

async fn semantic_search(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let query = payload.get("query")
        .and_then(|q| q.as_str())
        .unwrap_or("");
    
    let limit = payload.get("limit")
        .and_then(|l| l.as_u64())
        .unwrap_or(5) as usize;

    if state.embedding_client.is_none() {
        return Json(serde_json::json!({
            "error": "Embedding client not configured. Set OPENROUTER_API_KEY."
        }));
    }

    Json(serde_json::json!({
        "query": query,
        "results": [],
        "message": "Search not fully implemented - requires embeddings"
    }))
}

async fn rag_chat(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let message = payload.get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("");

    if state.chat_client.is_none() {
        return Json(serde_json::json!({
            "error": "Chat client not configured. Set OPENROUTER_API_KEY."
        }));
    }

    Json(serde_json::json!({
        "response": "Chat not fully implemented",
        "message": message
    }))
}
