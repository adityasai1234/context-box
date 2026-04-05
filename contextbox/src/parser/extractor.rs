use crate::error::{AppError, AppResult};
use std::path::Path;

pub struct DocumentParser;

impl DocumentParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_file(&self, path: &Path) -> AppResult<String> {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "txt" | "md" | "json" | "xml" | "yaml" | "yml" | "toml" | "cfg" | "ini" | "log" => {
                std::fs::read_to_string(path)
                    .map_err(|e| AppError::ParseError(format!("Failed to read file: {}", e)))
            }
            "rs" | "js" | "ts" | "jsx" | "tsx" | "py" | "go" | "java" | "c" | "cpp" | "h" | "hpp" | "cs" | "rb" | "php" | "swift" | "kt" | "scala" | "html" | "css" | "scss" | "sass" | "less" => {
                std::fs::read_to_string(path)
                    .map_err(|e| AppError::ParseError(format!("Failed to read code file: {}", e)))
            }
            "pdf" => {
                self.parse_pdf(path)
            }
            "docx" => {
                self.parse_docx(path)
            }
            _ => Err(AppError::ParseError(format!(
                "Unsupported file type: {}. Supported types: txt, md, json, xml, yaml, toml, cfg, ini, log, rs, js, ts, py, go, java, c, cpp, html, css, pdf, docx",
                extension
            ))),
        }
    }

    fn parse_pdf(&self, path: &Path) -> AppResult<String> {
        #[cfg(feature = "pdf")]
        {
            use std::fs::File;
            use pdfplumber::PDF;

            let file = File::open(path)
                .map_err(|e| AppError::ParseError(format!("Failed to open PDF: {}", e)))?;
            
            let mut pdf = PDF::load(file)
                .map_err(|e| AppError::ParseError(format!("Failed to parse PDF: {}", e)))?;

            let mut text = String::new();
            for page in pdf.pages() {
                if let Ok(page_text) = page.extract_text() {
                    text.push_str(&page_text);
                    text.push('\n');
                }
            }

            Ok(text)
        }

        #[cfg(not(feature = "pdf"))]
        {
            Err(AppError::ParseError("PDF support not compiled. Enable 'pdf' feature or install pdftotext".to_string()))
        }
    }

    fn parse_docx(&self, path: &Path) -> AppResult<String> {
        #[cfg(feature = "docx")]
        {
            use std::fs::File;
            use docx_rs::*;

            let file = File::open(path)
                .map_err(|e| AppError::ParseError(format!("Failed to open DOCX: {}", e)))?;
            
            let doc = docx_rs::read_docx(&mut std::io::BufReader::new(file))
                .map_err(|e| AppError::ParseError(format!("Failed to parse DOCX: {}", e)))?;

            let mut text = String::new();
            for child in doc.document.body {
                match child {
                    docx_rs::DocumentChild::Paragraph(p) => {
                        for child in p.children {
                            match child {
                                docx_rs::ParagraphChild::Run(run) => {
                                    for child in run.children {
                                        if let docx_rs::RunChild::Text(t) = child {
                                            text.push_str(&t.text);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        text.push('\n');
                    }
                    _ => {}
                }
            }

            Ok(text)
        }

        #[cfg(not(feature = "docx"))]
        {
            Err(AppError::ParseError("DOCX support not compiled. Enable 'docx' feature".to_string()))
        }
    }

    pub fn parse_text(&self, text: &str) -> AppResult<String> {
        Ok(text.to_string())
    }

    pub fn chunk_text(&self, text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut start = 0;

        while start < chars.len() {
            let end = (start + chunk_size).min(chars.len());
            let chunk: String = chars[start..end].iter().collect();
            
            if !chunk.trim().is_empty() {
                chunks.push(chunk);
            }

            start += chunk_size.saturating_sub(overlap);
        }

        chunks
    }

    pub fn get_doc_type(&self, path: &Path) -> Option<String> {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
    }
}

impl Default for DocumentParser {
    fn default() -> Self {
        Self::new()
    }
}
