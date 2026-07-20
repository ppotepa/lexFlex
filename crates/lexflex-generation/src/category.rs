use lexflex_model::SemanticExpression;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClausePlan<'a> {
    pub(crate) subject: &'a SemanticExpression,
    pub(crate) predicate: &'a SemanticExpression,
}

pub(crate) fn plan_satisfies(expression: &SemanticExpression) -> Option<ClausePlan<'_>> {
    match expression {
        SemanticExpression::Satisfies { subject, predicate } => {
            Some(ClausePlan { subject, predicate })
        }
        _ => None,
    }
}
