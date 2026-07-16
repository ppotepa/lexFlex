use crate::id::CategoryTypeVariableId;
use crate::{FeatureStructure, SurfaceRelationId};
use lexflex_model::{ParameterId, SemanticType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SlashDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CategoryType {
    Concrete(SemanticType),
    Variable(CategoryTypeVariableId),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AtomicCategoryKind {
    Sentence,
    NounPhrase,
    Predicate,
    MarkedArgument { relation: SurfaceRelationId },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SyntacticCategory {
    Atom {
        kind: AtomicCategoryKind,
        semantic_type: CategoryType,
        features: FeatureStructure,
    },
    Function {
        result: Box<SyntacticCategory>,
        argument: Box<SyntacticCategory>,
        direction: SlashDirection,
        semantic_parameter: ParameterId,
        features: FeatureStructure,
    },
}

impl SyntacticCategory {
    pub fn sentence() -> Self {
        Self::Atom {
            kind: AtomicCategoryKind::Sentence,
            semantic_type: CategoryType::Concrete(SemanticType::Boolean),
            features: FeatureStructure::default(),
        }
    }

    pub fn noun_phrase(semantic_type: CategoryType, features: FeatureStructure) -> Self {
        Self::Atom {
            kind: AtomicCategoryKind::NounPhrase,
            semantic_type,
            features,
        }
    }

    pub fn predicate(subject_type: CategoryType, features: FeatureStructure) -> Self {
        Self::Atom {
            kind: AtomicCategoryKind::Predicate,
            semantic_type: subject_type,
            features,
        }
    }

    pub fn marked_argument(
        relation: SurfaceRelationId,
        semantic_type: CategoryType,
        features: FeatureStructure,
    ) -> Self {
        Self::Atom {
            kind: AtomicCategoryKind::MarkedArgument { relation },
            semantic_type,
            features,
        }
    }

    pub fn semantic_type(&self) -> &CategoryType {
        match self {
            Self::Atom { semantic_type, .. } => semantic_type,
            Self::Function { result, .. } => result.semantic_type(),
        }
    }

    pub fn features(&self) -> &FeatureStructure {
        match self {
            Self::Atom { features, .. } | Self::Function { features, .. } => features,
        }
    }

    pub fn is_sentence(&self) -> bool {
        matches!(
            self,
            Self::Atom {
                kind: AtomicCategoryKind::Sentence,
                ..
            }
        )
    }
}
