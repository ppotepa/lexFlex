use crate::solve::UnifyError;
use lexflex_model::VariableId;

#[derive(Debug, Default, Clone)]
pub(crate) struct BoundVariableScope {
    frames: Vec<(VariableId, VariableId)>,
}

impl BoundVariableScope {
    pub(crate) fn push(&mut self, left: VariableId, right: VariableId) {
        self.frames.push((left, right));
    }

    pub(crate) fn pop(&mut self) -> Result<(), UnifyError> {
        self.frames
            .pop()
            .map(|_| ())
            .ok_or(UnifyError::BoundScopeUnderflow)
    }

    pub(crate) fn right_for_left(&self, left: &VariableId) -> Option<&VariableId> {
        self.frames
            .iter()
            .rev()
            .find(|(bound_left, _)| bound_left == left)
            .map(|(_, right)| right)
    }

    pub(crate) fn left_for_right(&self, right: &VariableId) -> Option<&VariableId> {
        self.frames
            .iter()
            .rev()
            .find(|(_, bound_right)| bound_right == right)
            .map(|(left, _)| left)
    }
}
