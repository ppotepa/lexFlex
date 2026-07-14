use super::anchor::MentionAnchor;
use super::id::{GraphDiagnosticId, GraphNodeId};
use crate::document::id::SentenceId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DocumentGraphDiagnosticSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentGraphDiagnostic {
    pub id: GraphDiagnosticId,
    pub code: String,
    pub severity: DocumentGraphDiagnosticSeverity,
    pub message: String,
    pub sentence_id: Option<SentenceId>,
    pub node_id: Option<GraphNodeId>,
    pub anchor: MentionAnchor,
    pub cause: Option<String>,
    pub recoverable: bool,
}
