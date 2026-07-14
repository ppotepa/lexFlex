use crate::document::graph::{DocumentMentionKind, DocumentMentionNode};

use super::model::MentionSourceForm;

pub trait LanguageReferenceProfile {
    fn classify(&self, mention: &DocumentMentionNode) -> MentionSourceForm;
    fn is_pro_drop(&self) -> bool {
        false
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PolishReferenceProfile;

#[derive(Debug, Default, Clone, Copy)]
pub struct EnglishReferenceProfile;

#[derive(Debug, Default, Clone, Copy)]
pub struct NeutralReferenceProfile;

impl LanguageReferenceProfile for NeutralReferenceProfile {
    fn classify(&self, mention: &DocumentMentionNode) -> MentionSourceForm {
        classify_mention(mention)
    }
}

impl LanguageReferenceProfile for EnglishReferenceProfile {
    fn classify(&self, mention: &DocumentMentionNode) -> MentionSourceForm {
        classify_mention(mention)
    }
}

impl LanguageReferenceProfile for PolishReferenceProfile {
    fn classify(&self, mention: &DocumentMentionNode) -> MentionSourceForm {
        classify_mention(mention)
    }

    fn is_pro_drop(&self) -> bool {
        true
    }
}

pub fn classify_mention(mention: &DocumentMentionNode) -> MentionSourceForm {
    let name = mention.normalized_name.as_deref().unwrap_or("");
    if matches!(mention.mention_kind, DocumentMentionKind::Modifier) {
        return MentionSourceForm::Modifier;
    }
    if matches!(mention.mention_kind, DocumentMentionKind::CoordinationGroup) {
        return MentionSourceForm::Group;
    }
    if is_reflexive_name(name) {
        return MentionSourceForm::ReflexivePronoun;
    }
    if is_pronoun_name(name) {
        return MentionSourceForm::Pronoun;
    }
    if is_possessive_name(name) {
        return MentionSourceForm::PossessivePronoun;
    }
    if is_demonstrative_name(name) {
        return MentionSourceForm::DemonstrativeDescription;
    }
    if mention.features.definiteness == Some(crate::core::interlingua::Definiteness::Definite) {
        return MentionSourceForm::DefiniteDescription;
    }
    if mention.name.is_some() && !name.is_empty() && !name.contains(' ') {
        return MentionSourceForm::ProperName;
    }
    if matches!(mention.reference, crate::core::interlingua::Reference::Generic) {
        return MentionSourceForm::Generic;
    }
    if matches!(mention.reference, crate::core::interlingua::Reference::Anaphoric(_)) {
        return MentionSourceForm::BareCommonNoun;
    }
    if matches!(mention.reference, crate::core::interlingua::Reference::Cataphoric(_)) {
        return MentionSourceForm::DemonstrativeDescription;
    }
    MentionSourceForm::Unknown
}

fn is_pronoun_name(name: &str) -> bool {
    matches!(
        name,
        "i" | "me"
            | "you"
            | "he"
            | "him"
            | "she"
            | "her"
            | "it"
            | "we"
            | "us"
            | "they"
            | "them"
            | "ja"
            | "mnie"
            | "mi"
            | "ty"
            | "ciebie"
            | "ci"
            | "on"
            | "go"
            | "jego"
            | "mu"
            | "ona"
            | "ją"
            | "jej"
            | "ono"
            | "je"
            | "my"
            | "nas"
            | "nam"
            | "wy"
            | "was"
            | "wam"
            | "oni"
            | "one"
            | "ich"
            | "im"
    )
}

fn is_possessive_name(name: &str) -> bool {
    matches!(name, "my" | "your" | "his" | "her" | "its" | "our" | "their" | "swój")
}

fn is_reflexive_name(name: &str) -> bool {
    matches!(name, "myself" | "yourself" | "himself" | "herself" | "itself" | "ourselves" | "themselves" | "się" | "siebie" | "sobie" | "sobą")
}

fn is_demonstrative_name(name: &str) -> bool {
    matches!(name, "this" | "that" | "these" | "those" | "ten" | "ta" | "to" | "tamten")
}
