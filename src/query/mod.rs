use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use crate::document::knowledge::{
    ClaimId, ClaimStatus, KnowledgeObjectRef, KnowledgePredicateRef,
    KnowledgeSubjectRef,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryIdError(pub String);

impl fmt::Display for QueryIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for QueryIdError {}

macro_rules! query_id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = QueryIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                validate_id(s)?;
                Ok(Self(s.to_string()))
            }
        }
    };
}

fn validate_id(value: &str) -> Result<(), QueryIdError> {
    if value.is_empty() {
        return Err(QueryIdError("id must not be empty".into()));
    }
    if value.len() > 128 {
        return Err(QueryIdError("id too long".into()));
    }
    if !value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.')) {
        return Err(QueryIdError("invalid id characters".into()));
    }
    Ok(())
}

query_id_type!(QueryId);
query_id_type!(QueryPlanId);
query_id_type!(QueryResultId);
query_id_type!(AnswerId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuerySchema {
    V1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryIntent {
    LookupEntity,
    LookupProperty,
    LookupRelation,
    Count,
    Boolean,
    Temporal,
    Causal,
    Comparison,
    Contradiction,
    Provenance,
    Explain,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryWorldPolicy {
    OpenWorld,
    ClosedWorldValidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryEvidencePolicy {
    Required,
    Optional,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryConstraint {
    And(Vec<QueryConstraint>),
    Or(Vec<QueryConstraint>),
    Not(Box<QueryConstraint>),
    Subject(KnowledgeSubjectRef),
    Predicate(KnowledgePredicateRef),
    Object(KnowledgeObjectRef),
    ClaimStatus(ClaimStatus),
    Claim(ClaimId),
    EvidenceRequired(bool),
    TextContains(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryProjection {
    Claim,
    Subject,
    Predicate,
    Object,
    Evidence,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryAggregation {
    Count,
    Distinct,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryOrdering {
    Asc(String),
    Desc(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryInterlingua {
    pub schema: QuerySchema,
    pub id: QueryId,
    pub source_language: String,
    pub source_text: Option<String>,
    pub intent: QueryIntent,
    pub constraints: QueryConstraint,
    pub projection: Vec<QueryProjection>,
    pub aggregation: Option<QueryAggregation>,
    pub ordering: Vec<QueryOrdering>,
    pub limit: Option<usize>,
    pub world_policy: QueryWorldPolicy,
    pub evidence_policy: QueryEvidencePolicy,
    pub query_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryStep {
    Scan,
    Filter,
    Project,
    Aggregate,
    Explain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryExecutionPlan {
    pub id: QueryPlanId,
    pub query_id: QueryId,
    pub steps: Vec<QueryStep>,
    pub plan_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryExecutionResultRow {
    pub columns: BTreeMap<String, String>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryExecutionResult {
    pub id: QueryResultId,
    pub query_id: QueryId,
    pub plan_id: QueryPlanId,
    pub rows: Vec<QueryExecutionResultRow>,
    pub diagnostics: Vec<String>,
    pub result_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnswerStatus {
    Exact,
    Supported,
    Partial,
    Ambiguous,
    Conflicting,
    Unknown,
    Unsupported,
    InvalidQuery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnswerKind {
    Entity,
    EntityList,
    Value,
    ValueList,
    Count,
    Boolean,
    Text,
    ConflictReport,
    EvidenceReport,
    NoAnswer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentAnswer {
    pub id: AnswerId,
    pub query_id: QueryId,
    pub plan_id: QueryPlanId,
    pub status: AnswerStatus,
    pub kind: AnswerKind,
    pub text: Option<String>,
    pub rows: Vec<QueryExecutionResultRow>,
    pub evidence: Vec<String>,
    pub conflicts: Vec<String>,
    pub answer_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryError(pub String);

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for QueryError {}

mod service;

pub use service::QueryService;
