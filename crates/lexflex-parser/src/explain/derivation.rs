use crate::token::Token;
use lexflex_language::LexicalSenseId;
use lexflex_model::ParameterId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplicationRule {
    Forward {
        semantic_parameter: ParameterId,
    },
    Backward {
        semantic_parameter: ParameterId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivationNode {
    Lexical {
        token: Token,
        sense: LexicalSenseId,
    },
    Applied {
        rule: ApplicationRule,
        left: Box<DerivationNode>,
        right: Box<DerivationNode>,
    },
}

impl DerivationNode {
    pub fn depth(&self) -> usize {
        match self {
            Self::Lexical { .. } => 1,
            Self::Applied { left, right, .. } => 1 + left.depth().max(right.depth()),
        }
    }
}
