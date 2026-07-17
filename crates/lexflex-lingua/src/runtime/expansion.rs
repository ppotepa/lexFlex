use crate::runtime::{InterpreterState, RuntimeError};
use lexflex_model::ConceptId;

impl InterpreterState<'_> {
    #[allow(dead_code)]
    pub(super) fn with_expansion<T>(
        &mut self,
        concept: &ConceptId,
        operation: impl FnOnce(&mut Self) -> Result<T, RuntimeError>,
    ) -> Result<T, RuntimeError> {
        self.enter_expansion(concept)?;
        let result = operation(self);
        self.leave_expansion();
        result
    }
}
