use std::collections::HashMap;

use crate::core::context;
use crate::core::graph::DialogueGraph;
use crate::core::interlingua::{Interlingua, LanguageId};
use crate::core::summary::trace_digest;
use crate::core::traits::IMeaningRepresentation;
use crate::error::LexFlexError;
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

        if let Interlingua::Natural(ref utt) = il {
            trace_digest("parse_complete", utt);
        }

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

        if let Interlingua::Natural(ref utt) = il {
            trace_digest("pre_generation", utt);
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
    ) -> Result<Interlingua, LexFlexError> {
        let source = self.engines.get(&from.0).ok_or_else(|| {
            LexFlexError::Translate(TranslateError::UnsupportedSourceLanguage {
                language: from.0.clone(),
            })
        })?;

        source
            .to_interlingua(input)
            .map_err(LexFlexError::Parse)
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

    pub fn parse_dialogue(
        &self,
        texts: &[&str],
        from: &LanguageId,
    ) -> Result<DialogueGraph, TranslateError> {
        let source = self.engines.get(&from.0).ok_or_else(|| {
            TranslateError::UnsupportedSourceLanguage {
                language: from.0.clone(),
            }
        })?;

        let mut utterances = Vec::new();
        for text in texts {
            let il = source.to_interlingua(text).map_err(|e| {
                TranslateError::InexpressibleInTarget {
                    target: from.0.clone(),
                    features: vec![format!("Parse error: {}", e)],
                }
            })?;
            if let Interlingua::Natural(u) = il {
                utterances.push(u);
            }
        }

        let mut dialogue = DialogueGraph {
            utterances,
            cross_edges: Vec::new(),
            utterance_node_ids: Vec::new(),
        };
        context::link_cross_utterance_context(&mut dialogue);
        Ok(dialogue)
    }

    pub fn translate_dialogue(
        &self,
        texts: &[&str],
        from: &LanguageId,
        to: &LanguageId,
    ) -> Result<Vec<String>, TranslateError> {
        let dialogue = self.parse_dialogue(texts, from)?;
        let target = self.engines.get(&to.0).ok_or_else(|| {
            TranslateError::UnsupportedTargetLanguage {
                language: to.0.clone(),
            }
        })?;

        let mut outputs = Vec::new();
        for utterance in &dialogue.utterances {
            let il = Interlingua::Natural(utterance.clone());
            let out = target.from_interlingua(&il).map_err(|e| {
                TranslateError::InexpressibleInTarget {
                    target: to.0.clone(),
                    features: vec![format!("Generate error: {}", e)],
                }
            })?;
            outputs.push(out);
        }
        Ok(outputs)
    }
}

impl Default for UniversalTranslator {
    fn default() -> Self {
        Self::new()
    }
}
