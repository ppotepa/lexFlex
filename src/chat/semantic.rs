use crate::api::LexFlexAPI;
use crate::chat::trace::ChatTraceEntry;
use crate::core::interlingua::{
    Entity, Frame, Interlingua, QueryProjection, QueryTerm, QuestionSemantics, SemanticRole,
};
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SemanticFact {
    pub subject: Entity,
    pub predicate: String,
    pub object: SemanticFactObject,
    pub source_text: String,
    pub source_sentence: usize,
}

#[derive(Debug, Clone)]
pub enum SemanticFactObject {
    Entity(Entity),
    Concept { id: String, label: String },
}

#[derive(Debug, Clone)]
pub struct SemanticAnswer {
    pub text: String,
    pub evidence: Vec<String>,
}

#[derive(Clone)]
pub struct SemanticConversationMemory {
    api: Arc<LexFlexAPI>,
    facts: Vec<SemanticFact>,
}

impl fmt::Debug for SemanticConversationMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SemanticConversationMemory")
            .field("facts", &self.facts)
            .finish()
    }
}

impl SemanticConversationMemory {
    pub fn new(api: LexFlexAPI) -> Self {
        Self {
            api: Arc::new(api),
            facts: Vec::new(),
        }
    }

    pub fn observe(&mut self, text: &str, language: &str) -> Result<usize, String> {
        let Interlingua::Natural(utterance) = self
            .api
            .parse(text, language)
            .map_err(|error| error.to_string())?
        else {
            return Ok(0);
        };
        let mut added = 0;
        for (sentence_index, sentence) in utterance.sentences.iter().enumerate() {
            for frame in &sentence.frames {
                if let Some(fact) = fact_from_frame(frame, text, sentence_index) {
                    self.facts.push(fact);
                    added += 1;
                }
            }
        }
        Ok(added)
    }

    pub fn inspect(&self, text: &str, language: &str) -> Result<ChatTraceEntry, String> {
        let Interlingua::Natural(utterance) = self
            .api
            .parse(text, language)
            .map_err(|error| error.to_string())?
        else {
            return Ok(ChatTraceEntry {
                turn: 0,
                input: text.to_string(),
                language: language.to_string(),
                parse_status: "non_natural_interlingua".to_string(),
                sentence_count: 0,
                sentences: Vec::new(),
                question: None,
                facts_added: 0,
                answer_status: None,
                answer: None,
                evidence: Vec::new(),
                fallback: Some("non-natural Interlingua".to_string()),
            });
        };
        let question = utterance
            .sentences
            .iter()
            .find_map(|sentence| sentence.question.as_ref())
            .map(|question| {
                format!(
                    "kind={:?} predicate={} projection={:?} pattern={:?}",
                    question.kind,
                    question.proposition.predicate,
                    question.projection,
                    question.proposition
                )
            });
        let sentences = utterance
            .sentences
            .iter()
            .enumerate()
            .map(|(index, sentence)| {
                let frames = sentence.frames.iter().map(frame_name).collect::<Vec<_>>().join(",");
                format!("sentence={} illocution={:?} frames=[{}]", index + 1, sentence.illocution, frames)
            })
            .collect();
        Ok(ChatTraceEntry {
            turn: 0,
            input: text.to_string(),
            language: language.to_string(),
            parse_status: "ok".to_string(),
            sentence_count: utterance.sentences.len(),
            sentences,
            question,
            facts_added: 0,
            answer_status: None,
            answer: None,
            evidence: Vec::new(),
            fallback: None,
        })
    }

    pub fn answer(&self, text: &str, language: &str) -> Result<Option<SemanticAnswer>, String> {
        let Interlingua::Natural(utterance) = self
            .api
            .parse(text, language)
            .map_err(|error| error.to_string())?
        else {
            return Ok(None);
        };
        let Some(question) = utterance
            .sentences
            .iter()
            .find_map(|sentence| sentence.question.as_ref())
        else {
            return Ok(None);
        };
        Ok(answer_question(question, &self.facts, language))
    }

    pub fn clear(&mut self) {
        self.facts.clear();
    }

    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }
}

fn frame_name(frame: &Frame) -> String {
    match frame {
        Frame::Transfer { verb_concept, .. }
        | Frame::Motion { verb_concept, .. }
        | Frame::Creation { verb_concept, .. }
        | Frame::Destruction { verb_concept, .. }
        | Frame::Perception { verb_concept, .. }
        | Frame::Cognition { verb_concept, .. }
        | Frame::Emotion { verb_concept, .. }
        | Frame::Communication { verb_concept, .. }
        | Frame::Statement { verb_concept, .. }
        | Frame::Existence { verb_concept, .. }
        | Frame::Possession { verb_concept, .. }
        | Frame::Consumption { verb_concept, .. } => verb_concept.clone(),
        Frame::Custom { name, .. } => name.clone(),
    }
}

fn fact_from_frame(frame: &Frame, source_text: &str, sentence_index: usize) -> Option<SemanticFact> {
    match frame {
        Frame::Statement { subject, property, verb_concept } if verb_concept == "BE" => {
            Some(SemanticFact {
                subject: subject.clone(),
                predicate: "IS_A".to_string(),
                object: SemanticFactObject::Concept {
                    id: property.concept.0.clone(),
                    label: property.name.clone().unwrap_or_else(|| property.concept.0.to_lowercase()),
                },
                source_text: source_text.to_string(),
                source_sentence: sentence_index,
            })
        }
        Frame::Existence { entity, location: Some(location), .. }
            if is_classification_entity(location) => Some(SemanticFact {
                subject: entity.clone(),
                predicate: "IS_A".to_string(),
                object: SemanticFactObject::Concept {
                    id: location.concept.0.clone(),
                    label: location.name.clone().unwrap_or_else(|| location.concept.0.to_lowercase()),
                },
                source_text: source_text.to_string(),
                source_sentence: sentence_index,
            }),
        Frame::Custom { name, roles } if name == "CAPITAL_OF" => {
            let subject = roles
                .iter()
                .find(|(role, _)| matches!(role, SemanticRole::Topic))?
                .1
                .clone();
            let object = roles
                .iter()
                .find(|(role, _)| matches!(role, SemanticRole::Location))?
                .1
                .clone();
            Some(SemanticFact {
                subject,
                predicate: "CAPITAL_OF".to_string(),
                object: SemanticFactObject::Entity(object),
                source_text: source_text.to_string(),
                source_sentence: sentence_index,
            })
        }
        _ => None,
    }
}

fn is_classification_entity(entity: &Entity) -> bool {
    entity
        .name
        .as_deref()
        .map(|name| name.chars().next().map(char::is_lowercase).unwrap_or(false))
        .unwrap_or(false)
}

fn answer_question(
    question: &QuestionSemantics,
    facts: &[SemanticFact],
    language: &str,
) -> Option<SemanticAnswer> {
    let predicate = question.proposition.predicate.0.as_str();
    let (matches, values, subject_label) = match (
        question.proposition.subject.as_ref(),
        question.proposition.object.as_ref(),
        question.projection,
    ) {
        (Some(QueryTerm::Entity(subject)), _, QueryProjection::Object | QueryProjection::Value | QueryProjection::Concept) => {
            let requested = subject.name.as_deref()?.to_lowercase();
            let matches = facts
                .iter()
                .filter(|fact| {
                    fact.predicate == predicate
                        && fact.subject.name.as_deref().map(str::to_lowercase).as_deref()
                            == Some(requested.as_str())
                })
                .collect::<Vec<_>>();
            let values = matches.iter().map(fact_object_label).collect::<Vec<_>>();
            (matches, values, subject.name.clone().unwrap_or_default())
        }
        (Some(QueryTerm::Variable(_)), Some(QueryTerm::Entity(object)), QueryProjection::Subject) => {
            let requested = object.name.as_deref()?.to_lowercase();
            let matches = facts
                .iter()
                .filter(|fact| {
                    fact.predicate == predicate
                        && matches!(&fact.object, SemanticFactObject::Entity(entity)
                            if entity.name.as_deref().map(str::to_lowercase).as_deref()
                                == Some(requested.as_str()))
                })
                .collect::<Vec<_>>();
            let values = matches
                .iter()
                .filter_map(|fact| fact.subject.name.clone())
                .collect::<Vec<_>>();
            (matches, values, object.name.clone().unwrap_or_default())
        }
        _ => return None,
    };
    if matches.is_empty() {
        return None;
    }
    let answer = values.join(", ");
    let text = if question.projection == QueryProjection::Subject {
        answer
    } else if language.eq_ignore_ascii_case("pl") {
        format!("{} jest {}.", subject_label, answer)
    } else {
        format!("{} is {}.", subject_label, answer)
    };
    Some(SemanticAnswer {
        text,
        evidence: matches
            .iter()
            .map(|fact| format!("sentence {}: {}", fact.source_sentence + 1, fact.source_text))
            .collect(),
    })
}

fn fact_object_label(fact: &&SemanticFact) -> String {
    match &fact.object {
        SemanticFactObject::Entity(entity) => entity
            .name
            .clone()
            .unwrap_or_else(|| entity.concept.0.clone()),
        SemanticFactObject::Concept { label, .. } => label.clone(),
    }
}
