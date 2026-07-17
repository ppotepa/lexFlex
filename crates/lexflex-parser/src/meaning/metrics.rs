#![allow(dead_code)]

pub(crate) fn count_lingua_nodes(expression: &lexflex_lingua::LinguaExpression) -> usize {
    super::fresh::count_lingua_nodes(expression)
}
