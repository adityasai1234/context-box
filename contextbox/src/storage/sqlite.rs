use std::path::PathBuf;
use std::fs;
use base64::Engine;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::crypto::age::{self, CryptoError};

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Encryption error: {0}")]
    CryptoError(#[from] CryptoError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Document not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub content: String,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMeta {
    pub id: String,
    pub name: String,
    pub source: String,
    pub created_at: String,
    pub size: u64,
}

pub struct SqliteStorage {
    conn: Connection,
    key: [u8; 32],
}

impl SqliteStorage {
    pub fn new(db_path: &PathBuf, key: &[u8; 32]) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(db_path)?;
        let mut storage = Self { conn, key: *key };
        storage.init_tables()?;
        Ok(storage)
    }
    
    fn init_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                content_encrypted BLOB NOT NULL,
                content_nonce BLOB NOT NULL,
                source TEXT NOT NULL DEFAULT 'upload',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_documents_created 
             ON documents(created_at DESC)",
            [],
        )?;
        
        Ok(())
    }
    
    pub fn add(&self, name: String, content: String, source: &str) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        let encrypted = age::encrypt_string(&content, &self.key)?;
        let encrypted_bytes = base64::engine::general_purpose::STANDARD
            .decode(&encrypted)
            .map_err(|e| CryptoError::EncryptError(e.to_string()))?;
        
        let nonce = generate_nonce();
        
        self.conn.execute(
            "INSERT INTO documents (id, name, content_encrypted, content_nonce, source, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                name,
                encrypted_bytes,
                nonce,
                source,
                now,
                now
            ],
        )?;
        
        Ok(id)
    }
    
    pub fn get(&self, id: &str) -> Result<Document> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, content_encrypted, content_nonce, source, created_at, updated_at
             FROM documents WHERE id = ?1"
        )?;
        
        let doc = stmt.query_row(params![id], |row| {
            Ok(RawDocument {
                id: row.get(0)?,
                name: row.get(1)?,
                content_encrypted: row.get(2)?,
                content_nonce: row.get(3)?,
                source: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        }).map_err(|_| StorageError::NotFound(id.to_string()))?;
        
        let encrypted_b64 = base64::engine::general_purpose::STANDARD
            .encode(&doc.content_encrypted);
        
        let content = age::decrypt_string(&encrypted_b64, &self.key)?;
        
        Ok(Document {
            id: doc.id,
            name: doc.name,
            content,
            source: doc.source,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        })
    }
    
    pub fn list(&self) -> Result<Vec<DocumentMeta>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, source, created_at, length(content_encrypted) as size
             FROM documents ORDER BY created_at DESC"
        )?;
        
        let docs = stmt.query_map([], |row| {
            Ok(DocumentMeta {
                id: row.get(0)?,
                name: row.get(1)?,
                source: row.get(2)?,
                created_at: row.get(3)?,
                size: row.get::<_, i64>(4)? as u64,
            })
        })?;
        
        let mut result = Vec::new();
        for doc in docs {
            result.push(doc?);
        }
        
        Ok(result)
    }
    
    pub fn delete(&self, id: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "DELETE FROM documents WHERE id = ?1",
            params![id],
        )?;
        
        Ok(affected > 0)
    }
    
    pub fn search(&self, query: &str) -> Result<Vec<Document>> {
        let query_lower = query.to_lowercase();
        
        let all_docs = self.list()?;
        let mut results = Vec::new();
        
        for meta in all_docs {
            if meta.name.to_lowercase().contains(&query_lower) {
                if let Ok(doc) = self.get(&meta.id) {
                    if doc.content.to_lowercase().contains(&query_lower) {
                        results.push(doc);
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    pub fn count(&self) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM documents",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }
    
    pub fn close(self) {
        drop(self.conn);
    }
}

fn generate_nonce() -> Vec<u8> {
    use rand::RngCore;
    let mut nonce = vec![0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    nonce
}

struct RawDocument {
    id: String,
    name: String,
    content_encrypted: Vec<u8>,
    content_nonce: Vec<u8>,
    source: String,
    created_at: String,
    updated_at: String,
}
