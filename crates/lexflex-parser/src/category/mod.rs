mod apply;
mod binding;
mod features;
mod outcome;
mod resolution;
mod substitution;
mod unify;

pub(crate) use apply::{apply_backward, apply_forward, resolve_query_variables};
pub(crate) use outcome::{CategoryOutcome, CategoryResult};
pub(crate) use resolution::CategoryResolutionError;
pub(crate) use substitution::CategorySubstitution;
pub(crate) use unify::unify_category;
