use std::collections::HashMap;

use crate::core::interlingua::{
    ConceptId, Entity, FeatureBundle, Frame, SemanticRole,
};
use crate::error::DeductionError;

#[derive(Debug, Clone)]
pub struct OntologyEntry {
    pub id: ConceptId,
    pub parent: Option<ConceptId>,
    pub features: FeatureBundle,
    pub allowed_roles: Vec<SemanticRole>,
}

#[derive(Debug, Clone)]
pub struct Ontology {
    entries: HashMap<ConceptId, OntologyEntry>,
}

impl Ontology {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, entry: OntologyEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }

    pub fn get(&self, id: &ConceptId) -> Option<&OntologyEntry> {
        self.entries.get(id)
    }

    pub fn is_a(&self, child: &ConceptId, parent: &ConceptId) -> bool {
        if child == parent {
            return true;
        }
        if let Some(entry) = self.entries.get(child) {
            if let Some(ref p) = entry.parent {
                return self.is_a(p, parent);
            }
        }
        false
    }

    pub fn validate_semantic_types(&self, frame: &Frame) -> Result<(), DeductionError> {
        for (role, entity) in self.frame_role_entities(frame) {
            match role {
                SemanticRole::Agent | SemanticRole::Experiencer | SemanticRole::Recipient => {
                    if entity.features.animacy
                        == Some(crate::core::interlingua::Animacy::Inanimate)
                    {
                        return Err(DeductionError::SemanticTypeViolation {
                            role,
                            expected: ConceptId::new("animate_entity"),
                            found: entity.concept.clone(),
                        });
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn inherit_features(&self, entity: &mut Entity) {
        let mut current = entity.concept.clone();
        let mut accumulated = FeatureBundle::default();

        while let Some(entry) = self.entries.get(&current) {
            merge_features(&mut accumulated, &entry.features);
            match &entry.parent {
                Some(parent) => current = parent.clone(),
                None => break,
            }
        }

        apply_defaults(&mut entity.features, &accumulated);
    }

    fn frame_role_entities<'a>(&self, frame: &'a Frame) -> Vec<(SemanticRole, &'a Entity)> {
        match frame {
            Frame::Transfer { agent, recipient, theme } => {
                vec![
                    (SemanticRole::Agent, agent),
                    (SemanticRole::Recipient, recipient),
                    (SemanticRole::Theme, theme),
                ]
            }
            Frame::Motion { mover, source, goal, .. } => {
                let mut v = vec![(SemanticRole::Agent, mover)];
                if let Some(s) = source { v.push((SemanticRole::Source, s)); }
                if let Some(g) = goal { v.push((SemanticRole::Goal, g)); }
                v
            }
            Frame::Perception { experiencer, stimulus } => {
                vec![
                    (SemanticRole::Experiencer, experiencer),
                    (SemanticRole::Stimulus, stimulus),
                ]
            }
            Frame::Cognition { cognizer, content } => {
                vec![
                    (SemanticRole::Cognizer, cognizer),
                    (SemanticRole::Content, content),
                ]
            }
            Frame::Emotion { experiencer, stimulus } => {
                vec![
                    (SemanticRole::Experiencer, experiencer),
                    (SemanticRole::Stimulus, stimulus),
                ]
            }
            Frame::Destruction { agent, patient, .. } => {
                vec![
                    (SemanticRole::Agent, agent),
                    (SemanticRole::Patient, patient),
                ]
            }
            Frame::Consumption { agent, patient } => {
                vec![
                    (SemanticRole::Agent, agent),
                    (SemanticRole::Patient, patient),
                ]
            }
            Frame::Communication { speaker, addressee, message } => {
                let mut v = vec![
                    (SemanticRole::Speaker, speaker),
                    (SemanticRole::Message, message),
                ];
                if let Some(a) = addressee {
                    v.push((SemanticRole::Recipient, a));
                }
                v
            }
            Frame::Creation { creator, created, .. } => {
                vec![
                    (SemanticRole::Agent, creator),
                    (SemanticRole::Theme, created),
                ]
            }
            Frame::Statement { subject, property } => {
                vec![
                    (SemanticRole::Topic, subject),
                    (SemanticRole::Theme, property),
                ]
            }
            Frame::Existence { entity, .. } => {
                vec![(SemanticRole::Theme, entity)]
            }
            Frame::Possession { possessor, possessed } => {
                vec![
                    (SemanticRole::Agent, possessor),
                    (SemanticRole::Theme, possessed),
                ]
            }
            Frame::Custom { roles, .. } => roles.iter().map(|(r, e)| (*r, e)).collect(),
        }
    }
}

impl Default for Ontology {
    fn default() -> Self {
        Self::new()
    }
}

fn merge_features(target: &mut FeatureBundle, source: &FeatureBundle) {
    if target.gender.is_none() {
        target.gender = source.gender;
    }
    if target.animacy.is_none() {
        target.animacy = source.animacy;
    }
    if target.countability.is_none() {
        target.countability = source.countability;
    }
    if target.concreteness.is_none() {
        target.concreteness = source.concreteness;
    }
}

fn apply_defaults(target: &mut FeatureBundle, defaults: &FeatureBundle) {
    if target.gender.is_none() {
        target.gender = defaults.gender;
    }
    if target.animacy.is_none() {
        target.animacy = defaults.animacy;
    }
    if target.countability.is_none() {
        target.countability = defaults.countability;
    }
}
