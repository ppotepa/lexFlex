mod language;
mod rules;

pub use language::detect_language;
pub use rules::{looks_like_question, question_lookup_title};

use crate::document::span::SourceSpan;
use crate::query::AnswerStatus;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConversationSourceId(pub usize);

impl fmt::Display for ConversationSourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FactId(pub usize);

impl fmt::Display for FactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationSourceKind {
    Conversation,
    Wikipedia,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityNumber {
    Singular,
    Plural,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityGender {
    Masculine,
    Feminine,
    Neutral,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRef {
    pub key: String,
    pub label: String,
    pub number: EntityNumber,
    pub gender: EntityGender,
}

impl EntityRef {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        let key = rules::normalize_entity_key(&label);
        let number = rules::guess_entity_number(&label);
        let gender = rules::guess_entity_gender(&label);
        Self {
            key,
            label: rules::normalize_entity_label(&label),
            number,
            gender,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptRef {
    pub key: String,
    pub label: String,
}

impl ConceptRef {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        let key = key.into();
        let label = label.into();
        Self {
            key: rules::normalize_text_key(&key),
            label: rules::normalize_entity_label(&label),
        }
    }

    pub fn render_short(&self, language: &str) -> String {
        match (self.key.as_str(), language::is_polish(language)) {
            ("capital", true) => "stolica".to_string(),
            ("capital", false) => "capital".to_string(),
            ("city", true) => "miasto".to_string(),
            ("city", false) => "city".to_string(),
            ("country", true) => "państwo".to_string(),
            ("country", false) => "country".to_string(),
            _ => self.label.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredicateId {
    IsA,
    CapitalOf,
    Population,
    LocatedIn,
    BornIn,
    WorksFor,
    CreatedBy,
    DateOf,
    HasProperty,
    HasValue,
    Unknown(String),
}

impl PredicateId {
    pub fn is_functional(&self) -> bool {
        matches!(self, Self::CapitalOf | Self::Population | Self::DateOf)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactObject {
    Entity(EntityRef),
    Concept(ConceptRef),
    Integer(i128),
    Decimal(crate::document::knowledge::DecimalValue),
    Text(String),
    Date(String),
    Boolean(bool),
}

impl FactObject {
    pub fn canonical_key(&self) -> String {
        match self {
            Self::Entity(entity) => format!("entity:{}", entity.key),
            Self::Concept(concept) => format!("concept:{}", concept.key),
            Self::Integer(value) => format!("int:{value}"),
            Self::Decimal(value) => format!(
                "decimal:{}:{}:{}",
                value.sign, value.mantissa, value.scale
            ),
            Self::Text(value) => format!("text:{}", rules::normalize_text_key(value)),
            Self::Date(value) => format!("date:{}", rules::normalize_text_key(value)),
            Self::Boolean(value) => format!("bool:{value}"),
        }
    }

    pub fn render_short(&self, language: &str) -> String {
        match self {
            Self::Entity(entity) => entity.label.clone(),
            Self::Concept(concept) => concept.render_short(language),
            Self::Integer(value) => {
                let formatted = format_integer(*value, if language::is_polish(language) { ' ' } else { ',' });
                if language::is_polish(language) {
                    format!("{formatted} osób")
                } else {
                    format!("{formatted} people")
                }
            }
            Self::Decimal(value) => rules::render_decimal(value),
            Self::Text(value) => value.clone(),
            Self::Date(value) => value.clone(),
            Self::Boolean(value) => {
                if *value {
                    if language::is_polish(language) {
                        "tak".to_string()
                    } else {
                        "yes".to_string()
                    }
                } else if language::is_polish(language) {
                    "nie".to_string()
                } else {
                    "no".to_string()
                }
            }
        }
    }
}

fn format_integer(value: i128, separator: char) -> String {
    let mut digits = value.abs().to_string();
    let mut groups = Vec::new();
    while digits.len() > 3 {
        let tail = digits.split_off(digits.len() - 3);
        groups.push(tail);
    }
    groups.push(digits);
    let mut formatted = groups
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(&separator.to_string());
    if value < 0 {
        formatted.insert(0, '-');
    }
    formatted
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationSource {
    pub id: ConversationSourceId,
    pub kind: ConversationSourceKind,
    pub label: String,
    pub title: Option<String>,
    pub message_id: Option<usize>,
    pub url: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationEvidence {
    pub source_id: ConversationSourceId,
    pub source_label: String,
    pub source_kind: ConversationSourceKind,
    pub message_id: Option<usize>,
    pub source_url: Option<String>,
    pub sentence_index: usize,
    pub span: SourceSpan,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionFact {
    pub id: FactId,
    pub subject: EntityRef,
    pub predicate: PredicateId,
    pub object: FactObject,
    pub evidence: Vec<ConversationEvidence>,
    pub confidence_milli: u16,
}

impl SessionFact {
    pub fn signature(&self) -> String {
        format!(
            "{}|{:?}|{}",
            self.subject.key,
            self.predicate,
            self.object.canonical_key()
        )
    }

    pub fn render(&self, language: &str) -> String {
        rules::render_fact(self, language)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationIngestReport {
    pub source_id: ConversationSourceId,
    pub source_label: String,
    pub fact_ids: Vec<FactId>,
    pub fact_summaries: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationAnswer {
    pub status: AnswerStatus,
    pub text: String,
    pub fact_ids: Vec<FactId>,
    pub evidence: Vec<ConversationEvidence>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationOutcome {
    Answered(ConversationAnswer),
    Ingested(ConversationIngestReport),
    Unsupported(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversationKnowledgeSession {
    sources: BTreeMap<ConversationSourceId, ConversationSource>,
    source_order: Vec<ConversationSourceId>,
    facts: BTreeMap<FactId, SessionFact>,
    fact_order: Vec<FactId>,
    facts_by_signature: BTreeMap<String, FactId>,
    entities: BTreeMap<String, rules::EntityState>,
    next_source_id: usize,
    next_fact_id: usize,
    next_sequence: usize,
}

impl ConversationKnowledgeSession {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn source_count(&self) -> usize {
        self.source_order.len()
    }

    pub fn fact_count(&self) -> usize {
        self.fact_order.len()
    }

    pub fn ingest_conversation_message(
        &mut self,
        message_id: usize,
        text: &str,
        language: &str,
    ) -> ConversationOutcome {
        let source = self.register_source(
            ConversationSourceKind::Conversation,
            format!("message-{message_id}"),
            None,
            Some(message_id),
            None,
            text,
        );
        self.ingest_source_text(source, language)
    }

    pub fn ingest_wikipedia_article(
        &mut self,
        title: &str,
        text: &str,
        language: &str,
    ) -> ConversationIngestReport {
        self.ingest_wikipedia_article_with_url(title, text, language, None)
    }

    pub fn ingest_wikipedia_article_with_url(
        &mut self,
        title: &str,
        text: &str,
        language: &str,
        url: Option<&str>,
    ) -> ConversationIngestReport {
        let source = self.register_source(
            ConversationSourceKind::Wikipedia,
            format!("wikipedia:{title}"),
            Some(title.to_string()),
            None,
            url.map(str::to_string),
            text,
        );
        let source_id = source.id;
        let source_label = title.to_string();
        match self.ingest_source_text(source, language) {
            ConversationOutcome::Ingested(report) => report,
            ConversationOutcome::Answered(answer) => ConversationIngestReport {
                source_id,
                source_label,
                fact_ids: answer.fact_ids,
                fact_summaries: vec![answer.text],
                notes: answer.notes,
            },
            ConversationOutcome::Unsupported(note) => ConversationIngestReport {
                source_id,
                source_label,
                fact_ids: vec![],
                fact_summaries: vec![],
                notes: vec![note],
            },
        }
    }

    pub fn answer_question(&self, question: &str, language: &str) -> ConversationAnswer {
        match rules::parse_question(question, language) {
            Some(parsed) => self.answer_parsed_question(parsed, language),
            None => ConversationAnswer {
                status: AnswerStatus::Unsupported,
                text: if language::is_polish(language) {
                    "Nie potrafię jeszcze odpowiedzieć na to pytanie.".to_string()
                } else {
                    "I cannot parse that question yet.".to_string()
                },
                fact_ids: vec![],
                evidence: vec![],
                notes: vec![],
            },
        }
    }

    pub fn observe_text(
        &mut self,
        message_id: usize,
        text: &str,
        language: &str,
    ) -> ConversationOutcome {
        if rules::looks_like_question(text, language) {
            return ConversationOutcome::Answered(self.answer_question(text, language));
        }
        self.ingest_conversation_message(message_id, text, language)
    }

    fn ingest_source_text(
        &mut self,
        source: ConversationSource,
        language: &str,
    ) -> ConversationOutcome {
        let source_id = source.id;
        let source_label = source.label.clone();
        let sentences = rules::split_sentences_with_spans(&source.text);
        let mut fact_ids = Vec::new();
        let mut summaries = Vec::new();
        let mut notes = Vec::new();

        for (sentence_index, sentence) in sentences.iter().enumerate() {
            let drafts = rules::extract_facts_from_sentence(
                &sentence.text,
                source_id,
                &source.label,
                source.message_id,
                source.url.as_deref(),
                sentence_index,
                sentence.span,
                language,
                &self.entities,
            );
            if drafts.is_empty() {
                continue;
            }
            for draft in drafts {
                self.next_sequence += 1;
                self.update_entities(&draft.subject, true);
                if let FactObject::Entity(entity) = &draft.object {
                    self.update_entities(entity, false);
                }
                let signature = draft.signature();
                let fact_id = if let Some(existing) = self.facts_by_signature.get(&signature).copied() {
                    if let Some(fact) = self.facts.get_mut(&existing) {
                        fact.evidence.extend(draft.evidence);
                        existing
                    } else {
                        self.insert_fact(draft, signature)
                    }
                } else {
                    self.insert_fact(draft, signature)
                };
                if !fact_ids.contains(&fact_id) {
                    fact_ids.push(fact_id);
                }
                if let Some(fact) = self.facts.get(&fact_id) {
                    summaries.push(fact.render(language));
                }
            }
        }

        self.sources.insert(source_id, source);
        self.source_order.push(source_id);

        if fact_ids.is_empty() {
            notes.push(if language::is_polish(language) {
                "Nie udało się wydobyć prostych faktów z tego tekstu.".to_string()
            } else {
                "No simple facts were extracted from this text.".to_string()
            });
        }

        ConversationOutcome::Ingested(ConversationIngestReport {
            source_id,
            source_label,
            fact_ids,
            fact_summaries: summaries,
            notes,
        })
    }

    fn register_source(
        &mut self,
        kind: ConversationSourceKind,
        label: String,
        title: Option<String>,
        message_id: Option<usize>,
        url: Option<String>,
        text: &str,
    ) -> ConversationSource {
        let source_id = ConversationSourceId(self.next_source_id + 1);
        self.next_source_id += 1;
        ConversationSource {
            id: source_id,
            kind,
            label,
            title,
            message_id,
            url,
            text: text.to_string(),
        }
    }

    fn insert_fact(&mut self, draft: SessionFact, signature: String) -> FactId {
        let fact_id = FactId(self.next_fact_id + 1);
        self.next_fact_id += 1;
        self.facts_by_signature.insert(signature, fact_id);
        self.fact_order.push(fact_id);
        self.facts.insert(
            fact_id,
            SessionFact {
                id: fact_id,
                ..draft
            },
        );
        fact_id
    }

    fn update_entities(&mut self, entity: &EntityRef, is_subject: bool) {
        let next = self.next_sequence;
        self.entities
            .entry(entity.key.clone())
            .and_modify(|state| {
                state.entity = entity.clone();
                state.mention_count += 1;
                state.last_seen = next;
                state.salience_score += if is_subject { 2 } else { 1 };
            })
            .or_insert_with(|| rules::EntityState {
                entity: entity.clone(),
                mention_count: 1,
                last_seen: next,
                salience_score: if is_subject { 2 } else { 1 },
            });
    }

    fn answer_parsed_question(
        &self,
        question: rules::ParsedQuestion,
        language: &str,
    ) -> ConversationAnswer {
        let resolved = match question.resolve(self, language) {
            Ok(value) => value,
            Err(reason) => {
                return ConversationAnswer {
                    status: AnswerStatus::Unsupported,
                    text: if language::is_polish(language) {
                        format!("Nie potrafię odpowiedzieć: {reason}")
                    } else {
                        format!("I cannot answer that: {reason}")
                    },
                    fact_ids: vec![],
                    evidence: vec![],
                    notes: vec![],
                };
            }
        };

        let matches = self.match_question(&resolved);
        if matches.is_empty() {
            if let Some(answer) = self.type_mismatch_answer(&resolved, language) {
                return answer;
            }
            return ConversationAnswer {
                status: AnswerStatus::Unknown,
                text: if language::is_polish(language) {
                    "Nie mam tej informacji w tej rozmowie.".to_string()
                } else {
                    "I do not have that information in this conversation.".to_string()
                },
                fact_ids: vec![],
                evidence: vec![],
                notes: vec![],
            };
        }

        let mut evidence = vec![];
        let mut fact_ids = vec![];
        let mut rendered = vec![];
        for fact in matches.iter().map(|id| self.facts.get(id).expect("fact exists")) {
            fact_ids.push(fact.id);
            evidence.extend(fact.evidence.clone());
            rendered.push(fact.render(language));
        }
        evidence.sort_by(|left, right| {
            left.source_id
                .cmp(&right.source_id)
                .then(left.sentence_index.cmp(&right.sentence_index))
                .then(left.span.start.cmp(&right.span.start))
        });
        evidence.dedup_by(|left, right| {
            left.source_id == right.source_id
                && left.sentence_index == right.sentence_index
                && left.span == right.span
                && left.text == right.text
        });

        let status = if matches.len() == 1 {
            AnswerStatus::Exact
        } else if self.query_is_conflicting(&resolved, &matches) {
            AnswerStatus::Conflicting
        } else {
            AnswerStatus::Supported
        };

        let text = if matches!(
            &resolved,
            rules::ResolvedQuestion::ObjectLookup {
                predicate: PredicateId::IsA,
                ..
            }
        ) {
            render_definition_answer(&resolved, &matches, &self.facts, language)
        } else {
            render_question_answer(
                &resolved,
                self.facts.get(matches.first().expect("nonempty matches")).expect("fact exists"),
                status,
                language,
            )
        };

        let notes = evidence
            .iter()
            .map(|item| rules::format_evidence_note(item))
            .collect();

        ConversationAnswer {
            status,
            text,
            fact_ids,
            evidence,
            notes,
        }
    }

    fn match_question(&self, question: &rules::ResolvedQuestion) -> Vec<FactId> {
        let mut out = vec![];
        for fact_id in &self.fact_order {
            let Some(fact) = self.facts.get(fact_id) else {
                continue;
            };
            if question.matches(fact) {
                out.push(*fact_id);
            }
        }
        out
    }

    fn query_is_conflicting(
        &self,
        question: &rules::ResolvedQuestion,
        matches: &[FactId],
    ) -> bool {
        let Some(first_id) = matches.first() else {
            return false;
        };
        let Some(first_fact) = self.facts.get(first_id) else {
            return false;
        };
        if !first_fact.predicate.is_functional() {
            return false;
        }
        let canonical = match question {
            rules::ResolvedQuestion::SubjectLookup { .. } => first_fact.subject.key.clone(),
            rules::ResolvedQuestion::ObjectLookup { .. } => first_fact.object.canonical_key(),
            rules::ResolvedQuestion::Boolean { .. } => first_fact.object.canonical_key(),
        };
        for fact_id in matches.iter().skip(1) {
            if let Some(fact) = self.facts.get(fact_id) {
                let other = match question {
                    rules::ResolvedQuestion::SubjectLookup { .. } => fact.subject.key.clone(),
                    rules::ResolvedQuestion::ObjectLookup { .. } => fact.object.canonical_key(),
                    rules::ResolvedQuestion::Boolean { .. } => fact.object.canonical_key(),
                };
                if other != canonical {
                    return true;
                }
            }
        }
        false
    }

    fn type_mismatch_answer(
        &self,
        question: &rules::ResolvedQuestion,
        language: &str,
    ) -> Option<ConversationAnswer> {
        let rules::ResolvedQuestion::SubjectLookup {
            predicate: PredicateId::CapitalOf,
            object,
        } = question
        else {
            return None;
        };
        let type_fact = self.fact_order.iter().find_map(|id| {
            let fact = self.facts.get(id)?;
            match (&fact.predicate, &fact.object) {
                (PredicateId::IsA, FactObject::Concept(concept))
                    if fact.subject.key == object.key
                        && matches!(concept.key.as_str(), "capital" | "city") => Some(fact),
                _ => None,
            }
        })?;
        let evidence = type_fact.evidence.clone();
        Some(ConversationAnswer {
            status: AnswerStatus::InvalidQuery,
            text: if language::is_polish(language) {
                format!(
                    "{} jest rozpoznany jako {}; nie mam podstaw, by traktować go jako obszar posiadający stolicę.",
                    object.label,
                    type_fact.object.render_short(language)
                )
            } else {
                format!(
                    "{} is identified as a {}; I have no evidence that it is a jurisdiction with a capital.",
                    object.label,
                    type_fact.object.render_short(language)
                )
            },
            fact_ids: vec![type_fact.id],
            notes: evidence.iter().map(rules::format_evidence_note).collect(),
            evidence,
        })
    }
}

fn render_definition_answer(
    question: &rules::ResolvedQuestion,
    matches: &[FactId],
    facts: &BTreeMap<FactId, SessionFact>,
    language: &str,
) -> String {
    let rules::ResolvedQuestion::ObjectLookup { subject, .. } = question else {
        return String::new();
    };
    let mut concepts = matches
        .iter()
        .filter_map(|id| match &facts.get(id)?.object {
            FactObject::Concept(concept) => Some(concept.render_short(language)),
            _ => None,
        })
        .collect::<Vec<_>>();
    concepts.sort();
    concepts.dedup();
    let joined = if language::is_polish(language) {
        concepts.join(" i ")
    } else {
        concepts.join(" and ")
    };
    if language::is_polish(language) {
        format!("{} to {}.", subject.label, joined)
    } else {
        format!("{} is a {}.", subject.label, joined)
    }
}

fn render_question_answer(
    question: &rules::ResolvedQuestion,
    fact: &SessionFact,
    status: AnswerStatus,
    language: &str,
) -> String {
    if matches!(status, AnswerStatus::Conflicting) {
        return if language::is_polish(language) {
            "Znalazłem sprzeczne fakty w rozmowie lub źródłach.".to_string()
        } else {
            "I found conflicting facts in the conversation or sources.".to_string()
        };
    }
    match question {
        rules::ResolvedQuestion::SubjectLookup { .. } => {
            format!("{}.", fact.subject.label)
        }
        rules::ResolvedQuestion::ObjectLookup { predicate, .. } => match predicate {
            PredicateId::Population => format!("{}.", fact.object.render_short(language)),
            PredicateId::LocatedIn | PredicateId::IsA | PredicateId::CapitalOf => {
                format!("{}.", fact.object.render_short(language))
            }
            _ => format!("{}.", fact.object.render_short(language)),
        },
        rules::ResolvedQuestion::Boolean { .. } => {
            if language::is_polish(language) {
                "Tak.".to_string()
            } else {
                "Yes.".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ConversationKnowledgeSession, ConversationOutcome};
    use crate::query::AnswerStatus;

    #[test]
    fn conversation_facts_answer_simple_questions() {
        let mut session = ConversationKnowledgeSession::default();
        let ingest = session.observe_text(
            1,
            "Paris is the capital of France. It has a population of 2,100,000 people.",
            "en",
        );
        match ingest {
            ConversationOutcome::Ingested(report) => {
                assert_eq!(report.fact_ids.len(), 3);
            }
            other => panic!("unexpected ingest outcome: {other:?}"),
        }

        let answer = session.answer_question("What is the capital of France?", "en");
        assert_eq!(answer.status, AnswerStatus::Exact);
        assert_eq!(answer.text, "Paris.");

        let answer = session.answer_question("What is its population?", "en");
        assert_eq!(answer.status, AnswerStatus::Exact);
        assert!(answer.text.contains("2,100,000"));
    }

    #[test]
    fn wikipedia_facts_merge_with_conversation_memory() {
        let mut session = ConversationKnowledgeSession::default();
        let report = session.ingest_wikipedia_article_with_url(
            "Paris",
            "Paris is the capital of France.",
            "en",
            Some("https://en.wikipedia.org/wiki/Paris"),
        );
        assert_eq!(report.fact_ids.len(), 2);
        let answer = session.answer_question("Is Paris the capital of France?", "en");
        assert_eq!(answer.status, AnswerStatus::Exact);
        assert_eq!(answer.text, "Yes.");
        assert_eq!(answer.evidence[0].source_url.as_deref(), Some("https://en.wikipedia.org/wiki/Paris"));
    }


    #[test]
    fn capital_relation_deduces_a_queryable_type() {
        let mut session = ConversationKnowledgeSession::default();
        session.ingest_wikipedia_article(
            "Paris",
            "Paris is the capital and largest city of France.",
            "en",
        );

        let answer = session.answer_question("What is Paris?", "en");
        assert_eq!(answer.status, AnswerStatus::Supported);
        assert!(answer.text.contains("capital"));
        assert!(answer.text.contains("city"));
        assert!(!answer.evidence.is_empty());
    }

    #[test]
    fn polish_definition_and_type_mismatch_use_deduced_types() {
        let mut session = ConversationKnowledgeSession::default();
        session.ingest_wikipedia_article(
            "Paryż",
            "Paryż (fr. Paris) – stolica i najludniejsze miasto Francji.",
            "pl",
        );

        let definition = session.answer_question("Co to jest Paryż?", "pl");
        assert_eq!(definition.status, AnswerStatus::Supported);
        assert!(definition.text.contains("stolica"));
        assert!(definition.text.contains("miasto"));

        let invalid = session.answer_question("Jaka jest stolica Paryża?", "pl");
        assert_eq!(invalid.status, AnswerStatus::InvalidQuery);
        assert!(invalid.text.contains("obszar posiadający stolicę"));
        assert!(!invalid.evidence.is_empty());
    }

    #[test]
    fn conflicting_sources_surface_conflicts() {
        let mut session = ConversationKnowledgeSession::default();
        session.ingest_wikipedia_article("Paris", "Lyon is the capital of France.", "en");
        session.observe_text(1, "Paris is the capital of France.", "en");
        let answer = session.answer_question("What is the capital of France?", "en");
        assert_eq!(answer.status, AnswerStatus::Conflicting);
        assert!(answer.text.to_lowercase().contains("conflicting"));
    }
}
