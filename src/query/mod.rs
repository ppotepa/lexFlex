use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use crate::document::knowledge::{
    ClaimId, ClaimStatus, KnowledgeObjectRef, KnowledgePredicateRef,
    KnowledgeSubjectRef,
};
use crate::core::interlingua::{Entity, QueryTerm, QuestionKind, QuestionSemantics};
use crate::document::resolution::EntityClusterId;

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

impl QueryInterlingua {
    /// Convert language-neutral question semantics into the authoritative
    /// structured query contract. Surface parsers never need to know how the
    /// knowledge store indexes entities.
    pub fn from_question<F>(
        question: &QuestionSemantics,
        source_language: impl Into<String>,
        source_text: Option<String>,
        resolve_entity: F,
    ) -> Result<Self, QueryError>
    where
        F: Fn(&Entity) -> Option<EntityClusterId>,
    {
        let predicate = predicate_from_concept(&question.proposition.predicate.0);
        let (mut intent, projection, mut constraints) = match (&question.proposition.subject, &question.proposition.object) {
            (Some(QueryTerm::Entity(entity)), Some(QueryTerm::Variable(_))) => {
                (if question.kind == QuestionKind::HowMany { QueryIntent::Count } else { QueryIntent::LookupProperty }, QueryProjection::Object, vec![constraint_from(subject_constraints(entity, &resolve_entity))])
            }
            (Some(QueryTerm::Variable(_)), Some(QueryTerm::Entity(entity))) => {
                (if question.kind == QuestionKind::HowMany { QueryIntent::Count } else { QueryIntent::LookupRelation }, QueryProjection::Subject, vec![constraint_from(object_constraints(entity, &resolve_entity))])
            }
            (Some(QueryTerm::Entity(subject)), Some(QueryTerm::Entity(object))) => {
                (QueryIntent::Boolean, QueryProjection::Claim, vec![constraint_from(subject_constraints(subject, &resolve_entity)), constraint_from(object_constraints(object, &resolve_entity))])
            }
            _ => return Err(QueryError("question proposition has no supported entity binding".into())),
        };

        constraints.push(QueryConstraint::Predicate(predicate));
        if question.kind == QuestionKind::YesNo {
            intent = QueryIntent::Boolean;
        }
        if question.kind == QuestionKind::HowMany {
            intent = QueryIntent::Count;
        }
        let language = source_language.into();
        let source_text = source_text;
        let id_seed = serde_json::to_string(&(&language, &source_text, question))
            .map_err(|error| QueryError(error.to_string()))?;
        let id = QueryId(format!("query:{}", digest_hex(id_seed.as_bytes())));
        let mut query = Self {
            schema: QuerySchema::V1,
            id,
            source_language: language,
            source_text,
            intent,
            constraints: QueryConstraint::And(constraints),
            projection: vec![projection],
            aggregation: (question.kind == QuestionKind::HowMany).then_some(QueryAggregation::Count),
            ordering: Vec::new(),
            limit: Some(20),
            world_policy: QueryWorldPolicy::OpenWorld,
            evidence_policy: QueryEvidencePolicy::Required,
            query_sha256: String::new(),
        };
        query.query_sha256 = hash_query(&query)?;
        Ok(query)
    }
}

fn subject_constraints<F>(entity: &Entity, resolve: &F) -> Vec<QueryConstraint>
where F: Fn(&Entity) -> Option<EntityClusterId> {
    if let Some(cluster) = resolve(entity) {
        vec![QueryConstraint::Subject(KnowledgeSubjectRef::EntityCluster(cluster))]
    } else {
        vec![QueryConstraint::Subject(KnowledgeSubjectRef::Unresolved)]
    }
}

fn object_constraints<F>(entity: &Entity, resolve: &F) -> Vec<QueryConstraint>
where F: Fn(&Entity) -> Option<EntityClusterId> {
    let mut constraints = Vec::new();
    if let Some(name) = entity.name.clone() {
        constraints.push(QueryConstraint::Object(KnowledgeObjectRef::TextLiteral(name)));
    }
    if let Some(cluster) = resolve(entity) {
        constraints.push(QueryConstraint::Object(KnowledgeObjectRef::EntityCluster(cluster)));
    }
    if constraints.is_empty() { constraints.push(QueryConstraint::Object(KnowledgeObjectRef::Unknown)); }
    constraints
}

fn constraint_from(mut constraints: Vec<QueryConstraint>) -> QueryConstraint {
    if constraints.len() == 1 { constraints.remove(0) } else { QueryConstraint::Or(constraints) }
}

fn predicate_from_concept(concept: &str) -> KnowledgePredicateRef {
    match concept {
        "IS_A" => KnowledgePredicateRef::Relation(crate::document::knowledge::KnowledgeRelationKind::HasProperty),
        "LOCATED_IN" => KnowledgePredicateRef::Relation(crate::document::knowledge::KnowledgeRelationKind::LocatedAt),
        value => KnowledgePredicateRef::Custom(value.to_string()),
    }
}

fn hash_query(query: &QueryInterlingua) -> Result<String, QueryError> {
    let mut value = serde_json::to_value(query).map_err(|error| QueryError(error.to_string()))?;
    if let serde_json::Value::Object(map) = &mut value { map.remove("query_sha256"); }
    let bytes = serde_json::to_vec(&value).map_err(|error| QueryError(error.to_string()))?;
    Ok(digest_hex(&bytes))
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    let mut digest = sha2::Sha256::new();
    digest.update(bytes);
    digest.finalize().into()
}

fn digest_hex(bytes: &[u8]) -> String {
    sha256(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
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
pub enum QueryOperator {
    ScanBySubject,
    ScanByPredicate,
    ScanByObject,
    ScanByTime,
    Filter,
    Join,
    Project,
    Aggregate,
    Distinct,
    Sort,
    Limit,
    EvidenceAttach,
    ConflictGroup,
    UnknownGuard,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryExecutionPlan {
    pub id: QueryPlanId,
    pub query_id: QueryId,
    pub steps: Vec<QueryStep>,
    pub operators: Vec<QueryOperator>,
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
    No,
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
