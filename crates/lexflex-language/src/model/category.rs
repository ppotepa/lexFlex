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

    pub fn depth(&self) -> usize {
        match self {
            Self::Atom { .. } => 1,
            Self::Function {
                result, argument, ..
            } => 1 + result.depth().max(argument.depth()),
        }
    }

    pub fn node_count(&self) -> usize {
        match self {
            Self::Atom { .. } => 1,
            Self::Function {
                result, argument, ..
            } => 1 + result.node_count() + argument.node_count(),
        }
    }

    pub fn map_types(&self, mapper: &mut impl FnMut(&CategoryType) -> CategoryType) -> Self {
        match self {
            Self::Atom {
                kind,
                semantic_type,
                features,
            } => Self::Atom {
                kind: kind.clone(),
                semantic_type: mapper(semantic_type),
                features: features.clone(),
            },
            Self::Function {
                result,
                argument,
                direction,
                semantic_parameter,
                features,
            } => Self::Function {
                result: Box::new(result.map_types(mapper)),
                argument: Box::new(argument.map_types(mapper)),
                direction: *direction,
                semantic_parameter: semantic_parameter.clone(),
                features: features.clone(),
            },
        }
    }

    pub fn try_map_types<E>(
        &self,
        mapper: &mut impl FnMut(&CategoryType) -> Result<CategoryType, E>,
    ) -> Result<Self, E> {
        match self {
            Self::Atom {
                kind,
                semantic_type,
                features,
            } => Ok(Self::Atom {
                kind: kind.clone(),
                semantic_type: mapper(semantic_type)?,
                features: features.clone(),
            }),
            Self::Function {
                result,
                argument,
                direction,
                semantic_parameter,
                features,
            } => Ok(Self::Function {
                result: Box::new(result.try_map_types(mapper)?),
                argument: Box::new(argument.try_map_types(mapper)?),
                direction: *direction,
                semantic_parameter: semantic_parameter.clone(),
                features: features.clone(),
            }),
        }
    }

    pub fn with_root_features(
        &self,
        additional: &FeatureStructure,
    ) -> Result<Self, crate::FeatureConflict> {
        match self {
            Self::Atom {
                kind,
                semantic_type,
                features,
            } => Ok(Self::Atom {
                kind: kind.clone(),
                semantic_type: semantic_type.clone(),
                features: features.unify(additional)?,
            }),
            Self::Function {
                result,
                argument,
                direction,
                semantic_parameter,
                features,
            } => Ok(Self::Function {
                result: result.clone(),
                argument: argument.clone(),
                direction: *direction,
                semantic_parameter: semantic_parameter.clone(),
                features: features.unify(additional)?,
            }),
        }
    }
}
