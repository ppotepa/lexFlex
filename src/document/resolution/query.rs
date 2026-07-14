use crate::document::id::SentenceId;

use super::id::{EntityClusterId, ResolutionDecisionId, ResolutionMentionRef, SyntheticMentionId};
use super::model::{
    DocumentEntityResolution, EntityResolutionDecision, MentionResolutionProfile, ResolvedEntityCluster,
    SyntheticResolutionMention,
};

pub struct DocumentEntityResolutionQuery<'a> {
    resolution: &'a DocumentEntityResolution,
}

impl<'a> DocumentEntityResolutionQuery<'a> {
    pub fn new(resolution: &'a DocumentEntityResolution) -> Self {
        Self { resolution }
    }

    pub fn mention_profile(&self, mention: &ResolutionMentionRef) -> Option<&'a MentionResolutionProfile> {
        self.resolution.mention_profiles.get(mention)
    }

    pub fn synthetic_mention(&self, id: &SyntheticMentionId) -> Option<&'a SyntheticResolutionMention> {
        self.resolution.synthetic_mentions.get(id)
    }

    pub fn decision(&self, id: &ResolutionDecisionId) -> Option<&'a EntityResolutionDecision> {
        self.resolution.decisions.get(id)
    }

    pub fn decision_for_mention(&self, mention: &ResolutionMentionRef) -> Option<&'a EntityResolutionDecision> {
        self.resolution.decision_for_mention(mention)
    }

    pub fn cluster(&self, id: &EntityClusterId) -> Option<&'a ResolvedEntityCluster> {
        self.resolution.clusters.get(id)
    }

    pub fn cluster_for_mention(&self, mention: &ResolutionMentionRef) -> Option<&'a ResolvedEntityCluster> {
        self.resolution
            .decision_for_mention(mention)
            .and_then(|decision| decision.result_cluster.as_ref())
            .and_then(|cluster_id| self.resolution.clusters.get(cluster_id))
            .or_else(|| {
                self.resolution.clusters.values().find(|cluster| {
                    cluster.mention_refs.iter().any(|cluster_mention| cluster_mention == mention)
                })
            })
    }

    pub fn clusters_by_name(&self, name: &str) -> Vec<&'a ResolvedEntityCluster> {
        self.resolution
            .clusters
            .values()
            .filter(|cluster| cluster.canonical_name.as_deref() == Some(name))
            .collect()
    }

    pub fn mentions_in_cluster(&self, id: &EntityClusterId) -> Vec<&'a MentionResolutionProfile> {
        self.resolution
            .clusters
            .get(id)
            .map(|cluster| {
                cluster
                    .mention_refs
                    .iter()
                    .filter_map(|mention| self.resolution.mention_profiles.get(mention))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn decisions_in_sentence(
        &self,
        sentence_id: &SentenceId,
    ) -> Vec<&'a EntityResolutionDecision> {
        self.resolution
            .decisions
            .values()
            .filter(|decision| &decision.sentence_id == sentence_id)
            .collect()
    }

    pub fn zero_mentions_in_sentence(
        &self,
        sentence_id: &SentenceId,
    ) -> Vec<&'a SyntheticResolutionMention> {
        self.resolution
            .synthetic_mentions
            .values()
            .filter(|mention| &mention.source_sentence_id == sentence_id)
            .collect()
    }
}
