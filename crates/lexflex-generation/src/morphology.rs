use lexflex_language::FeatureStructure;
use lexflex_language::{CompiledLexicalSense, Form, LanguageModel};

pub(crate) fn select_form<'a>(
    sense: &CompiledLexicalSense,
    features: &FeatureStructure,
    language: &'a LanguageModel,
) -> Option<&'a Form> {
    language
        .forms
        .values()
        .filter(|form| form.lexeme_id == sense.lexeme_id && form.features.contains_all(features))
        .min_by_key(|form| (form.priority, form.id.clone()))
}
