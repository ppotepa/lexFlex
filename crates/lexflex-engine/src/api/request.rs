use crate::api::input::TextInput;
use lexflex_lingua::solve::EvidencePolicy;
use lexflex_lingua::{ExecutionPolicy, LinguaGoal, LinguaProgram};
use lexflex_model::Evidence;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineRequest {
    EvaluateLingua {
        program: LinguaProgram,
        policy: ExecutionPolicy,
        include_trace: bool,
    },
    IngestLingua {
        program: LinguaProgram,
        evidence: Vec<Evidence>,
    },
    QueryLingua {
        goal: LinguaGoal,
    },
    AnalyzeText {
        input: TextInput,
        include_derivation: bool,
    },
    IngestText {
        input: TextInput,
    },
    AskText {
        input: TextInput,
        evidence_policy: EvidencePolicy,
        limit: Option<usize>,
    },
    InspectSession,
    ClearSession,
}
