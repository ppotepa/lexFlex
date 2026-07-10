use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    TemporalReference,
    Deixis,
    EmotionExpression,
    Pragmatics,
    ProDrop,
    FreeWordOrder,
    MorphologicalInflection,
    Quantification,
    FormalProof,
    NumericPrecision,
    SetTheory,
    LogicalConnectives,
    Procedures,
    ControlFlow,
    SideEffects,
    TypeSystem,
    Negation,
    Coordination,
    Conditionality,
    Reference,
    Ambiguity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Limitation {
    NoEmotionExpression,
    NoDeixis,
    NoFormalProofs,
    NoQuantification,
    NoAmbiguity,
    NoProcedures,
    NoProDrop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InexpressibleFeature {
    pub capability: Capability,
    pub suggestion: Option<String>,
}
