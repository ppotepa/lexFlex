use crate::document::{GraphNodeId, SentenceId};
use serde::{Deserialize, Serialize};

use super::id::{EntityClusterId, ResolutionDecisionId, ResolutionDiagnosticId, ResolutionMentionRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityResolutionDiagnosticSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityResolutionDiagnostic {
    pub id: ResolutionDiagnosticId,
    pub code: String,
    pub severity: EntityResolutionDiagnosticSeverity,
    pub message: String,
    pub sentence_id: Option<SentenceId>,
    pub mention: Option<ResolutionMentionRef>,
    pub cluster_id: Option<EntityClusterId>,
    pub decision_id: Option<ResolutionDecisionId>,
    pub node_id: Option<GraphNodeId>,
    pub recoverable: bool,
}
