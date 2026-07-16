use lexflex_model::AssertionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssertionWriteOutcome {
    Inserted {
        assertion_id: AssertionId,
    },
    EvidenceMerged {
        assertion_id: AssertionId,
        added_evidence: usize,
    },
    Unchanged {
        assertion_id: AssertionId,
    },
}
