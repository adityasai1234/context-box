use std::path::Path;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::error::{AppError, AppResult};

const EMBEDDING_DIMENSION: usize = 384;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub id: String,
    pub document_id: String,
    pub vector: Vec<f32>,
    pub chunk_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document_id: String,
    pub document_name: String,
    pub chunk_text: String,
    pub score: f32,
}

#[derive(Debug)]
pub struct VectorStore {
    embeddings: Vec<Embedding>,
    document_index: HashMap<String, String>,
}

impl VectorStore {
    pub fn new(db_path: Option<&Path>) -> AppResult<Self> {
        let embeddings = if let Some(path) = db_path {
            if path.exists() {
                let data = std::fs::read_to_string(path)
                    .map_err(|e| AppError::StorageError(e.to_string()))?;
                serde_json::from_str(&data)
                    .map_err(|e| AppError::StorageError(e.to_string()))?
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let mut store = Self {
            embeddings,
            document_index: HashMap::new(),
        };
        
        store.build_index();
        Ok(store)
    }

    fn build_index(&mut self) {
        self.document_index.clear();
        for embedding in &self.embeddings {
            self.document_index.insert(
                embedding.document_id.clone(),
                embedding.chunk_text.clone(),
            );
        }
    }

    pub fn add_embedding(&mut self, doc_id: String, chunk_text: String, vector: Vec<f32>) -> AppResult<()> {
        if vector.len() != EMBEDDING_DIMENSION {
            return Err(AppError::InvalidInput(
                format!("Expected embedding dimension {}, got {}", EMBEDDING_DIMENSION, vector.len())
            ));
        }

        let embedding = Embedding {
            id: uuid::Uuid::new_v4().to_string(),
            document_id: doc_id.clone(),
            vector,
            chunk_text: chunk_text.clone(),
        };

        self.embeddings.push(embedding);
        self.document_index.insert(doc_id, chunk_text);

        Ok(())
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot / (norm_a * norm_b)
    }

    pub fn search(&self, query_vector: &[f32], limit: usize) -> AppResult<Vec<SearchResult>> {
        if query_vector.len() != EMBEDDING_DIMENSION {
            return Err(AppError::InvalidInput(
                format!("Query vector dimension must be {}", EMBEDDING_DIMENSION)
            ));
        }

        let mut scored: Vec<(f32, &Embedding)> = self.embeddings
            .iter()
            .map(|e| (Self::cosine_similarity(&e.vector, query_vector), e))
            .filter(|(score, _)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        Ok(scored.into_iter().take(limit).map(|(score, e)| {
            SearchResult {
                document_id: e.document_id.clone(),
                document_name: self.document_index
                    .get(&e.document_id)
                    .cloned()
                    .unwrap_or_default(),
                chunk_text: e.chunk_text.clone(),
                score,
            }
        }).collect())
    }

    pub fn delete_by_document(&mut self, doc_id: &str) -> AppResult<()> {
        self.embeddings.retain(|e| e.document_id != doc_id);
        self.document_index.remove(doc_id);
        Ok(())
    }

    pub fn save(&self, path: &Path) -> AppResult<()> {
        let data = serde_json::to_string_pretty(&self.embeddings)
            .map_err(|e| AppError::StorageError(e.to_string()))?;
        std::fs::write(path, data)
            .map_err(|e| AppError::StorageError(e.to_string()))?;
        Ok(())
    }

    pub fn get_all_document_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.document_index.keys().cloned().collect();
        ids.sort();
        ids.dedup();
        ids
    }
}

impl Default for VectorStore {
    fn default() -> Self {
        Self {
            embeddings: Vec::new(),
            document_index: HashMap::new(),
        }
    }
}
