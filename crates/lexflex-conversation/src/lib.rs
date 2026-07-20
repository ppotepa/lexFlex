#![forbid(unsafe_code)]

use lexflex_model::{EntityId, SemanticExpression};
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use std::io::Read;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const MAX_PERSISTED_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Turn {
    pub turn_id: String,
    pub expression: SemanticExpression,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityMention {
    pub entity: EntityId,
    pub turn_id: String,
    pub salience: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceCandidate {
    pub entity: EntityId,
    pub turn_id: String,
    pub score: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceResolution {
    Resolved(ReferenceCandidate),
    Ambiguous(Vec<ReferenceCandidate>),
    Unresolved,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ConversationState {
    turns: Vec<Turn>,
    mentions: Vec<EntityMention>,
}

impl<'de> Deserialize<'de> for ConversationState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            turns: Vec<Turn>,
            mentions: Vec<EntityMention>,
        }

        let wire = Wire::deserialize(deserializer)?;
        let state = Self {
            turns: wire.turns,
            mentions: wire.mentions,
        };
        state.verify().map_err(D::Error::custom)?;
        Ok(state)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConversationError {
    #[error("turn id is empty")]
    EmptyTurnId,
    #[error("turn id already exists: {0}")]
    DuplicateTurn(String),
    #[error("mention references an unknown turn: {0}")]
    UnknownTurn(String),
    #[error("entity mention id is empty")]
    EmptyEntityId,
    #[error("turn expression is not canonicalizable")]
    InvalidExpression,
    #[error("conversation persistence IO error: {0}")]
    PersistenceIo(String),
    #[error("conversation persistence serialization error: {0}")]
    PersistenceSerde(String),
    #[error("conversation payload exceeds resource limit")]
    PersistenceTooLarge,
}

impl ConversationState {
    fn collect_entities(expression: &SemanticExpression, output: &mut Vec<EntityId>) {
        match expression {
            SemanticExpression::Entity(entity) => output.push(entity.clone()),
            SemanticExpression::Apply { bindings, .. } => {
                for expression in bindings.values() {
                    Self::collect_entities(expression, output);
                }
            }
            SemanticExpression::Satisfies { subject, predicate }
            | SemanticExpression::Equals {
                left: subject,
                right: predicate,
            } => {
                Self::collect_entities(subject, output);
                Self::collect_entities(predicate, output);
            }
            SemanticExpression::And(expressions) | SemanticExpression::Or(expressions) => {
                for expression in expressions {
                    Self::collect_entities(expression, output);
                }
            }
            SemanticExpression::Not(expression)
            | SemanticExpression::Exists {
                body: expression, ..
            }
            | SemanticExpression::ForAll {
                body: expression, ..
            } => Self::collect_entities(expression, output),
            SemanticExpression::Qualified {
                expression,
                qualifiers,
            } => {
                Self::collect_entities(expression, output);
                for expression in qualifiers.values() {
                    Self::collect_entities(expression, output);
                }
            }
            SemanticExpression::Concept(_)
            | SemanticExpression::Value(_)
            | SemanticExpression::Variable(_) => {}
        }
    }

    fn validate_turn(turn: &Turn) -> Result<(), ConversationError> {
        if turn.turn_id.trim().is_empty() {
            return Err(ConversationError::EmptyTurnId);
        }
        let normalized = turn
            .expression
            .normalized()
            .map_err(|_| ConversationError::InvalidExpression)?;
        if normalized != turn.expression {
            return Err(ConversationError::InvalidExpression);
        }
        Ok(())
    }

    pub fn load_from_file(path: &std::path::Path) -> Result<Self, ConversationError> {
        let file = std::fs::File::open(path)
            .map_err(|error| ConversationError::PersistenceIo(error.to_string()))?;
        let mut bytes = Vec::new();
        file.take((MAX_PERSISTED_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| ConversationError::PersistenceIo(error.to_string()))?;
        if bytes.len() > MAX_PERSISTED_BYTES {
            return Err(ConversationError::PersistenceTooLarge);
        }
        serde_json::from_slice(&bytes)
            .map_err(|error| ConversationError::PersistenceSerde(error.to_string()))
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), ConversationError> {
        self.verify()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| ConversationError::PersistenceIo(error.to_string()))?;
        }
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| ConversationError::PersistenceSerde(error.to_string()))?;
        let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = path.with_file_name(format!(
            "{}.{}.{}.tmp",
            path.file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("state"),
            std::process::id(),
            sequence
        ));
        let mut temporary_file = std::fs::File::create(&temporary)
            .map_err(|error| ConversationError::PersistenceIo(error.to_string()))?;
        use std::io::Write;
        temporary_file
            .write_all(&bytes)
            .and_then(|_| temporary_file.sync_all())
            .map_err(|error| ConversationError::PersistenceIo(error.to_string()))?;
        std::fs::rename(&temporary, path).map_err(|error| {
            let _ = std::fs::remove_file(&temporary);
            ConversationError::PersistenceIo(error.to_string())
        })
    }

    pub fn append_turn(&mut self, turn: Turn) -> Result<(), ConversationError> {
        if turn.turn_id.trim().is_empty() {
            return Err(ConversationError::EmptyTurnId);
        }
        if self.turns.iter().any(|item| item.turn_id == turn.turn_id) {
            return Err(ConversationError::DuplicateTurn(turn.turn_id));
        }
        let normalized = turn
            .expression
            .normalized()
            .map_err(|_| ConversationError::InvalidExpression)?;
        let mut candidate = self.clone();
        let mut entities = Vec::new();
        Self::collect_entities(&normalized, &mut entities);
        let turn_id = turn.turn_id.clone();
        candidate.turns.push(Turn {
            turn_id: turn.turn_id,
            expression: normalized,
        });
        let salience = candidate
            .mentions
            .iter()
            .map(|mention| mention.salience)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        for entity in entities {
            candidate.mentions.push(EntityMention {
                entity,
                turn_id: turn_id.clone(),
                salience,
            });
        }
        candidate.verify()?;
        *self = candidate;
        Ok(())
    }

    pub fn add_mention(&mut self, mention: EntityMention) -> Result<(), ConversationError> {
        if mention.entity.as_str().trim().is_empty() {
            return Err(ConversationError::EmptyEntityId);
        }
        if !self
            .turns
            .iter()
            .any(|turn| turn.turn_id == mention.turn_id)
        {
            return Err(ConversationError::UnknownTurn(mention.turn_id));
        }
        let mut candidate = self.clone();
        candidate.mentions.push(mention);
        candidate.verify()?;
        *self = candidate;
        Ok(())
    }

    pub fn turns(&self) -> &[Turn] {
        &self.turns
    }
    pub fn mentions(&self) -> &[EntityMention] {
        &self.mentions
    }

    pub fn resolve_entity(&self, entity_type: &str) -> ReferenceResolution {
        let mut candidates: Vec<_> = self
            .mentions
            .iter()
            .filter(|mention| mention.entity.as_str().starts_with(entity_type))
            .map(|mention| ReferenceCandidate {
                entity: mention.entity.clone(),
                turn_id: mention.turn_id.clone(),
                score: mention.salience,
            })
            .collect();
        candidates.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.turn_id.cmp(&right.turn_id))
                .then_with(|| left.entity.cmp(&right.entity))
        });
        match candidates.len() {
            0 => ReferenceResolution::Unresolved,
            1 => ReferenceResolution::Resolved(candidates.remove(0)),
            _ if candidates[0].score > candidates[1].score => {
                ReferenceResolution::Resolved(candidates.remove(0))
            }
            _ => ReferenceResolution::Ambiguous(candidates),
        }
    }

    pub fn verify(&self) -> Result<(), ConversationError> {
        let mut turn_ids = std::collections::BTreeSet::new();
        for turn in &self.turns {
            Self::validate_turn(turn)?;
            if !turn_ids.insert(turn.turn_id.clone()) {
                return Err(ConversationError::DuplicateTurn(turn.turn_id.clone()));
            }
        }
        for mention in &self.mentions {
            if mention.entity.as_str().trim().is_empty() {
                return Err(ConversationError::EmptyEntityId);
            }
            if !turn_ids.contains(&mention.turn_id) {
                return Err(ConversationError::UnknownTurn(mention.turn_id.clone()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(id: &str) -> Turn {
        Turn {
            turn_id: id.into(),
            expression: SemanticExpression::Value(true.into()),
        }
    }

    #[test]
    fn equal_reference_scores_remain_ambiguous() {
        let mut state = ConversationState::default();
        state.append_turn(turn("t1")).unwrap();
        state.append_turn(turn("t2")).unwrap();
        state
            .add_mention(EntityMention {
                entity: EntityId::new_unchecked("entity:paris"),
                turn_id: "t1".into(),
                salience: 4,
            })
            .unwrap();
        state
            .add_mention(EntityMention {
                entity: EntityId::new_unchecked("entity:london"),
                turn_id: "t2".into(),
                salience: 4,
            })
            .unwrap();
        assert!(matches!(
            state.resolve_entity("entity:"),
            ReferenceResolution::Ambiguous(_)
        ));
    }

    #[test]
    fn appending_semantic_turn_indexes_entity_mentions() {
        let mut state = ConversationState::default();
        state
            .append_turn(Turn {
                turn_id: "t1".into(),
                expression: SemanticExpression::And(vec![
                    SemanticExpression::Entity(EntityId::new_unchecked("entity:paris")),
                    SemanticExpression::Entity(EntityId::new_unchecked("entity:france")),
                ]),
            })
            .unwrap();
        assert_eq!(state.mentions().len(), 2);
        assert!(matches!(
            state.resolve_entity("entity:paris"),
            ReferenceResolution::Resolved(_)
        ));
    }

    #[test]
    fn newer_turn_has_higher_reference_salience() {
        let mut state = ConversationState::default();
        for (turn_id, entity) in [("t1", "entity:paris"), ("t2", "entity:london")] {
            state
                .append_turn(Turn {
                    turn_id: turn_id.into(),
                    expression: SemanticExpression::Entity(EntityId::new_unchecked(entity)),
                })
                .unwrap();
        }
        let ReferenceResolution::Resolved(candidate) = state.resolve_entity("entity:") else {
            panic!("expected recency to resolve the newer turn");
        };
        assert_eq!(candidate.turn_id, "t2");
    }

    #[test]
    fn corrupt_mention_cannot_be_verified() {
        let mut state = ConversationState::default();
        state.append_turn(turn("t1")).unwrap();
        state.mentions.push(EntityMention {
            entity: EntityId::new_unchecked("entity:paris"),
            turn_id: "missing".into(),
            salience: 1,
        });
        assert_eq!(
            state.verify(),
            Err(ConversationError::UnknownTurn("missing".into()))
        );
    }

    #[test]
    fn corrupted_serialized_state_is_rejected() {
        let value = serde_json::json!({
            "turns": [{
                "turn_id": "t1",
                "expression": {"Value": {"Boolean": true}}
            }],
            "mentions": [{
                "entity": "entity:paris",
                "turn_id": "missing",
                "salience": 1
            }]
        });
        assert!(serde_json::from_value::<ConversationState>(value).is_err());
    }

    #[test]
    fn conversation_file_round_trip_is_verified_and_atomic() {
        let root =
            std::env::temp_dir().join(format!("lexflex-conversation-{}", std::process::id()));
        let path = root.join("state.json");
        let mut state = ConversationState::default();
        state.append_turn(turn("t1")).expect("turn");
        state.save_to_file(&path).expect("save");
        let loaded = ConversationState::load_from_file(&path).expect("load");
        assert_eq!(loaded, state);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn concurrent_conversation_saves_leave_one_valid_file() {
        let root = std::env::temp_dir().join(format!(
            "lexflex-conversation-concurrent-{}",
            std::process::id()
        ));
        let path = root.join("state.json");
        let state = ConversationState::default();
        let workers = (0..8)
            .map(|_| {
                let path = path.clone();
                let state = state.clone();
                std::thread::spawn(move || state.save_to_file(&path).expect("save"))
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().expect("worker");
        }
        assert!(ConversationState::load_from_file(&path).is_ok());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn oversized_persisted_state_is_rejected_before_deserialization() {
        let root = std::env::temp_dir().join(format!(
            "lexflex-conversation-oversized-{}",
            std::process::id()
        ));
        let path = root.join("state.json");
        std::fs::create_dir_all(&root).expect("root");
        std::fs::write(&path, vec![b'{'; MAX_PERSISTED_BYTES + 1]).expect("payload");
        assert_eq!(
            ConversationState::load_from_file(&path),
            Err(ConversationError::PersistenceTooLarge)
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
