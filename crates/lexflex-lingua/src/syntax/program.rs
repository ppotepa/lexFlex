use crate::id::ProgramId;
use crate::syntax::LinguaDeclaration;
use crate::syntax::LinguaExpression;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinguaProgram {
    pub id: ProgramId,
    pub declarations: Vec<LinguaDeclaration>,
    pub entry: LinguaExpression,
}
