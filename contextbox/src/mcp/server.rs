use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: ToolInputSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolInputSchema {
    Object { schema: serde_json::Value },
}
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::storage::{Document, DocumentListItem, VectorStore};
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInput {
    pub query: Option<String>,
    pub limit: Option<usize>,
    pub id: Option<String>,
    pub content: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct McpServer {
    pub vector_store: Arc<Mutex<VectorStore>>,
    pub documents: Arc<Mutex<Vec<Document>>>,
}

impl McpServer {
    pub fn new(
        vector_store: Arc<Mutex<VectorStore>>,
        documents: Arc<Mutex<Vec<Document>>>,
    ) -> Self {
        Self {
            vector_store,
            documents,
        }
    }

    pub fn get_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "search_documents".to_string(),
                description: "Search documents by semantic query. Returns relevant document chunks.".to_string(),
                input_schema: ToolInputSchema::Object {
                    schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "The search query"
                            },
                            "limit": {
                                "type": "number",
                                "description": "Maximum number of results",
                                "default": 5
                            }
                        },
                        "required": ["query"]
                    }),
                },
            },
            Tool {
                name: "list_documents".to_string(),
                description: "List all available documents in the knowledge base.".to_string(),
                input_schema: ToolInputSchema::Object {
                    schema: serde_json::json!({
                        "type": "object",
                        "properties": {}
                    }),
                },
            },
            Tool {
                name: "get_document".to_string(),
                description: "Get the full content of a specific document by ID.".to_string(),
                input_schema: ToolInputSchema::Object {
                    schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "The document ID"
                            }
                        },
                        "required": ["id"]
                    }),
                },
            },
            Tool {
                name: "add_document".to_string(),
                description: "Add a new document to the knowledge base. The content will be indexed for search.".to_string(),
                input_schema: ToolInputSchema::Object {
                    schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string",
                                "description": "The document content"
                            },
                            "metadata": {
                                "type": "object",
                                "description": "Optional metadata (name, source, tags)"
                            }
                        },
                        "required": ["content"]
                    }),
                },
            },
        ]
    }

    pub async fn handle_tool_call(&self, tool_name: &str, input: serde_json::Value) -> AppResult<serde_json::Value> {
        match tool_name {
            "search_documents" => {
                let input: McpToolInput = serde_json::from_value(input)
                    .map_err(|e| crate::error::AppError::InvalidInput(e.to_string()))?;
                
                let query = input.query.unwrap_or_default();
                let _limit = input.limit.unwrap_or(5);
                
                Ok(serde_json::json!({
                    "query": query,
                    "results": [],
                    "message": "Search not yet implemented - requires embedding client setup"
                }))
            }
            "list_documents" => {
                let docs = self.documents.lock().await;
                let list: Vec<DocumentListItem> = docs.iter()
                    .map(|d| DocumentListItem {
                        id: d.id.clone(),
                        name: d.name.clone(),
                        source: d.source.clone(),
                        created_at: d.created_at,
                        size: d.content.len() as u64,
                    })
                    .collect();
                
                Ok(serde_json::json!({ "documents": list }))
            }
            "get_document" => {
                let input: McpToolInput = serde_json::from_value(input)
                    .map_err(|e| crate::error::AppError::InvalidInput(e.to_string()))?;
                
                let id = input.id.ok_or_else(|| 
                    crate::error::AppError::InvalidInput("id is required".to_string()))?;
                
                let docs = self.documents.lock().await;
                let doc = docs.iter()
                    .find(|d| d.id == id)
                    .cloned()
                    .ok_or_else(|| crate::error::AppError::NotFound(format!("Document {} not found", id)))?;
                
                Ok(serde_json::json!({
                    "id": doc.id,
                    "name": doc.name,
                    "content": doc.content,
                    "source": doc.source,
                    "created_at": doc.created_at
                }))
            }
            "add_document" => {
                let input: McpToolInput = serde_json::from_value(input)
                    .map_err(|e| crate::error::AppError::InvalidInput(e.to_string()))?;
                
                let content = input.content.ok_or_else(|| 
                    crate::error::AppError::InvalidInput("content is required".to_string()))?;
                
                let name = input.metadata
                    .and_then(|m| m.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                    .unwrap_or_else(|| "Untitled".to_string());
                
                let doc = Document::new(name, content);
                let mut docs = self.documents.lock().await;
                docs.push(doc.clone());
                
                Ok(serde_json::json!({
                    "id": doc.id,
                    "name": doc.name,
                    "message": "Document added successfully"
                }))
            }
            _ => Err(crate::error::AppError::InvalidInput(
                format!("Unknown tool: {}", tool_name)
            )),
        }
    }
}
