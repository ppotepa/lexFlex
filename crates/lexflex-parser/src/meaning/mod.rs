mod allocator;
mod application;
mod category_freshener;
mod error;
mod expression_rename;
mod fresh;
mod instance;
mod metrics;
mod query_freshener;
mod symbol_freshener;

pub(crate) use application::apply_meaning;
pub(crate) use fresh::{count_lingua_nodes, instantiate_meaning};
pub(crate) use instance::MeaningInstance;
