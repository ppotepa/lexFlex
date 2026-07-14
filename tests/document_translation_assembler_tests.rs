mod support;

use std::collections::BTreeMap;

use lexflex::document::compilation::DocumentCompiler;
use lexflex::document::translation::{BestEffortDocumentTranslator, DocumentOutputAssembler, DocumentSentenceGenerator, SentenceGenerationError, SentenceGenerationInput};
use lexflex::document::{SentenceId};
use support::{segmented, AlwaysSuccessAnalyzer};

struct EchoGenerator;

impl DocumentSentenceGenerator for EchoGenerator {
    fn generator_id(&self) -> &'static str {
        "echo"
    }

    fn generate(
        &self,
        input: SentenceGenerationInput<'_>,
    ) -> Result<String, SentenceGenerationError> {
        let _ = input;
        Ok("X".to_string())
    }
}

#[test]
fn assembler_preserves_inter_sentence_spacing() {
    let document = segmented("Ala ma kota.   Kot śpi.");
    let ids: Vec<SentenceId> = document
        .ordered_sentences()
        .into_iter()
        .map(|sentence| sentence.id.clone())
        .collect();
    let generated = BTreeMap::from([
        (ids[0].clone(), "Alice has a cat.".to_string()),
        (ids[1].clone(), "The cat sleeps.".to_string()),
    ]);
    let output = DocumentOutputAssembler::assemble(&document, &generated).unwrap();
    assert_eq!(output.text, "Alice has a cat.   The cat sleeps.");
}

#[test]
fn fallback_translation_is_lossless_by_default() {
    let document = segmented("Ala ma kota. Kot śpi.");
    let compilation = DocumentCompiler::new(AlwaysSuccessAnalyzer)
        .compile(document)
        .unwrap();
    let translation = BestEffortDocumentTranslator::new(EchoGenerator)
        .translate(&compilation, lexflex::core::interlingua::LanguageId::new("en"))
        .unwrap();
    assert!(!translation.output.is_empty());
    assert_eq!(translation.summary.total_sentences, 2);
}
