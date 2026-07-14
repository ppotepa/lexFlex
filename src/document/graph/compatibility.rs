use crate::core::interlingua::{ConceptId, FeatureBundle};

pub trait EntityCompatibility {
    fn concepts_compatible(&self, left: &ConceptId, right: &ConceptId) -> bool;
    fn features_compatible(&self, left: &FeatureBundle, right: &FeatureBundle) -> bool;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultEntityCompatibility;

impl EntityCompatibility for DefaultEntityCompatibility {
    fn concepts_compatible(&self, left: &ConceptId, right: &ConceptId) -> bool {
        left == right
    }

    fn features_compatible(&self, left: &FeatureBundle, right: &FeatureBundle) -> bool {
        left.gender == right.gender
            && left.number == right.number
            && left.animacy == right.animacy
            && left.person == right.person
            && left.countability == right.countability
            && left.concreteness == right.concreteness
    }
}
