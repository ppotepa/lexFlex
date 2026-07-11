use std::path::Path;

use crate::core::interlingua::{Interlingua, LanguageId};
use crate::data::loader;
use crate::engines::en::EnglishEngine;
use crate::engines::en::morphology::EnglishMorphology;
use crate::engines::pl::PolishEngine;
use crate::engines::pl::morphology::PolishMorphology;
use crate::error::LexFlexError;
use crate::translator::UniversalTranslator;

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
}

pub struct LexFlexBuilder {
    data_dir: Option<String>,
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
        self.data_dir = Some(dir.to_string());
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
        let data_dir = self.data_dir.clone().unwrap_or_else(|| "data".to_string());
        let data_path = Path::new(&data_dir);

        let mut translator = UniversalTranslator::new();

        let concepts_path = data_path.join("concepts/concepts.ron");
        let concepts = if concepts_path.exists() {
            loader::load_concepts(&concepts_path)?
        } else {
            Vec::new()
        };
        let ontology = loader::build_ontology_from_concepts(&concepts);

        if self.enable_pl {
            let pl_lexicon = self.load_lexicon(data_path, "pl")?;
            let pl_descriptor = self.load_descriptor(data_path, "pl")?;
            let pl_noun_paradigms = self.load_morphology(data_path, "pl", "noun_paradigms")?;
            let pl_verb_paradigms = self.load_morphology(data_path, "pl", "verb_paradigms")?;
            let pl_adj_paradigms = self.load_morphology(data_path, "pl", "adj_paradigms")?;

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
            );

            translator.register_engine("pl", Box::new(pl_engine));
        }

        if self.enable_en {
            let en_lexicon = self.load_lexicon(data_path, "en")?;
            let en_descriptor = self.load_descriptor(data_path, "en")?;
            let en_verb_paradigms = self.load_morphology(data_path, "en", "verb_paradigms")?;
            let en_noun_paradigms = self.load_morphology(data_path, "en", "noun_paradigms")?;

            let en_morphology = EnglishMorphology::new(
                en_verb_paradigms,
                en_noun_paradigms,
            );

            let en_engine = EnglishEngine::new(
                en_lexicon,
                en_morphology,
                en_descriptor,
                ontology.clone(),
            );

            translator.register_engine("en", Box::new(en_engine));
        }

        Ok(LexFlexAPI { translator })
    }

    fn load_lexicon(&self, data_path: &Path, lang: &str) -> Result<crate::data::lexicon::Lexicon, LexFlexError> {
        let path = data_path.join(format!("lexicons/{}/lexicon.ron", lang));
        if path.exists() {
            Ok(loader::load_lexicon(&path)?)
        } else {
            Ok(crate::data::lexicon::Lexicon::new())
        }
    }

    fn load_descriptor(&self, data_path: &Path, lang: &str) -> Result<crate::data::descriptor::LanguageDescriptor, LexFlexError> {
        let path = data_path.join(format!("descriptors/{}.ron", lang));
        if path.exists() {
            Ok(loader::load_descriptor(&path)?)
        } else {
            Ok(crate::data::descriptor::LanguageDescriptor {
                language: lang.to_string(),
                name: lang.to_string(),
                morphology: crate::data::descriptor::MorphologyDescriptor {
                    has_cases: lang == "pl",
                    cases: if lang == "pl" {
                        vec![
                            crate::core::interlingua::Case::Nominative,
                            crate::core::interlingua::Case::Genitive,
                            crate::core::interlingua::Case::Dative,
                            crate::core::interlingua::Case::Accusative,
                            crate::core::interlingua::Case::Instrumental,
                            crate::core::interlingua::Case::Locative,
                            crate::core::interlingua::Case::Vocative,
                        ]
                    } else {
                        vec![]
                    },
                    has_articles: lang == "en",
                    aspect_type: if lang == "pl" {
                        crate::data::descriptor::AspectType::Morphological
                    } else {
                        crate::data::descriptor::AspectType::Periphrastic
                    },
                },
                syntax: crate::data::descriptor::SyntaxDescriptor {
                    word_order: crate::data::descriptor::WordOrder::SVO,
                    pro_drop: lang == "pl",
                    negation_particle: if lang == "pl" { "nie" } else { "not" }.to_string(),
                    preposition_roles: std::collections::HashMap::new(),
                },
            })
        }
    }

    fn load_morphology(
        &self,
        data_path: &Path,
        lang: &str,
        name: &str,
    ) -> Result<Vec<crate::data::morphology::MorphParadigm>, LexFlexError> {
        let path = data_path.join(format!("morphology/{}/{}.ron", lang, name));
        if path.exists() {
            Ok(loader::load_paradigms(&path)?)
        } else {
            Ok(Vec::new())
        }
    }
}

impl Default for LexFlexBuilder {
    fn default() -> Self {
        Self::new()
    }
}
