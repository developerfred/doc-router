use serde::{Deserialize, Serialize};

/// A parsed documentation section, identified by heading hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub doc_path: String,
    pub title: String,
    pub level: u8,
    pub line_start: usize,
    pub line_end: usize,
    pub byte_start: usize,
    pub byte_end: usize,
    pub token_estimate: usize,
}

/// A documentation file parsed into sections.
#[derive(Debug, Clone)]
pub struct ParsedDoc {
    pub path: String,
    pub total_tokens: usize,
    pub sections: Vec<Section>,
}

/// The decision the router makes for a given task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    /// The router found relevant sections above the threshold.
    Read,
    /// The router is confident nothing needs to be read.
    Skip,
    /// The signal is ambiguous; the agent should decide.
    Uncertain,
}

/// A single navigation recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Navigation {
    pub doc: String,
    pub section_id: String,
    pub title: String,
    pub reason: String,
    pub score: f32,
    pub token_estimate: usize,
}

/// The output of routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    pub task: String,
    pub decision: Decision,
    pub navigate: Vec<Navigation>,
    pub total_navigated_tokens: usize,
    pub total_document_tokens: usize,
    pub savings_ratio: String,
}
