use crate::{FeatureConflict, LexicalSenseId, ValencySlotId};
use lexflex_model::ParameterId;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LanguageCompileError {
    #[error("duplicate valency application rank {rank} in sense {sense_id}")]
    DuplicateValencyRank { sense_id: LexicalSenseId, rank: u16 },
    #[error("surface-marked valency slot {slot_id} requires an atomic argument category")]
    MarkedArgumentRequiresAtom { slot_id: ValencySlotId },
    #[error("required parameter {parameter} is not represented by valency in sense {sense_id}")]
    MissingRequiredParameter {
        sense_id: LexicalSenseId,
        parameter: ParameterId,
    },
    #[error("feature conflict compiling sense {sense_id}: {conflict:?}")]
    FeatureConflict {
        sense_id: LexicalSenseId,
        conflict: FeatureConflict,
    },
    #[error("compiled category exceeds depth limit in sense {sense_id}: {depth}")]
    CategoryDepthExceeded {
        sense_id: LexicalSenseId,
        depth: usize,
    },
}
