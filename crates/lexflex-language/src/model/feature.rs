use crate::{FeatureName, FeatureValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FeatureConflict {
    pub name: FeatureName,
    pub left: FeatureValue,
    pub right: FeatureValue,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FeatureStructure {
    pub values: BTreeMap<FeatureName, FeatureValue>,
}

impl FeatureStructure {
    pub fn get(&self, name: &FeatureName) -> Option<&FeatureValue> {
        self.values.get(name)
    }

    pub fn contains_all(&self, required: &Self) -> bool {
        required
            .values
            .iter()
            .all(|(name, value)| self.values.get(name) == Some(value))
    }

    pub fn merged(&self, other: &Self) -> Result<Self, FeatureConflict> {
        self.unify(other)
    }

    pub fn unify(&self, other: &Self) -> Result<Self, FeatureConflict> {
        let mut output = self.clone();
        for (name, value) in &other.values {
            match output.values.get(name) {
                Some(existing) if existing != value => {
                    return Err(FeatureConflict {
                        name: name.clone(),
                        left: existing.clone(),
                        right: value.clone(),
                    });
                }
                _ => {
                    output.values.insert(name.clone(), value.clone());
                }
            }
        }
        Ok(output)
    }
}
