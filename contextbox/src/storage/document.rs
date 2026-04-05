use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub content: String,
    pub source: DocumentSource,
    pub metadata: DocumentMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentSource {
    Upload,
    Url,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentMetadata {
    pub doc_type: Option<String>,
    pub size: Option<u64>,
    pub url: Option<String>,
    pub tags: Vec<String>,
}

impl Document {
    pub fn new(name: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            content,
            source: DocumentSource::Upload,
            metadata: DocumentMetadata::default(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn from_url(name: String, content: String, url: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            content,
            source: DocumentSource::Url,
            metadata: DocumentMetadata {
                url: Some(url),
                ..Default::default()
            },
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentListItem {
    pub id: String,
    pub name: String,
    pub source: DocumentSource,
    pub created_at: DateTime<Utc>,
    pub size: u64,
}

impl From<Document> for DocumentListItem {
    fn from(doc: Document) -> Self {
        Self {
            id: doc.id,
            name: doc.name,
            source: doc.source,
            created_at: doc.created_at,
            size: doc.content.len() as u64,
        }
    }
}
