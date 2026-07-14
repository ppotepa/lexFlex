use std::collections::BTreeMap;
use std::path::Path;

use crate::core::interlingua::ConceptDefinition;
use crate::core::ontology::{Ontology, OntologyEntry};
use crate::core::interlingua::{ConceptId, SemanticRole};
use crate::data::descriptor::LanguageDescriptor;
use crate::data::lexicon::{LexEntry, Lexicon};
use crate::data::morphology::MorphParadigm;
use crate::error::DataError;

pub fn load_concepts(path: &Path) -> Result<Vec<ConceptDefinition>, DataError> {
    let content = std::fs::read_to_string(path).map_err(|_| DataError::FileNotFound {
        path: path.display().to_string(),
    })?;
    ron::from_str(&content).map_err(|e| DataError::InvalidData {
        path: path.display().to_string(),
        message: e.to_string(),
    })
}

pub fn load_lexicon(path: &Path) -> Result<Lexicon, DataError> {
    let content = std::fs::read_to_string(path).map_err(|_| DataError::FileNotFound {
        path: path.display().to_string(),
    })?;
    let entries: Vec<(String, LexEntry)> =
        ron::from_str(&content).map_err(|e| DataError::InvalidData {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
    let mut lexicon = Lexicon::new();
    for (key, entry) in entries {
        lexicon.add_entry(key, entry);
    }
    Ok(lexicon)
}

pub fn load_paradigms(path: &Path) -> Result<Vec<MorphParadigm>, DataError> {
    let content = std::fs::read_to_string(path).map_err(|_| DataError::FileNotFound {
        path: path.display().to_string(),
    })?;
    ron::from_str(&content).map_err(|e| DataError::InvalidData {
        path: path.display().to_string(),
        message: e.to_string(),
    })
}

pub fn load_descriptor(path: &Path) -> Result<LanguageDescriptor, DataError> {
    let content = std::fs::read_to_string(path).map_err(|_| DataError::FileNotFound {
        path: path.display().to_string(),
    })?;
    ron::from_str(&content).map_err(|e| DataError::InvalidData {
        path: path.display().to_string(),
        message: e.to_string(),
    })
}

#[derive(Debug, serde::Deserialize)]
struct OntologyFile {
    concepts: BTreeMap<String, OntologyFileEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct OntologyFileEntry {
    id: String,
    parent: Option<String>,
    features: crate::core::interlingua::FeatureBundle,
    allowed_roles: Vec<String>,
}

/// Load the semantic hierarchy separately from language-specific lexicons.
pub fn load_ontology(path: &Path) -> Result<Ontology, DataError> {
    let content = std::fs::read_to_string(path).map_err(|_| DataError::FileNotFound {
        path: path.display().to_string(),
    })?;
    let file: OntologyFile = ron::from_str(&content).map_err(|e| DataError::InvalidData {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    let mut ontology = Ontology::new();
    for entry in file.concepts.into_values() {
        ontology.add_entry(OntologyEntry {
            id: ConceptId::new(&entry.id),
            parent: entry.parent.map(|parent| ConceptId::new(&parent)),
            features: entry.features,
            allowed_roles: parse_roles(&entry.allowed_roles),
        });
    }
    Ok(ontology)
}

fn parse_roles(roles: &[String]) -> Vec<SemanticRole> {
    roles
        .iter()
        .filter_map(|role| match role.as_str() {
            "Agent" => Some(SemanticRole::Agent),
            "Patient" => Some(SemanticRole::Patient),
            "Theme" => Some(SemanticRole::Theme),
            "Recipient" => Some(SemanticRole::Recipient),
            "Experiencer" => Some(SemanticRole::Experiencer),
            "Stimulus" => Some(SemanticRole::Stimulus),
            "Source" => Some(SemanticRole::Source),
            "Goal" => Some(SemanticRole::Goal),
            "Location" => Some(SemanticRole::Location),
            "Instrument" => Some(SemanticRole::Instrument),
            "Beneficiary" => Some(SemanticRole::Beneficiary),
            "Topic" => Some(SemanticRole::Topic),
            "Creator" => Some(SemanticRole::Creator),
            "Created" => Some(SemanticRole::Created),
            "Cognizer" => Some(SemanticRole::Cognizer),
            "Content" => Some(SemanticRole::Content),
            "Speaker" => Some(SemanticRole::Speaker),
            "Message" => Some(SemanticRole::Message),
            "Accompaniment" => Some(SemanticRole::Accompaniment),
            _ => None,
        })
        .collect()
}

pub fn build_ontology_from_concepts(concepts: &[ConceptDefinition]) -> Ontology {
    let mut ontology = Ontology::new();
    for concept in concepts {
        let roles: Vec<SemanticRole> = concept
            .roles
            .iter()
            .filter_map(|r| match r.as_str() {
                "Agent" => Some(SemanticRole::Agent),
                "Patient" => Some(SemanticRole::Patient),
                "Theme" => Some(SemanticRole::Theme),
                "Recipient" => Some(SemanticRole::Recipient),
                "Experiencer" => Some(SemanticRole::Experiencer),
                "Stimulus" => Some(SemanticRole::Stimulus),
                "Source" => Some(SemanticRole::Source),
                "Goal" => Some(SemanticRole::Goal),
                "Location" => Some(SemanticRole::Location),
                "Instrument" => Some(SemanticRole::Instrument),
                "Beneficiary" => Some(SemanticRole::Beneficiary),
                "Topic" => Some(SemanticRole::Topic),
                "Creator" => Some(SemanticRole::Agent),
                "Created" => Some(SemanticRole::Theme),
                "Speaker" => Some(SemanticRole::Agent),
                "Addressee" => Some(SemanticRole::Recipient),
                "Message" => Some(SemanticRole::Theme),
                "Content" => Some(SemanticRole::Theme),
                "Cognizer" => Some(SemanticRole::Experiencer),
                _ => None,
            })
            .collect();

        ontology.add_entry(OntologyEntry {
            id: ConceptId::new(&concept.id),
            parent: None,
            features: concept.inherent_features.clone(),
            allowed_roles: roles,
        });
    }
    ontology
}
