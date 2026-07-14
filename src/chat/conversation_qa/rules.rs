use super::{
    ConceptRef, ConversationEvidence, ConversationSourceId, ConversationSourceKind, EntityGender,
    EntityNumber, EntityRef, FactObject, PredicateId, SessionFact,
};
use crate::document::span::SourceSpan;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct SentenceFragment {
    pub text: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PronounKind {
    It,
    He,
    She,
    They,
    Its,
    His,
    Her,
    On,
    Ona,
    Ono,
    To,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityPattern {
    Literal(String),
    Pronoun(PronounKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedQuestion {
    SubjectLookup {
        predicate: PredicateId,
        object: EntityPattern,
    },
    ObjectLookup {
        subject: EntityPattern,
        predicate: PredicateId,
    },
    Boolean {
        subject: EntityPattern,
        predicate: PredicateId,
        object: EntityPattern,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedQuestion {
    SubjectLookup {
        predicate: PredicateId,
        object: EntityRef,
    },
    ObjectLookup {
        subject: EntityRef,
        predicate: PredicateId,
    },
    Boolean {
        subject: EntityRef,
        predicate: PredicateId,
        object: EntityRef,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityState {
    pub entity: EntityRef,
    pub mention_count: usize,
    pub last_seen: usize,
    pub salience_score: usize,
}

impl ResolvedQuestion {
    pub fn matches(&self, fact: &SessionFact) -> bool {
        match self {
            Self::SubjectLookup { predicate, object } => {
                &fact.predicate == predicate
                    && matches!(&fact.object, FactObject::Entity(entity) if entity.key == object.key)
            }
            Self::ObjectLookup { subject, predicate } => {
                &fact.predicate == predicate && fact.subject.key == subject.key
            }
            Self::Boolean {
                subject,
                predicate,
                object,
            } => {
                &fact.predicate == predicate
                    && fact.subject.key == subject.key
                    && matches!(&fact.object, FactObject::Entity(entity) if entity.key == object.key)
            }
        }
    }
}

pub fn is_polish(language: &str) -> bool {
    super::language::is_polish(language)
}

pub fn normalize_entity_label(value: &str) -> String {
    let trimmed = value.trim().trim_matches(|ch: char| {
        matches!(
            ch,
            '.' | ',' | ';' | ':' | '"' | '\'' | ')' | '(' | '[' | ']' | '{' | '}'
        )
    });
    collapse_whitespace(trimmed)
}

pub fn normalize_entity_key(value: &str) -> String {
    let mut normalized = normalize_entity_label(value).to_lowercase();
    for prefix in ["the ", "a ", "an ", "ta ", "ten ", "to "] {
        if normalized.starts_with(prefix) {
            normalized = normalized[prefix.len()..].to_string();
            break;
        }
    }
    normalized
}

pub fn normalize_text_key(value: &str) -> String {
    collapse_whitespace(value.trim()).to_lowercase()
}

pub fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn guess_entity_number(value: &str) -> EntityNumber {
    let key = normalize_entity_key(value);
    if key.split_whitespace().any(|part| matches!(part, "people" | "inhabitants" | "cities" | "countries")) {
        return EntityNumber::Plural;
    }
    if key.contains(" and ") || key.contains(", ") {
        return EntityNumber::Plural;
    }
    if key.ends_with(" people") || key.ends_with(" inhabitants") {
        return EntityNumber::Plural;
    }
    EntityNumber::Singular
}

pub fn guess_entity_gender(value: &str) -> EntityGender {
    let key = normalize_entity_key(value);
    if matches!(key.as_str(), "man" | "boy" | "father" | "king" | "mr" | "pan") {
        return EntityGender::Masculine;
    }
    if matches!(key.as_str(), "woman" | "girl" | "mother" | "queen" | "mrs" | "ms" | "pani") {
        return EntityGender::Feminine;
    }
    if key.ends_with("a") && !key.ends_with("ia") && !key.ends_with("ea") {
        return EntityGender::Feminine;
    }
    EntityGender::Unknown
}

pub fn render_decimal(value: &crate::document::knowledge::DecimalValue) -> String {
    let mut digits = value.mantissa.to_string();
    if value.scale > 0 {
        let scale = value.scale as usize;
        if digits.len() <= scale {
            digits = format!("{}{}", "0".repeat(scale + 1 - digits.len()), digits);
        }
        let split = digits.len() - scale;
        digits.insert(split, '.');
    }
    if value.sign < 0 && value.mantissa != 0 {
        format!("-{digits}")
    } else {
        digits
    }
}

pub fn split_sentences_with_spans(text: &str) -> Vec<SentenceFragment> {
    let mut fragments = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        let is_boundary = (matches!(ch, '.' | '!' | '?')
            && !(ch == '.' && looks_like_abbreviation(text, index)))
            || (ch == '\n' && chars.peek().is_some());
        if is_boundary {
            let end = index + ch.len_utf8();
            push_fragment(text, start, end, &mut fragments);
            start = end;
        }
    }
    if start < text.len() {
        push_fragment(text, start, text.len(), &mut fragments);
    }
    fragments
}

fn looks_like_abbreviation(text: &str, period: usize) -> bool {
    let before = &text[..period];
    let token = before
        .rsplit(|ch: char| ch.is_whitespace() || matches!(ch, '(' | '[' | '{'))
        .next()
        .unwrap_or("");
    let has_following_text = text[period + 1..].chars().any(|ch| !ch.is_whitespace());
    has_following_text
        && !token.is_empty()
        && token.chars().count() <= 3
        && token.chars().all(char::is_alphabetic)
}

fn push_fragment(text: &str, start: usize, end: usize, fragments: &mut Vec<SentenceFragment>) {
    if start >= end {
        return;
    }
    let span = SourceSpan { start, end };
    let Ok(slice) = span.slice(text) else {
        return;
    };
    let trimmed_start = slice
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(offset, _)| start + offset)
        .unwrap_or(end);
    let trimmed_end = slice
        .char_indices()
        .rev()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(offset, ch)| start + offset + ch.len_utf8())
        .unwrap_or(trimmed_start);
    if trimmed_start < trimmed_end {
        fragments.push(SentenceFragment {
            text: text[trimmed_start..trimmed_end].to_string(),
            span: SourceSpan {
                start: trimmed_start,
                end: trimmed_end,
            },
        });
    }
}

pub fn looks_like_question(text: &str, language: &str) -> bool {
    let trimmed = text.trim();
    trimmed.ends_with('?')
        || if is_polish(language) {
            matches!(
                trimmed.to_lowercase().as_str(),
                s if s.starts_with("czy ")
                    || s.starts_with("jaka ")
                    || s.starts_with("jakie ")
                    || s.starts_with("ile ")
                    || s.starts_with("ilu ")
                    || s.starts_with("gdzie ")
                    || s.starts_with("kto ")
                    || s.starts_with("co ")
            )
        } else {
            matches!(
                trimmed.to_lowercase().as_str(),
                s if s.starts_with("what ")
                    || s.starts_with("who ")
                    || s.starts_with("where ")
                    || s.starts_with("how many ")
                    || s.starts_with("is ")
            )
        }
}

pub fn parse_question(text: &str, language: &str) -> Option<ParsedQuestion> {
    let normalized = normalize_question(text);
    if is_polish(language) {
        parse_polish_question(&normalized)
    } else {
        parse_english_question(&normalized)
    }
}

/// Returns the concrete entity whose Wikipedia page is most likely to answer
/// a parsed question. Pronoun-only questions wait for local context instead.
pub fn question_lookup_title(text: &str, language: &str) -> Option<String> {
    let parsed = parse_question(text, language)?;
    let pattern = match parsed {
        ParsedQuestion::SubjectLookup { object, .. }
        | ParsedQuestion::Boolean { object, .. } => object,
        ParsedQuestion::ObjectLookup { subject, .. } => subject,
    };
    match pattern {
        EntityPattern::Literal(value) => Some(normalize_entity_label(&value)),
        EntityPattern::Pronoun(_) => None,
    }
}

pub fn extract_facts_from_sentence(
    sentence: &str,
    source_id: ConversationSourceId,
    source_label: &str,
    message_id: Option<usize>,
    source_url: Option<&str>,
    sentence_index: usize,
    span: SourceSpan,
    language: &str,
    entities: &BTreeMap<String, EntityState>,
) -> Vec<SessionFact> {
    let mut facts = vec![];
    if let Some((subject, object)) = parse_capital_of_statement(sentence, language, entities) {
        facts.push(build_fact(
            subject.clone(),
            PredicateId::CapitalOf,
            FactObject::Entity(object),
            source_id,
            source_label,
            message_id,
            source_url,
            sentence_index,
            span,
            sentence,
        ));
        facts.push(build_fact(
            subject.clone(),
            PredicateId::IsA,
            FactObject::Concept(ConceptRef::new("capital", "Capital")),
            source_id,
            source_label,
            message_id,
            source_url,
            sentence_index,
            span,
            sentence,
        ));
        if sentence_mentions_city(sentence, language) {
            facts.push(build_fact(
                subject,
                PredicateId::IsA,
                FactObject::Concept(ConceptRef::new("city", "City")),
                source_id,
                source_label,
                message_id,
                source_url,
                sentence_index,
                span,
                sentence,
            ));
        }
        return facts;
    }
    if let Some((subject, object)) = parse_population_statement(sentence, language, entities) {
        facts.push(build_fact(
            subject,
            PredicateId::Population,
            FactObject::Integer(object),
            source_id,
            source_label,
            message_id,
            source_url,
            sentence_index,
            span,
            sentence,
        ));
        return facts;
    }
    if let Some((subject, object)) = parse_located_in_statement(sentence, language, entities) {
        facts.push(build_fact(
            subject,
            PredicateId::LocatedIn,
            FactObject::Entity(object),
            source_id,
            source_label,
            message_id,
            source_url,
            sentence_index,
            span,
            sentence,
        ));
        return facts;
    }
    if let Some((subject, object)) = parse_is_a_statement(sentence, language, entities) {
        facts.push(build_fact(
            subject,
            PredicateId::IsA,
            FactObject::Concept(object),
            source_id,
            source_label,
            message_id,
            source_url,
            sentence_index,
            span,
            sentence,
        ));
        return facts;
    }
    facts
}

fn build_fact(
    subject: EntityRef,
    predicate: PredicateId,
    object: FactObject,
    source_id: ConversationSourceId,
    source_label: &str,
    message_id: Option<usize>,
    source_url: Option<&str>,
    sentence_index: usize,
    span: SourceSpan,
    sentence: &str,
) -> SessionFact {
    SessionFact {
        id: super::FactId(0),
        subject,
        predicate,
        object,
        evidence: vec![ConversationEvidence {
            source_id,
            source_label: source_label.to_string(),
            source_kind: if message_id.is_some() {
                ConversationSourceKind::Conversation
            } else {
                ConversationSourceKind::Wikipedia
            },
            message_id,
            source_url: source_url.map(str::to_string),
            sentence_index,
            span,
            text: sentence.to_string(),
        }],
        confidence_milli: 950,
    }
}

pub fn render_fact(fact: &SessionFact, language: &str) -> String {
    match (&fact.predicate, &fact.object) {
        (PredicateId::CapitalOf, FactObject::Entity(object)) => {
            if is_polish(language) {
                format!("{} jest stolicą {}.", fact.subject.label, object.label)
            } else {
                format!("{} is the capital of {}.", fact.subject.label, object.label)
            }
        }
        (PredicateId::Population, FactObject::Integer(value)) => {
            if is_polish(language) {
                format!("{} ma populację {} osób.", fact.subject.label, value)
            } else {
                format!("{} has a population of {} people.", fact.subject.label, value)
            }
        }
        (PredicateId::LocatedIn, FactObject::Entity(object)) => {
            if is_polish(language) {
                format!("{} znajduje się w {}.", fact.subject.label, object.label)
            } else {
                format!("{} is located in {}.", fact.subject.label, object.label)
            }
        }
        (PredicateId::IsA, FactObject::Entity(object)) => {
            if is_polish(language) {
                format!("{} jest {}.", fact.subject.label, object.label)
            } else {
                format!("{} is a {}.", fact.subject.label, object.label)
            }
        }
        (PredicateId::IsA, FactObject::Concept(concept)) => {
            if is_polish(language) {
                format!("{} to {}.", fact.subject.label, concept.render_short(language))
            } else {
                format!("{} is a {}.", fact.subject.label, concept.render_short(language))
            }
        }
        _ => format!(
            "{} {:?} {}",
            fact.subject.label,
            fact.predicate,
            fact.object.render_short(language)
        ),
    }
}

pub fn format_evidence_note(item: &ConversationEvidence) -> String {
    match item.message_id {
        Some(message_id) => format!(
            "{} message {}, sentence {} [{}..{}]",
            match item.source_kind {
                ConversationSourceKind::Wikipedia => "Wikipedia:",
                ConversationSourceKind::Conversation => "Conversation:",
            },
            message_id,
            item.sentence_index + 1,
            item.span.start,
            item.span.end
        ),
        None => format!(
            "{} sentence {} [{}..{}]",
            item.source_label,
            item.sentence_index + 1,
            item.span.start,
            item.span.end
        ),
    }
}

fn normalize_question(text: &str) -> String {
    collapse_whitespace(text.trim().trim_end_matches('?'))
        .trim()
        .to_string()
}

fn parse_english_question(text: &str) -> Option<ParsedQuestion> {
    if let Some(object) = tail_after_prefix(text, "what is the capital of ") {
        return Some(ParsedQuestion::SubjectLookup {
            predicate: PredicateId::CapitalOf,
            object: EntityPattern::Literal(object.to_string()),
        });
    }
    if let Some(subject) = tail_after_prefix(text, "what is the population of ") {
        return Some(ParsedQuestion::ObjectLookup {
            subject: EntityPattern::from_text(subject),
            predicate: PredicateId::Population,
        });
    }
    if text.to_lowercase().starts_with("what is its population") {
        return Some(ParsedQuestion::ObjectLookup {
            subject: EntityPattern::Pronoun(PronounKind::Its),
            predicate: PredicateId::Population,
        });
    }
    if text.to_lowercase().starts_with("what is his population") {
        return Some(ParsedQuestion::ObjectLookup {
            subject: EntityPattern::Pronoun(PronounKind::His),
            predicate: PredicateId::Population,
        });
    }
    if text.to_lowercase().starts_with("what is her population") {
        return Some(ParsedQuestion::ObjectLookup {
            subject: EntityPattern::Pronoun(PronounKind::Her),
            predicate: PredicateId::Population,
        });
    }
    if let Some(subject) = tail_after_prefix(text, "where is ") {
        return Some(ParsedQuestion::ObjectLookup {
            subject: EntityPattern::from_text(subject),
            predicate: PredicateId::LocatedIn,
        });
    }
    if let Some(rest) = tail_after_prefix(text, "is ") {
        if let Some((subject, object)) = split_case_insensitive(rest, " the capital of ") {
            return Some(ParsedQuestion::Boolean {
                subject: EntityPattern::from_text(subject),
                predicate: PredicateId::CapitalOf,
                object: EntityPattern::from_text(object),
            });
        }
    }
    if let Some(subject) = tail_after_prefix(text, "how many inhabitants does ") {
        let subject = subject.trim_end_matches(" have").trim();
        return Some(ParsedQuestion::ObjectLookup {
            subject: EntityPattern::from_text(subject),
            predicate: PredicateId::Population,
        });
    }
    if let Some(subject) = tail_after_prefix(text, "what is ") {
        return Some(ParsedQuestion::ObjectLookup {
            subject: EntityPattern::from_text(subject),
            predicate: PredicateId::IsA,
        });
    }
    if let Some(subject) = tail_after_prefix(text, "who is ") {
        return Some(ParsedQuestion::ObjectLookup {
            subject: EntityPattern::from_text(subject),
            predicate: PredicateId::IsA,
        });
    }
    None
}

fn parse_polish_question(text: &str) -> Option<ParsedQuestion> {
    if let Some(object) = tail_after_prefix(text, "jaka jest stolica ") {
        return Some(ParsedQuestion::SubjectLookup {
            predicate: PredicateId::CapitalOf,
            object: EntityPattern::Literal(object.to_string()),
        });
    }
    if let Some(subject) = tail_after_prefix(text, "jaka jest populacja ") {
        return Some(ParsedQuestion::ObjectLookup {
            subject: EntityPattern::from_text(subject),
            predicate: PredicateId::Population,
        });
    }
    if text.to_lowercase().starts_with("jaka jest jego populacja") {
        return Some(ParsedQuestion::ObjectLookup {
            subject: EntityPattern::Pronoun(PronounKind::His),
            predicate: PredicateId::Population,
        });
    }
    if text.to_lowercase().starts_with("jaka jest jej populacja") {
        return Some(ParsedQuestion::ObjectLookup {
            subject: EntityPattern::Pronoun(PronounKind::Her),
            predicate: PredicateId::Population,
        });
    }
    if let Some(subject) = tail_after_prefix(text, "gdzie jest ") {
        return Some(ParsedQuestion::ObjectLookup {
            subject: EntityPattern::from_text(subject),
            predicate: PredicateId::LocatedIn,
        });
    }
    if let Some(rest) = tail_after_prefix(text, "czy ") {
        if let Some((subject, object)) = split_case_insensitive(rest, " jest stolicą ") {
            return Some(ParsedQuestion::Boolean {
                subject: EntityPattern::from_text(subject),
                predicate: PredicateId::CapitalOf,
                object: EntityPattern::from_text(object),
            });
        }
    }
    if let Some(subject) = tail_after_prefix(text, "ilu mieszkańców ma ") {
        return Some(ParsedQuestion::ObjectLookup {
            subject: EntityPattern::from_text(subject),
            predicate: PredicateId::Population,
        });
    }
    for prefix in ["co to jest ", "czym jest ", "kim jest "] {
        if let Some(subject) = tail_after_prefix(text, prefix) {
            return Some(ParsedQuestion::ObjectLookup {
                subject: EntityPattern::from_text(subject),
                predicate: PredicateId::IsA,
            });
        }
    }
    None
}

fn split_case_insensitive<'a>(full: &'a str, phrase: &str) -> Option<(&'a str, &'a str)> {
    let lower_full = full.to_lowercase();
    let lower_phrase = phrase.to_lowercase();
    let split = lower_full.find(&lower_phrase)?;
    let subject = full[..split].trim();
    let object = full[split + phrase.len()..].trim();
    Some((subject, object))
}

fn tail_after_prefix<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let lower = text.to_lowercase();
    lower.strip_prefix(prefix).map(|_| text[prefix.len()..].trim())
}

impl EntityPattern {
    fn from_text(text: &str) -> Self {
        let normalized = normalize_entity_key(text);
        match normalized.as_str() {
            "it" => Self::Pronoun(PronounKind::It),
            "its" => Self::Pronoun(PronounKind::Its),
            "he" => Self::Pronoun(PronounKind::He),
            "she" => Self::Pronoun(PronounKind::She),
            "they" => Self::Pronoun(PronounKind::They),
            "his" => Self::Pronoun(PronounKind::His),
            "her" => Self::Pronoun(PronounKind::Her),
            "on" => Self::Pronoun(PronounKind::On),
            "ona" => Self::Pronoun(PronounKind::Ona),
            "ono" => Self::Pronoun(PronounKind::Ono),
            "to" => Self::Pronoun(PronounKind::To),
            _ => Self::Literal(text.trim().to_string()),
        }
    }
}

impl ParsedQuestion {
    pub fn resolve(
        &self,
        session: &crate::chat::conversation_qa::ConversationKnowledgeSession,
        language: &str,
    ) -> Result<ResolvedQuestion, String> {
        match self {
            Self::SubjectLookup { predicate, object } => Ok(ResolvedQuestion::SubjectLookup {
                predicate: predicate.clone(),
                object: resolve_entity_pattern(object, session, language)?,
            }),
            Self::ObjectLookup { subject, predicate } => Ok(ResolvedQuestion::ObjectLookup {
                subject: resolve_entity_pattern(subject, session, language)?,
                predicate: predicate.clone(),
            }),
            Self::Boolean {
                subject,
                predicate,
                object,
            } => Ok(ResolvedQuestion::Boolean {
                subject: resolve_entity_pattern(subject, session, language)?,
                predicate: predicate.clone(),
                object: resolve_entity_pattern(object, session, language)?,
            }),
        }
    }
}

fn resolve_entity_pattern(
    pattern: &EntityPattern,
    session: &crate::chat::conversation_qa::ConversationKnowledgeSession,
    language: &str,
) -> Result<EntityRef, String> {
    match pattern {
        EntityPattern::Literal(text) => Ok(resolve_literal_entity(text, session, language)),
        EntityPattern::Pronoun(pronoun) => resolve_pronoun(*pronoun, session, language),
    }
}

fn resolve_literal_entity(
    text: &str,
    session: &crate::chat::conversation_qa::ConversationKnowledgeSession,
    language: &str,
) -> EntityRef {
    let requested = normalize_entity_key(text);
    if let Some(state) = session.entities.get(&requested) {
        return state.entity.clone();
    }
    if is_polish(language) {
        let mut matches = session
            .entities
            .values()
            .filter(|state| polish_genitive_form(&state.entity.label) == requested)
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| left.entity.key.cmp(&right.entity.key));
        if matches.len() == 1 {
            return matches[0].entity.clone();
        }
    }
    EntityRef::new(text)
}

fn polish_genitive_form(label: &str) -> String {
    let key = normalize_entity_key(label);
    let Some((stem, last)) = key.rsplit_once(' ') else {
        return inflect_polish_genitive_word(&key);
    };
    format!("{stem} {}", inflect_polish_genitive_word(last))
}

fn inflect_polish_genitive_word(word: &str) -> String {
    if let Some(stem) = word.strip_suffix("cja") {
        format!("{stem}cji")
    } else if let Some(stem) = word.strip_suffix("ja") {
        format!("{stem}ji")
    } else if let Some(stem) = word.strip_suffix('a') {
        format!("{stem}y")
    } else {
        format!("{word}a")
    }
}

fn resolve_pronoun(
    pronoun: PronounKind,
    session: &crate::chat::conversation_qa::ConversationKnowledgeSession,
    _language: &str,
) -> Result<EntityRef, String> {
    let mut candidates = session
        .entities
            .values()
            .filter(|state| pronoun_matches(&state.entity, pronoun))
            .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .salience_score
            .cmp(&left.salience_score)
            .then(right.last_seen.cmp(&left.last_seen))
            .then(right.mention_count.cmp(&left.mention_count))
    });
    match candidates.as_slice() {
        [] => Err("no compatible antecedent in session".to_string()),
        [first, ..] => Ok(first.entity.clone()),
    }
}

fn pronoun_matches(entity: &EntityRef, pronoun: PronounKind) -> bool {
    match pronoun {
        PronounKind::They => matches!(entity.number, EntityNumber::Plural | EntityNumber::Unknown),
        PronounKind::He | PronounKind::His | PronounKind::On => {
            matches!(entity.gender, EntityGender::Masculine | EntityGender::Unknown)
                && !matches!(entity.number, EntityNumber::Plural)
        }
        PronounKind::She | PronounKind::Her | PronounKind::Ona => {
            matches!(entity.gender, EntityGender::Feminine | EntityGender::Unknown)
                && !matches!(entity.number, EntityNumber::Plural)
        }
        PronounKind::It | PronounKind::Its | PronounKind::Ono | PronounKind::To => {
            !matches!(entity.number, EntityNumber::Plural)
        }
    }
}

fn parse_capital_of_statement(
    sentence: &str,
    language: &str,
    entities: &BTreeMap<String, EntityState>,
) -> Option<(EntityRef, EntityRef)> {
    let lower = sentence.to_lowercase();
    if is_polish(language) {
        if let Some((subject, object)) = split_statement(sentence, &lower, " jest stolicą ") {
            return Some((resolve_entity_text(subject, entities)?, resolve_entity_text(object, entities)?));
        }
        if let Some((subject_text, description)) = sentence
            .split_once(" – ")
            .or_else(|| sentence.split_once(" - "))
        {
            let subject = subject_text
                .split(" (")
                .next()
                .unwrap_or("")
                .trim();
            let description = description.trim();
            if description.to_lowercase().starts_with("stolica") {
                let object = polish_capital_object(description)?;
                return Some((
                    resolve_entity_text(subject, entities)?,
                    resolve_entity_text(&object, entities)?,
                ));
            }
        }
    } else if let Some((subject, object)) = split_statement(sentence, &lower, " is the capital of ") {
        return Some((
            resolve_entity_text(subject, entities)?,
            resolve_entity_text(entity_clause(object), entities)?,
        ));
    } else if let Some(capital_start) = lower.find(" is the capital") {
        let subject = sentence[..capital_start].trim();
        let remainder = &sentence[capital_start + " is the capital".len()..];
        let remainder_lower = remainder.to_lowercase();
        let of_start = remainder_lower.find(" of ")?;
        let object = entity_clause(&remainder[of_start + 4..]);
        return Some((resolve_entity_text(subject, entities)?, resolve_entity_text(object, entities)?));
    }
    None
}

fn parse_population_statement(
    sentence: &str,
    language: &str,
    entities: &BTreeMap<String, EntityState>,
) -> Option<(EntityRef, i128)> {
    let lower = sentence.to_lowercase();
    if is_polish(language) {
        if let Some((subject, value)) = split_statement(sentence, &lower, " ma ") {
            if value.contains(" mieszkańców") {
                let number = extract_number(value)?;
                return Some((resolve_entity_text(subject, entities)?, number));
            }
        }
    } else {
        for phrase in [
            " has an estimated population of ",
            " had an estimated population of ",
            " has a population of ",
            " had a population of ",
        ] {
            if let Some((subject, value)) = split_statement(sentence, &lower, phrase) {
                let number = extract_number(value)?;
                return Some((resolve_entity_text(subject, entities)?, number));
            }
        }
        if let Some((subject, value)) = split_statement(sentence, &lower, " has ") {
            if value.contains(" inhabitants") || value.contains(" people") {
                let number = extract_number(value)?;
                return Some((resolve_entity_text(subject, entities)?, number));
            }
        }
    }
    None
}

fn parse_located_in_statement(
    sentence: &str,
    language: &str,
    entities: &BTreeMap<String, EntityState>,
) -> Option<(EntityRef, EntityRef)> {
    let lower = sentence.to_lowercase();
    if is_polish(language) {
        if let Some((subject, object)) = split_statement(sentence, &lower, " znajduje się w ") {
            return Some((resolve_entity_text(subject, entities)?, resolve_entity_text(object, entities)?));
        }
    } else if let Some((subject, object)) = split_statement(sentence, &lower, " is located in ") {
        return Some((resolve_entity_text(subject, entities)?, resolve_entity_text(object, entities)?));
    }
    None
}

fn parse_is_a_statement(
    sentence: &str,
    language: &str,
    entities: &BTreeMap<String, EntityState>,
) -> Option<(EntityRef, ConceptRef)> {
    let lower = sentence.to_lowercase();
    if is_polish(language) {
        if let Some((subject, object)) = split_statement(sentence, &lower, " jest ") {
            return Some((resolve_entity_text(subject, entities)?, concept_from_text(object, language)));
        }
    } else {
        for phrase in [" is a ", " is an ", " was a ", " was an "] {
            if let Some((subject, object)) = split_statement(sentence, &lower, phrase) {
                return Some((resolve_entity_text(subject, entities)?, concept_from_text(object, language)));
            }
        }
    }
    None
}

fn concept_from_text(text: &str, language: &str) -> ConceptRef {
    let label = normalize_entity_label(text);
    let normalized = normalize_entity_key(&label);
    let key = match normalized.as_str() {
        "capital" | "stolica" | "stolicą" => "capital",
        "city" | "miasto" | "miastem" => "city",
        "country" | "państwo" | "krajem" => "country",
        _ => normalized.as_str(),
    };
    let display = if is_polish(language) { label } else { label.to_lowercase() };
    ConceptRef::new(key, display)
}

fn sentence_mentions_city(sentence: &str, language: &str) -> bool {
    let tokens = sentence
        .to_lowercase()
        .split(|ch: char| !ch.is_alphabetic())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let marker = if is_polish(language) { "miasto" } else { "city" };
    tokens.iter().any(|token| token == marker)
}

fn entity_clause(text: &str) -> &str {
    text.split([',', ';'])
        .next()
        .unwrap_or(text)
        .trim()
        .trim_end_matches(|ch: char| matches!(ch, '.' | '!' | '?'))
}

fn polish_capital_object(description: &str) -> Option<String> {
    let lower = description.to_lowercase();
    let raw = if let Some(index) = lower.find(" miasto ") {
        &description[index + " miasto ".len()..]
    } else {
        description.get("stolica".len()..)?.trim()
    };
    let value = entity_clause(raw);
    let last_phrase = value
        .split(" a także ")
        .next()
        .unwrap_or(value)
        .trim();
    Some(normalize_polish_genitive(last_phrase))
}

fn normalize_polish_genitive(value: &str) -> String {
    let mut words = value.split_whitespace().map(str::to_string).collect::<Vec<_>>();
    if let Some(last) = words.last_mut() {
        let lower = last.to_lowercase();
        if lower.ends_with("ji") || lower.ends_with("ii") {
            last.truncate(last.len() - 1);
            last.push('a');
        }
    }
    words.join(" ")
}

fn split_statement<'a>(sentence: &'a str, lower: &str, phrase: &str) -> Option<(&'a str, &'a str)> {
    let split = lower.find(phrase)?;
    let subject = sentence[..split].trim();
    let object = sentence[split + phrase.len()..]
        .trim()
        .trim_end_matches(|ch: char| matches!(ch, '.' | '!' | '?' | ','));
    Some((subject, object))
}

fn resolve_entity_text(
    text: &str,
    entities: &BTreeMap<String, EntityState>,
) -> Option<EntityRef> {
    let candidate = EntityPattern::from_text(text);
    match candidate {
        EntityPattern::Literal(value) => Some(EntityRef::new(value)),
        EntityPattern::Pronoun(pronoun) => {
            let mut candidates = entities
                .values()
                .filter(|state| pronoun_matches(&state.entity, pronoun.clone()))
                .collect::<Vec<_>>();
            candidates.sort_by(|left, right| {
                right
                    .salience_score
                    .cmp(&left.salience_score)
                    .then(right.last_seen.cmp(&left.last_seen))
                    .then(right.mention_count.cmp(&left.mention_count))
            });
            candidates.first().map(|state| state.entity.clone())
        }
    }
}

fn extract_number(value: &str) -> Option<i128> {
    let mut digits = String::new();
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else if matches!(ch, ',' | ' ' | '_' | '\u{00A0}') {
            continue;
        } else if !digits.is_empty() {
            break;
        }
    }
    digits.parse().ok()
}
