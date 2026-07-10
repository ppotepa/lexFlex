use std::collections::HashMap;

use crate::core::interlingua::{Interlingua, LanguageId};
use crate::core::traits::IMeaningRepresentation;
use crate::error::TranslateError;

pub struct UniversalTranslator {
    engines: HashMap<String, Box<dyn IMeaningRepresentation<Input = str, Output = String>>>,
}

impl UniversalTranslator {
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
        }
    }

    pub fn register_engine(
        &mut self,
        id: &str,
        engine: Box<dyn IMeaningRepresentation<Input = str, Output = String>>,
    ) {
        self.engines.insert(id.to_string(), engine);
    }

    pub fn translate(
        &self,
        input: &str,
        from: &LanguageId,
        to: &LanguageId,
    ) -> Result<String, TranslateError> {
        let source = self.engines.get(&from.0).ok_or_else(|| {
            TranslateError::UnsupportedSourceLanguage {
                language: from.0.clone(),
            }
        })?;

        let target = self.engines.get(&to.0).ok_or_else(|| {
            TranslateError::UnsupportedTargetLanguage {
                language: to.0.clone(),
            }
        })?;

        let il = source.to_interlingua(input).map_err(|e| {
            tracing::error!("Parse error: {}", e);
            TranslateError::InexpressibleInTarget {
                target: to.0.clone(),
                features: vec![format!("Parse error: {}", e)],
            }
        })?;

        let inexpressible = target.can_express(&il);
        if !inexpressible.is_empty() {
            return Err(TranslateError::InexpressibleInTarget {
                target: to.0.clone(),
                features: inexpressible
                    .iter()
                    .map(|f| format!("{:?}", f.capability))
                    .collect(),
            });
        }

        let output = target.from_interlingua(&il).map_err(|e| {
            tracing::error!("Generate error: {}", e);
            TranslateError::InexpressibleInTarget {
                target: to.0.clone(),
                features: vec![format!("Generate error: {}", e)],
            }
        })?;

        Ok(output)
    }

    pub fn parse_to_interlingua(
        &self,
        input: &str,
        from: &LanguageId,
    ) -> Result<Interlingua, TranslateError> {
        let source = self.engines.get(&from.0).ok_or_else(|| {
            TranslateError::UnsupportedSourceLanguage {
                language: from.0.clone(),
            }
        })?;

        source.to_interlingua(input).map_err(|e| {
            TranslateError::InexpressibleInTarget {
                target: from.0.clone(),
                features: vec![format!("Parse error: {}", e)],
            }
        })
    }

    pub fn generate_from_interlingua(
        &self,
        il: &Interlingua,
        to: &LanguageId,
    ) -> Result<String, TranslateError> {
        let target = self.engines.get(&to.0).ok_or_else(|| {
            TranslateError::UnsupportedTargetLanguage {
                language: to.0.clone(),
            }
        })?;

        target.from_interlingua(il).map_err(|e| {
            TranslateError::InexpressibleInTarget {
                target: to.0.clone(),
                features: vec![format!("Generate error: {}", e)],
            }
        })
    }

    pub fn supported_languages(&self) -> Vec<String> {
        self.engines.keys().cloned().collect()
    }
}

impl Default for UniversalTranslator {
    fn default() -> Self {
        Self::new()
    }
}
