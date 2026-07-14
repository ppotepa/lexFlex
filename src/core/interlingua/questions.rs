use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Illocution {
    Statement,
    Question,
    Command,
    Exclamation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuestionKind {
    What,
    Who,
    Where,
    When,
    Which,
    HowMany,
    YesNo,
    Why,
    How,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QueryVariable {
    pub id: String,
}

impl QueryVariable {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryTerm {
    Entity(Entity),
    Concept(ConceptId),
    Variable(QueryVariable),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropositionPattern {
    pub predicate: ConceptId,
    pub subject: Option<QueryTerm>,
    pub object: Option<QueryTerm>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryProjection {
    Subject,
    Object,
    Predicate,
    Concept,
    Value,
    Boolean,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestionSemantics {
    pub kind: QuestionKind,
    pub variable: QueryVariable,
    pub proposition: PropositionPattern,
    pub projection: QueryProjection,
}

impl QuestionSemantics {
    pub fn definition(kind: QuestionKind, subject: Entity) -> Self {
        let projection = match kind {
            QuestionKind::What | QuestionKind::Which => QueryProjection::Concept,
            QuestionKind::Who => QueryProjection::Subject,
            _ => QueryProjection::Concept,
        };
        Self {
            kind,
            variable: QueryVariable::new("answer"),
            proposition: PropositionPattern {
                predicate: ConceptId::new("IS_A"),
                subject: Some(QueryTerm::Entity(subject)),
                object: Some(QueryTerm::Variable(QueryVariable::new("answer"))),
            },
            projection,
        }
    }

    pub fn relation(
        kind: QuestionKind,
        predicate: ConceptId,
        object: Entity,
        projection: QueryProjection,
    ) -> Self {
        Self {
            kind,
            variable: QueryVariable::new("answer"),
            proposition: PropositionPattern {
                predicate,
                subject: Some(QueryTerm::Variable(QueryVariable::new("answer"))),
                object: Some(QueryTerm::Entity(object)),
            },
            projection,
        }
    }

    pub fn relation_from_subject(
        kind: QuestionKind,
        predicate: ConceptId,
        subject: Entity,
        projection: QueryProjection,
    ) -> Self {
        Self {
            kind,
            variable: QueryVariable::new("answer"),
            proposition: PropositionPattern {
                predicate,
                subject: Some(QueryTerm::Entity(subject)),
                object: Some(QueryTerm::Variable(QueryVariable::new("answer"))),
            },
            projection,
        }
    }
}

pub fn question_kind_from_concept(concept: &ConceptId) -> Option<QuestionKind> {
    match concept.0.as_str() {
        "WH_THING" => Some(QuestionKind::What),
        "WH_PERSON" => Some(QuestionKind::Who),
        "WH_LOCATION" => Some(QuestionKind::Where),
        "WH_TIME" => Some(QuestionKind::When),
        "WH_CHOICE" => Some(QuestionKind::Which),
        "WH_COUNT" => Some(QuestionKind::HowMany),
        "WH_REASON" => Some(QuestionKind::Why),
        "WH_MANNER" => Some(QuestionKind::How),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Modality {
    Realis,
    Irrealis,
    Possibility,
    Necessity,
}

// ─── Temporal ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemporalReference {
    Absolute { timestamp: String },
    Relative { offset_days: i64, anchor: TemporalAnchor },
    Deictic { word: String },
    Duration { days: i64 },
    Frequency { times: i32, period: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TemporalAnchor {
    Now,
    Past,
    Future,
}

// ─── Quantification ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Quantifier {
    Universal,
    Existential,
    NegatedExistential,
    Numerical(i32),
    Proportional(String),
}
