use crate::api::outcome::AssertionWriteOutcome;
use crate::api::text::{TextAnalysis, TextAnalysisAlternative};
use crate::error::EngineErrorCode;
use lexflex_lingua::{ExecutionResult, LinguaGoal, QuerySolution};
use lexflex_model::{AssertionId, SemanticAssertion};
use lexflex_parser::ParseError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineResponse {
    LinguaEvaluated {
        result: ExecutionResult,
    },
    LinguaIngested {
        assertion: SemanticAssertion,
        outcome: AssertionWriteOutcome,
        snapshot_hash: String,
    },
    LinguaQueryResult {
        goal: LinguaGoal,
        solutions: Vec<QuerySolution>,
        snapshot_hash: String,
    },
    TextAnalyzed {
        analysis: TextAnalysis,
    },
    TextIngested {
        analysis: TextAnalysis,
        assertion: SemanticAssertion,
        outcome: AssertionWriteOutcome,
        snapshot_hash: String,
    },
    TextAnswer {
        analysis: TextAnalysis,
        goal: LinguaGoal,
        solutions: Vec<QuerySolution>,
        snapshot_hash: String,
    },
    TextAmbiguous {
        alternatives: Vec<TextAnalysisAlternative>,
    },
    TextNotParsed {
        diagnostics: Vec<ParseError>,
    },
    SessionInspection {
        assertion_count: usize,
        evidence_count: usize,
        snapshot_hash: String,
        model_hash: String,
        language_hash: String,
    },
    SessionCleared {
        snapshot_hash: String,
    },
    Unsupported {
        capability: String,
        message: String,
    },
    Error {
        code: EngineErrorCode,
        message: String,
        diagnostics: Vec<EngineDiagnostic>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineDiagnostic {
    MissingAssertion { assertion_id: AssertionId },
}
