#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRuntimeBudget {
    max_formal_derivations: usize,
    max_formal_alternatives: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TextRuntimeBudgetError {
    #[error("formal derivation limit must be greater than zero")]
    ZeroFormalDerivations,
    #[error("formal alternative limit must be greater than zero")]
    ZeroFormalAlternatives,
}

impl TextRuntimeBudget {
    pub fn new(
        max_formal_derivations: usize,
        max_formal_alternatives: usize,
    ) -> Result<Self, TextRuntimeBudgetError> {
        if max_formal_derivations == 0 {
            return Err(TextRuntimeBudgetError::ZeroFormalDerivations);
        }
        if max_formal_alternatives == 0 {
            return Err(TextRuntimeBudgetError::ZeroFormalAlternatives);
        }
        Ok(Self {
            max_formal_derivations,
            max_formal_alternatives,
        })
    }

    pub fn max_formal_derivations(&self) -> usize {
        self.max_formal_derivations
    }

    pub fn max_formal_alternatives(&self) -> usize {
        self.max_formal_alternatives
    }
}

impl Default for TextRuntimeBudget {
    fn default() -> Self {
        Self {
            max_formal_derivations: 1_024,
            max_formal_alternatives: 256,
        }
    }
}
