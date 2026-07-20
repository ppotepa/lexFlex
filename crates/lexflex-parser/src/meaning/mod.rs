mod application;
mod fresh;
mod instance;

pub(crate) use application::apply_meaning;
pub(crate) use fresh::{count_lingua_nodes, instantiate_meaning};
pub(crate) use instance::{BooleanOperator, MeaningInstance};
