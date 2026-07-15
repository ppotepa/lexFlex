use std::path::{Path, PathBuf};

use crate::core::graph::DialogueGraph;
use crate::core::interlingua::{Interlingua, LanguageId, Utterance};
use crate::core::summary::{
    format_digest_human, format_digest_json, summarize_utterance, SemanticDigest,
};
use crate::data::loader;
use crate::engines::en::EnglishEngine;
use crate::engines::en::morphology::EnglishMorphology;
use crate::engines::pl::PolishEngine;
use crate::engines::pl::morphology::PolishMorphology;
use crate::error::LexFlexError;
use crate::translator::UniversalTranslator;

mod document;

pub struct LexFlexAPI {
    translator: UniversalTranslator,
}

impl LexFlexAPI {
    pub fn builder() -> LexFlexBuilder {
        LexFlexBuilder::new()
    }

    pub fn translate(
        &self,
        input: &str,
        from: &str,
        to: &str,
    ) -> Result<String, LexFlexError> {
        self.compile(input, from, to)
    }

    /// Full parse → discourse resolution → target compilation pipeline.
    pub fn compile(
        &self,
        input: &str,
        from: &str,
        to: &str,
    ) -> Result<String, LexFlexError> {
        let from_id = LanguageId::new(from);
        let to_id = LanguageId::new(to);
        Ok(self.translator.translate(input, &from_id, &to_id)?)
    }

    pub fn parse(
        &self,
        input: &str,
        lang: &str,
    ) -> Result<Interlingua, LexFlexError> {
        let lang_id = LanguageId::new(lang);
        Ok(self.translator.parse_to_interlingua(input, &lang_id)?)
    }

    pub fn generate(
        &self,
        il: &Interlingua,
        lang: &str,
    ) -> Result<String, LexFlexError> {
        let lang_id = LanguageId::new(lang);
        Ok(self.translator.generate_from_interlingua(il, &lang_id)?)
    }

    pub fn supported_languages(&self) -> Vec<String> {
        self.translator.supported_languages()
    }

    /// Parse multiple utterances and build a dialogue graph with cross-utterance context.
    pub fn parse_dialogue(
        &self,
        texts: &[&str],
        lang: &str,
    ) -> Result<DialogueGraph, LexFlexError> {
        Ok(self.translator.parse_dialogue(texts, &LanguageId::new(lang))?)
    }

    /// Parse multi-sentence text as a single utterance with discourse tracking.
    pub fn parse_multi_sentence(
        &self,
        input: &str,
        lang: &str,
    ) -> Result<Utterance, LexFlexError> {
        let il = self.parse(input, lang)?;
        il.as_natural()
            .cloned()
            .ok_or_else(|| {
                LexFlexError::Translate(crate::error::TranslateError::InexpressibleInTarget {
                    target: lang.to_string(),
                    features: vec!["Expected natural language utterance".to_string()],
                })
            })
    }

    /// IL-derived semantic digest after full parse → deduction → discourse resolution.
    pub fn explain(&self, input: &str, lang: &str) -> Result<SemanticDigest, LexFlexError> {
        let utt = self.parse_multi_sentence(input, lang)?;
        Ok(summarize_utterance(&utt))
    }

    /// Human-readable semantic digest (chat/CLI default).
    pub fn explain_human(&self, input: &str, lang: &str) -> Result<String, LexFlexError> {
        Ok(format_digest_human(&self.explain(input, lang)?))
    }

    /// JSON semantic digest.
    pub fn explain_json(&self, input: &str, lang: &str) -> Result<String, LexFlexError> {
        Ok(format_digest_json(&self.explain(input, lang)?))
    }

    /// Translate a sequence of utterances, carrying dialogue context.
    pub fn translate_dialogue(
        &self,
        texts: &[&str],
        from: &str,
        to: &str,
    ) -> Result<Vec<String>, LexFlexError> {
        Ok(self
            .translator
            .translate_dialogue(texts, &LanguageId::new(from), &LanguageId::new(to))?)
    }
}

pub struct LexFlexBuilder {
    data_dir: Option<PathBuf>,
    enable_pl: bool,
    enable_en: bool,
}

impl LexFlexBuilder {
    pub fn new() -> Self {
        Self {
            data_dir: None,
            enable_pl: true,
            enable_en: true,
        }
    }

    pub fn data_dir(mut self, dir: &str) -> Self {
        self.data_dir = Some(PathBuf::from(dir));
        self
    }

    pub fn with_polish(mut self) -> Self {
        self.enable_pl = true;
        self
    }

    pub fn with_english(mut self) -> Self {
        self.enable_en = true;
        self
    }

    pub fn build(self) -> Result<LexFlexAPI, LexFlexError> {
        let data_path = if let Some(path) = self.data_dir.clone() {
            let canonical = std::fs::canonicalize(&path).map_err(|error| {
                LexFlexError::Data(crate::error::DataError::InvalidDataRoot {
                    path: path.display().to_string(),
                    message: error.to_string(),
                })
            })?;
            crate::data::layout::validate_data_root(&canonical)?;
            canonical
        } else {
            crate::data::layout::resolve_data_root(None)?.path
        };

        let mut translator = UniversalTranslator::new();

        let concepts_path = data_path.join("concepts/concepts.ron");
        let concepts = loader::load_concepts(&concepts_path)?;
        let ontology_path = data_path.join("ontology/ontology.ron");
        let ontology = loader::load_ontology(&ontology_path)?;
        let mut concept_ids = concepts.iter().map(|c| c.id.clone()).collect::<Vec<_>>();
        concept_ids.extend(ontology.concept_ids());
        concept_ids.sort();
        concept_ids.dedup();

        if self.enable_pl {
            let pl_lexicon = self.load_lexicon(&data_path, "pl")?;
            let pl_descriptor = self.load_descriptor(&data_path, "pl")?;
            let pl_noun_paradigms = self.load_morphology(&data_path, "pl", "noun_paradigms")?;
            let pl_verb_paradigms = self.load_morphology(&data_path, "pl", "verb_paradigms")?;
            let pl_adj_paradigms = self.load_morphology(&data_path, "pl", "adj_paradigms")?;

            let pl_morphology = PolishMorphology::new(
                pl_noun_paradigms,
                pl_verb_paradigms,
                pl_adj_paradigms,
            );

            let pl_engine = PolishEngine::new(
                pl_lexicon,
                pl_morphology,
                pl_descriptor,
                ontology.clone(),
                concept_ids.clone(),
            );

            translator.register_engine("pl", Box::new(pl_engine));
        }

        if self.enable_en {
            let en_lexicon = self.load_lexicon(&data_path, "en")?;
            let en_descriptor = self.load_descriptor(&data_path, "en")?;
            let en_verb_paradigms = self.load_morphology(&data_path, "en", "verb_paradigms")?;
            let en_noun_paradigms = self.load_morphology(&data_path, "en", "noun_paradigms")?;

            let en_morphology = EnglishMorphology::new(
                en_verb_paradigms,
                en_noun_paradigms,
            );

            let en_engine = EnglishEngine::new(
                en_lexicon,
                en_morphology,
                en_descriptor,
                ontology.clone(),
                concept_ids.clone(),
            );

            translator.register_engine("en", Box::new(en_engine));
        }

        Ok(LexFlexAPI { translator })
    }

    fn load_lexicon(&self, data_path: &Path, lang: &str) -> Result<crate::data::lexicon::Lexicon, LexFlexError> {
        let path = data_path.join(format!("lexicons/{}/lexicon.ron", lang));
        Ok(loader::load_lexicon(&path)?)
    }

    fn load_descriptor(&self, data_path: &Path, lang: &str) -> Result<crate::data::descriptor::LanguageDescriptor, LexFlexError> {
        let path = data_path.join(format!("descriptors/{}.ron", lang));
        Ok(loader::load_descriptor(&path)?)
    }

    fn load_morphology(
        &self,
        data_path: &Path,
        lang: &str,
        name: &str,
    ) -> Result<Vec<crate::data::morphology::MorphParadigm>, LexFlexError> {
        let path = data_path.join(format!("morphology/{}/{}.ron", lang, name));
        Ok(loader::load_paradigms(&path)?)
    }

}

impl Default for LexFlexBuilder {
    fn default() -> Self {
        Self::new()
    }
}
