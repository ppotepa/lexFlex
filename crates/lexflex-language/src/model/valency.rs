use crate::{SlashDirection, SurfaceRelationId, SyntacticCategory, ValencySlotId};
use lexflex_model::ParameterId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValencySlot {
    pub id: ValencySlotId,
    pub parameter: ParameterId,
    pub argument_category: SyntacticCategory,
    #[serde(default)]
    pub surface_relation: Option<SurfaceRelationId>,
    pub direction: SlashDirection,
    #[serde(default)]
    pub application_rank: u16,
    pub required: bool,
}
