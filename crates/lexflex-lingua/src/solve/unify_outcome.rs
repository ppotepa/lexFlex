use lexflex_model::{ParameterId, SemanticExpression, SemanticType, VariableId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnifyMismatch {
    Shape,

    MissingBinding(ParameterId),

    ConflictingBinding {
        variable: VariableId,
        existing: SemanticExpression,
        incoming: SemanticExpression,
    },

    Value {
        pattern: SemanticExpression,
        candidate: SemanticExpression,
    },

    Occurs {
        variable: VariableId,
        candidate: SemanticExpression,
    },

    VariableType {
        variable: VariableId,
        expected: SemanticType,
        actual: SemanticType,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnifyOutcome {
    Matched,
    Mismatch(UnifyMismatch),
}
