use crate::{ConceptCatalog, ConceptId};
use std::collections::BTreeMap;

pub(crate) fn find_hierarchy_cycle(catalog: &ConceptCatalog) -> Option<Vec<String>> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mark {
        Visiting,
        Done,
    }

    fn dfs(
        node: &str,
        catalog: &ConceptCatalog,
        marks: &mut BTreeMap<String, Mark>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        match marks.get(node) {
            Some(Mark::Visiting) => {
                let start = stack.iter().position(|current| current == node)?;
                let mut cycle = stack[start..].to_vec();
                cycle.push(node.to_string());
                Some(cycle)
            }
            Some(Mark::Done) => None,
            None => {
                marks.insert(node.to_string(), Mark::Visiting);
                stack.push(node.to_string());

                if let Some(parents) = catalog.parents.get(&ConceptId::new_unchecked(node)) {
                    for parent in parents {
                        if let Some(cycle) = dfs(parent.as_str(), catalog, marks, stack) {
                            return Some(cycle);
                        }
                    }
                }

                stack.pop();
                marks.insert(node.to_string(), Mark::Done);
                None
            }
        }
    }

    let mut marks = BTreeMap::new();
    let mut stack = Vec::new();
    for node in catalog.parents.keys().map(|id| id.as_str().to_string()) {
        if let Some(cycle) = dfs(&node, catalog, &mut marks, &mut stack) {
            return Some(cycle);
        }
    }
    None
}
