use lexflex_language::LanguageId;
use lexflex_model::{CanonicalDigest, ConceptId, ParameterId, SemanticExpression, VariableId};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationNode {
    Value,
    Lexical {
        expression: SemanticExpression,
    },
    Clause {
        subject: Box<GenerationNode>,
        predicate: Box<GenerationNode>,
    },
    Apply {
        concept: ConceptId,
        bindings: BTreeMap<ParameterId, GenerationNode>,
    },
    Coordination {
        conjunction: bool,
        items: Vec<GenerationNode>,
    },
    Negated(Box<GenerationNode>),
    Quantified {
        universal: bool,
        variable: VariableId,
        body: Box<GenerationNode>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationPlan {
    root: GenerationNode,
    language: LanguageId,
    semantic_hash: CanonicalDigest,
}

impl GenerationPlan {
    pub(crate) fn new(
        root: GenerationNode,
        language: LanguageId,
        semantic_hash: CanonicalDigest,
    ) -> Self {
        Self {
            root,
            language,
            semantic_hash,
        }
    }

    pub fn root(&self) -> &GenerationNode {
        &self.root
    }

    pub fn language(&self) -> &LanguageId {
        &self.language
    }

    pub fn semantic_hash(&self) -> &CanonicalDigest {
        &self.semantic_hash
    }
}
