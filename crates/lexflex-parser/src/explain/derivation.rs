use crate::token::Token;
use lexflex_language::LexicalSenseId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivationNode {
    Lexical {
        token: Token,
        sense: LexicalSenseId,
    },
    Applied {
        left: Box<DerivationNode>,
        right: Box<DerivationNode>,
    },
}

impl DerivationNode {
    pub fn depth(&self) -> usize {
        match self {
            Self::Lexical { .. } => 1,
            Self::Applied { left, right } => 1 + left.depth().max(right.depth()),
        }
    }
}
