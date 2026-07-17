use crate::solve::{QuerySolution, SolveError};
use lexflex_model::{canonical_hash, AssertionId, CanonicalDigest, EvidenceId};

pub(crate) fn query_solution_sort_key(
    solution: &QuerySolution,
) -> Result<(AssertionId, CanonicalDigest, Vec<EvidenceId>), SolveError> {
    let substitution_hash =
        canonical_hash(&solution.substitution).map_err(SolveError::CanonicalHash)?;
    let evidence_ids = solution.evidence.keys().cloned().collect();
    Ok((solution.assertion_id.clone(), substitution_hash, evidence_ids))
}
