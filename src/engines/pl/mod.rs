pub mod parser;
pub mod generator;
pub mod morphology;

use crate::core::capability::Capability;
use crate::core::interlingua::*;
use crate::core::ontology::Ontology;
use crate::core::traits::{IMeaningRepresentation, LanguageKind};
use crate::data::descriptor::LanguageDescriptor;
use crate::data::lexicon::Lexicon;
use crate::engines::pl::generator::PolishGenerator;
use crate::engines::pl::morphology::PolishMorphology;
use crate::engines::pl::parser::PolishParser;
use crate::error::{GenerateError, ParseError};

pub struct PolishEngine {
    parser: PolishParser,
    generator: PolishGenerator,
    _descriptor: LanguageDescriptor,
    _lexicon: Lexicon,
    lang_id: LanguageId,
}

impl PolishEngine {
    pub fn new(
        lexicon: Lexicon,
        morphology: PolishMorphology,
        descriptor: LanguageDescriptor,
        ontology: Ontology,
        concept_ids: Vec<String>,
    ) -> Self {
        let parser = PolishParser::new(lexicon.clone(), morphology.clone(), ontology, descriptor.clone(), concept_ids);
        let generator = PolishGenerator::new(lexicon.clone(), morphology, descriptor.clone());

        Self {
            parser,
            generator,
            _descriptor: descriptor,
            _lexicon: lexicon,
            lang_id: LanguageId::new("pl"),
        }
    }
}

static PL_CAPABILITIES: &[Capability] = &[
    Capability::TemporalReference,
    Capability::Deixis,
    Capability::EmotionExpression,
    Capability::Pragmatics,
    Capability::ProDrop,
    Capability::FreeWordOrder,
    Capability::MorphologicalInflection,
    Capability::Negation,
    Capability::Coordination,
    Capability::Conditionality,
    Capability::Reference,
    Capability::Ambiguity,
    Capability::Quantification,
];

impl IMeaningRepresentation for PolishEngine {
    type Input = str;
    type Output = String;

    fn language_id(&self) -> &LanguageId {
        &self.lang_id
    }

    fn name(&self) -> &str {
        "Polish"
    }

    fn language_kind(&self) -> LanguageKind {
        LanguageKind::Natural
    }

    fn capabilities(&self) -> &[Capability] {
        PL_CAPABILITIES
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
