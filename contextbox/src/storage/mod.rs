pub mod document;
pub mod document_store;
pub mod sqlite;
pub mod vector;

pub use document::{Document, DocumentListItem, DocumentMetadata, DocumentSource};
pub use document_store::{DocumentStore, StoredDocument, create_document};
pub use sqlite::{SqliteStorage, Document as SqliteDocument, DocumentMeta, StorageError};
pub use vector::{SearchResult, VectorStore};
