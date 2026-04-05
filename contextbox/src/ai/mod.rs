use crate::error::{AppError, AppResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub data: Vec<EmbeddingData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub embedding: Vec<f32>,
}

pub struct EmbeddingClient {
    client: Client,
    api_key: String,
    model: String,
}

impl EmbeddingClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub async fn generate_embedding(&self, text: &str) -> AppResult<Vec<f32>> {
        let request = EmbeddingRequest {
            model: self.model.clone(),
            input: text.to_string(),
        };

        let response = self.client
            .post(format!("{}/embeddings", OPENROUTER_API_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::AiError(format!("Failed to call embedding API: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::AiError(format!("Embedding API error {}: {}", status, text)));
        }

        let result: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| AppError::AiError(format!("Failed to parse embedding response: {}", e)))?;

        result.data
            .first()
            .map(|d| d.embedding.clone())
            .ok_or_else(|| AppError::AiError("No embedding returned".to_string()))
    }

    pub async fn generate_embeddings(&self, texts: &[String]) -> AppResult<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();

        for text in texts {
            let embedding = self.generate_embedding(text).await?;
            embeddings.push(embedding);
        }

        Ok(embeddings)
    }
}

pub struct ChatClient {
    client: Client,
    api_key: String,
    default_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
}

impl ChatClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            default_model: "anthropic/claude-3-haiku".to_string(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.default_model = model;
        self
    }

    pub async fn chat(&self, system: &str, user: &str, context: Option<&str>) -> AppResult<String> {
        let mut messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system.to_string(),
            }
        ];

        if let Some(ctx) = context {
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: format!("Context from documents:\n\n{}", ctx),
            });
        }

        messages.push(ChatMessage {
            role: "user".to_string(),
            content: user.to_string(),
        });

        let request = ChatRequest {
            model: self.default_model.clone(),
            messages,
            temperature: 0.7,
        };

        let response = self.client
            .post(format!("{}/chat/completions", OPENROUTER_API_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::AiError(format!("Failed to call chat API: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::AiError(format!("Chat API error {}: {}", status, text)));
        }

        let result: ChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::AiError(format!("Failed to parse chat response: {}", e)))?;

        result.choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| AppError::AiError("No response from chat API".to_string()))
    }
}
