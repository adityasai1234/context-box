use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStore {
    documents: HashMap<String, StoredDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredDocument {
    pub id: String,
    pub name: String,
    pub content: String,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
    pub metadata: HashMap<String, String>,
}

impl DocumentStore {
    pub fn new(path: &PathBuf) -> Self {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(data) => {
                    match serde_json::from_str(&data) {
                        Ok(store) => return store,
                        Err(e) => {
                            eprintln!("Error loading document store: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading document store: {}", e);
                }
            }
        }
        
        Self {
            documents: HashMap::new(),
        }
    }
    
    pub fn save(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, data)
    }
    
    pub fn add(&mut self, doc: StoredDocument) -> String {
        let id = doc.id.clone();
        self.documents.insert(id.clone(), doc);
        id
    }
    
    pub fn get(&self, id: &str) -> Option<&StoredDocument> {
        self.documents.get(id)
    }
    
    pub fn get_mut(&mut self, id: &str) -> Option<&mut StoredDocument> {
        self.documents.get_mut(id)
    }
    
    pub fn remove(&mut self, id: &str) -> Option<StoredDocument> {
        self.documents.remove(id)
    }
    
    pub fn list(&self) -> Vec<&StoredDocument> {
        self.documents.values().collect()
    }
    
    pub fn count(&self) -> usize {
        self.documents.len()
    }
    
    pub fn clear(&mut self) {
        self.documents.clear();
    }
}

pub fn create_document(name: String, content: String, source: &str) -> StoredDocument {
    let now = chrono::Utc::now().to_rfc3339();
    
    StoredDocument {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        content,
        source: source.to_string(),
        created_at: now.clone(),
        updated_at: now,
        metadata: HashMap::new(),
    }
}
