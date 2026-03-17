// src/templates/mod.rs

use rust_embed::RustEmbed;
use std::path::PathBuf;

use crate::domain::TemplateKind;

#[derive(RustEmbed)]
#[folder = "embedded_templates"]
struct EmbeddedTemplates;

pub struct TemplateStore {
    project_root: PathBuf,
}

impl TemplateStore {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Load template with fallback chain
    pub fn load(&self, kind: TemplateKind) -> Result<String, TemplateError> {
        let filename = format!("{}.hbs", kind.source_key());

        // 1. Project-local override
        let local = self
            .project_root
            .join(".gencomp")
            .join("templates")
            .join(&filename);
        if local.exists() {
            return std::fs::read_to_string(&local).map_err(|e| TemplateError::Read(local, e));
        }

        // 2. User-global override (optional - skip for MVP)

        // 3. Embedded default
        EmbeddedTemplates::get(&filename)
            .map(|f| String::from_utf8_lossy(&f.data).to_string())
            .ok_or(TemplateError::NotFound(kind))
    }
}

#[derive(Debug)]
pub enum TemplateError {
    NotFound(TemplateKind),
    Read(PathBuf, std::io::Error),
    InvalidUtf8,
}
