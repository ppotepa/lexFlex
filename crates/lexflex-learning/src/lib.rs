#![forbid(unsafe_code)]

use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Observation {
    pub source_id: String,
    pub surface: String,
    pub context: Option<String>,
}

impl<'de> Deserialize<'de> for Observation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            source_id: String,
            surface: String,
            context: Option<String>,
        }
        let wire = Wire::deserialize(deserializer)?;
        let observation = Self {
            source_id: wire.source_id,
            surface: wire.surface,
            context: wire.context,
        };
        observation.verify().map_err(D::Error::custom)?;
        Ok(observation)
    }
}

impl Observation {
    pub fn verify(&self) -> Result<(), LearningError> {
        if self.source_id.trim().is_empty() {
            return Err(LearningError::EmptyObservationId);
        }
        if self.surface.trim().is_empty() {
            return Err(LearningError::EmptyObservationSurface);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Proposal {
    pub proposal_id: String,
    pub observation_id: String,
    pub summary: String,
}

impl<'de> Deserialize<'de> for Proposal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            proposal_id: String,
            observation_id: String,
            summary: String,
        }
        let wire = Wire::deserialize(deserializer)?;
        let proposal = Self {
            proposal_id: wire.proposal_id,
            observation_id: wire.observation_id,
            summary: wire.summary,
        };
        proposal.verify().map_err(D::Error::custom)?;
        Ok(proposal)
    }
}

impl Proposal {
    pub fn verify(&self) -> Result<(), LearningError> {
        if self.proposal_id.trim().is_empty() {
            return Err(LearningError::EmptyProposalId);
        }
        if self.observation_id.trim().is_empty() {
            return Err(LearningError::EmptyObservationId);
        }
        if self.summary.trim().is_empty() {
            return Err(LearningError::EmptyProposalSummary);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningProposal {
    pub proposal: Proposal,
    pub expected_behavior: String,
    pub status: ProposalStatus,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LearningOverlay {
    proposals: Vec<LearningProposal>,
}

impl<'de> Deserialize<'de> for LearningOverlay {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            proposals: Vec<LearningProposal>,
        }
        let wire = Wire::deserialize(deserializer)?;
        let overlay = Self {
            proposals: wire.proposals,
        };
        overlay.verify().map_err(D::Error::custom)?;
        Ok(overlay)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LearningError {
    #[error("proposal id is empty")]
    EmptyProposalId,
    #[error("observation id is empty")]
    EmptyObservationId,
    #[error("observation surface is empty")]
    EmptyObservationSurface,
    #[error("proposal is not pending")]
    NotPending,
    #[error("proposal is not approved")]
    NotApproved,
    #[error("expected behavior is empty")]
    EmptyExpectedBehavior,
    #[error("proposal summary is empty")]
    EmptyProposalSummary,
    #[error("proposal already exists: {0}")]
    DuplicateProposal(String),
}

impl LearningOverlay {
    pub fn verify(&self) -> Result<(), LearningError> {
        let mut checked = Self::default();
        for item in &self.proposals {
            checked.propose(
                &Observation {
                    source_id: item.proposal.observation_id.clone(),
                    surface: "validated-observation".into(),
                    context: None,
                },
                item.proposal.clone(),
                item.expected_behavior.clone(),
            )?;
            let checked_item = checked
                .find_mut(&item.proposal.proposal_id)
                .map_err(|_| LearningError::DuplicateProposal(item.proposal.proposal_id.clone()))?;
            checked_item.status = item.status;
        }
        Ok(())
    }

    pub fn propose(
        &mut self,
        observation: &Observation,
        proposal: Proposal,
        expected_behavior: String,
    ) -> Result<(), LearningError> {
        observation.verify()?;
        proposal.verify()?;
        if expected_behavior.trim().is_empty() {
            return Err(LearningError::EmptyExpectedBehavior);
        }
        if proposal.observation_id != observation.source_id {
            return Err(LearningError::EmptyObservationId);
        }
        if self
            .proposals
            .iter()
            .any(|item| item.proposal.proposal_id == proposal.proposal_id)
        {
            return Err(LearningError::DuplicateProposal(proposal.proposal_id));
        }
        self.proposals.push(LearningProposal {
            proposal,
            expected_behavior,
            status: ProposalStatus::Pending,
        });
        Ok(())
    }

    pub fn approve(&mut self, proposal_id: &str) -> Result<(), LearningError> {
        let item = self.find_mut(proposal_id)?;
        if item.status != ProposalStatus::Pending {
            return Err(LearningError::NotPending);
        }
        item.status = ProposalStatus::Approved;
        Ok(())
    }

    pub fn reject(&mut self, proposal_id: &str) -> Result<(), LearningError> {
        let item = self.find_mut(proposal_id)?;
        if item.status != ProposalStatus::Pending {
            return Err(LearningError::NotPending);
        }
        item.status = ProposalStatus::Rejected;
        Ok(())
    }

    pub fn approved(&self) -> impl Iterator<Item = &LearningProposal> {
        self.proposals
            .iter()
            .filter(|item| item.status == ProposalStatus::Approved)
    }

    pub fn promote(&self, proposal_id: &str) -> Result<Proposal, LearningError> {
        let item = self
            .proposals
            .iter()
            .find(|item| item.proposal.proposal_id == proposal_id)
            .ok_or_else(|| LearningError::DuplicateProposal(proposal_id.to_owned()))?;
        if item.status != ProposalStatus::Approved {
            return Err(LearningError::NotApproved);
        }
        Ok(item.proposal.clone())
    }

    pub fn proposals(&self) -> &[LearningProposal] {
        &self.proposals
    }

    fn find_mut(&mut self, proposal_id: &str) -> Result<&mut LearningProposal, LearningError> {
        self.proposals
            .iter_mut()
            .find(|item| item.proposal.proposal_id == proposal_id)
            .ok_or_else(|| LearningError::DuplicateProposal(proposal_id.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_requires_explicit_approval_before_overlay_use() {
        let observation = Observation {
            source_id: "obs-1".into(),
            surface: "Paris".into(),
            context: None,
        };
        let proposal = Proposal {
            proposal_id: "p-1".into(),
            observation_id: "obs-1".into(),
            summary: "candidate".into(),
        };
        let mut overlay = LearningOverlay::default();
        overlay
            .propose(&observation, proposal, "resolve to PARIS".into())
            .unwrap();
        assert_eq!(overlay.approved().count(), 0);
        overlay.approve("p-1").unwrap();
        assert_eq!(overlay.approved().count(), 1);
        assert_eq!(overlay.promote("p-1").unwrap().proposal_id, "p-1");
    }

    #[test]
    fn corrupted_overlay_deserialization_is_rejected() {
        let value = serde_json::json!({"proposals": [
            {
                "proposal": {
                    "proposal_id": "p-1",
                    "observation_id": "obs-1",
                    "summary": "candidate"
                },
                "expected_behavior": "resolve",
                "status": "Pending"
            },
            {
                "proposal": {
                    "proposal_id": "p-1",
                    "observation_id": "obs-1",
                    "summary": "duplicate"
                },
                "expected_behavior": "resolve",
                "status": "Pending"
            }
        ]});
        assert!(serde_json::from_value::<LearningOverlay>(value).is_err());
    }

    #[test]
    fn learning_value_objects_reject_invalid_serde_payloads() {
        assert!(serde_json::from_str::<Observation>(
            r#"{"source_id":"","surface":"Paris","context":null}"#
        )
        .is_err());
        assert!(serde_json::from_str::<Proposal>(
            r#"{"proposal_id":"p-1","observation_id":"obs-1","summary":""}"#
        )
        .is_err());
    }
}
