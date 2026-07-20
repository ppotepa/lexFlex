use crate::{Evidence, EvidenceError, EvidenceId};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct EvidenceSet {
    entries: BTreeMap<EvidenceId, Evidence>,
}

impl EvidenceSet {
    pub fn try_from_iter(
        values: impl IntoIterator<Item = Evidence>,
    ) -> Result<Self, EvidenceError> {
        let mut entries = BTreeMap::new();
        for evidence in values {
            evidence.verify()?;
            match entries.get(evidence.id()) {
                None => {
                    entries.insert(evidence.id().clone(), evidence);
                }
                Some(existing) if existing == &evidence => {}
                Some(_) => return Err(EvidenceError::ConflictingEvidence(evidence.id().clone())),
            }
        }
        Ok(Self { entries })
    }

    pub fn singleton(evidence: Evidence) -> Result<Self, EvidenceError> {
        Self::try_from_iter([evidence])
    }

    pub fn verify(&self) -> Result<(), EvidenceError> {
        for (key, evidence) in &self.entries {
            if key != evidence.id() {
                return Err(EvidenceError::KeyMismatch {
                    key: key.clone(),
                    evidence: evidence.id().clone(),
                });
            }
            evidence.verify()?;
        }
        Ok(())
    }

    pub fn merge(&mut self, incoming: Self) -> Result<usize, EvidenceError> {
        self.verify()?;
        incoming.verify()?;

        let additions = incoming
            .entries
            .iter()
            .filter_map(|(id, evidence)| match self.entries.get(id) {
                None => Some(Ok((id.clone(), evidence.clone()))),
                Some(existing) if existing == evidence => None,
                Some(_) => Some(Err(EvidenceError::ConflictingEvidence(id.clone()))),
            })
            .collect::<Result<Vec<_>, EvidenceError>>()?;

        let added = additions.len();
        self.entries.extend(additions);
        Ok(added)
    }
}

impl<'de> Deserialize<'de> for EvidenceSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries = BTreeMap::<EvidenceId, Evidence>::deserialize(deserializer)?;
        let value = Self { entries };
        value.verify().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

impl EvidenceSet {
    pub fn get(&self, id: &EvidenceId) -> Option<&Evidence> {
        self.entries.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&EvidenceId, &Evidence)> {
        self.entries.iter()
    }

    pub fn keys(&self) -> impl Iterator<Item = &EvidenceId> {
        self.entries.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &Evidence> {
        self.entries.values()
    }

    pub fn into_values(self) -> impl Iterator<Item = Evidence> {
        self.entries.into_values()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(source_id: &str) -> Evidence {
        Evidence::create(source_id, None, None).expect("valid evidence")
    }

    #[test]
    fn merge_is_transactional_on_late_conflict() {
        let z = evidence("source:z");
        let mut existing = EvidenceSet::singleton(z.clone()).expect("existing evidence set");

        let conflicting = evidence("source:changed");
        let incoming = EvidenceSet {
            entries: BTreeMap::from([
                (evidence("source:a").id().clone(), evidence("source:a")),
                (z.id().clone(), conflicting),
            ]),
        };

        let before = existing.clone();
        let result = existing.merge(incoming);

        assert!(result.is_err());
        assert_eq!(existing, before);
    }
}
