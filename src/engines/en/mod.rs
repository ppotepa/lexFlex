pub mod parser;
pub mod generator;
pub mod morphology;

use crate::core::capability::Capability;
use crate::core::interlingua::*;
use crate::core::ontology::Ontology;
use crate::core::traits::{IMeaningRepresentation, LanguageKind};
use crate::data::descriptor::LanguageDescriptor;
use crate::data::lexicon::Lexicon;
use crate::engines::en::generator::EnglishGenerator;
use crate::engines::en::morphology::EnglishMorphology;
use crate::engines::en::parser::EnglishParser;
use crate::error::{GenerateError, ParseError};

pub struct EnglishEngine {
    parser: EnglishParser,
    generator: EnglishGenerator,
    _descriptor: LanguageDescriptor,
    lang_id: LanguageId,
}

impl EnglishEngine {
    pub fn new(
        lexicon: Lexicon,
        morphology: EnglishMorphology,
        descriptor: LanguageDescriptor,
        ontology: Ontology,
    ) -> Self {
        let parser = EnglishParser::new(lexicon.clone(), EnglishMorphology::new(Vec::new(), Vec::new()), ontology);
        let generator = EnglishGenerator::new(lexicon, morphology, descriptor.clone());

        Self {
            parser,
            generator,
            _descriptor: descriptor,
            lang_id: LanguageId::new("en"),
        }
    }
}

static EN_CAPABILITIES: &[Capability] = &[
    Capability::TemporalReference,
    Capability::Deixis,
    Capability::EmotionExpression,
    Capability::Pragmatics,
    Capability::Negation,
    Capability::Coordination,
    Capability::Conditionality,
    Capability::Reference,
    Capability::Quantification,
    Capability::MorphologicalInflection,
    Capability::ProDrop,
    Capability::FreeWordOrder,
    Capability::Ambiguity,
];

impl IMeaningRepresentation for EnglishEngine {
    type Input = str;
    type Output = String;

    fn language_id(&self) -> &LanguageId {
        &self.lang_id
    }

    fn name(&self) -> &str {
        "English"
    }

    fn language_kind(&self) -> LanguageKind {
        LanguageKind::Natural
    }

    fn capabilities(&self) -> &[Capability] {
        EN_CAPABILITIES
    }

    fn limitations(&self) -> &[crate::core::capability::Limitation] {
        &[]
    }

    fn to_interlingua(&self, input: &str) -> Result<Interlingua, ParseError> {
        let utterance = self.parser.parse(input)?;
        Ok(Interlingua::Natural(utterance))
    }

    fn from_interlingua(&self, il: &Interlingua) -> Result<String, GenerateError> {
        match il {
            Interlingua::Natural(utterance) => self.generator.generate(utterance),
            _ => Err(GenerateError::UnsupportedInterlinguaType),
        }
    }
}
