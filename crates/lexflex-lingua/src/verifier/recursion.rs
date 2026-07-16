use lexflex_model::ConceptId;
use std::collections::BTreeMap;

pub fn find_recursive_concepts(graph: &BTreeMap<ConceptId, Vec<ConceptId>>) -> Vec<Vec<ConceptId>> {
    enum Mark {
        Temporary,
        Permanent,
    }
    let mut marks: BTreeMap<ConceptId, Mark> = BTreeMap::new();
    let mut stack = Vec::new();
    let mut cycles = Vec::new();

    fn visit(
        node: &ConceptId,
        graph: &BTreeMap<ConceptId, Vec<ConceptId>>,
        marks: &mut BTreeMap<ConceptId, Mark>,
        stack: &mut Vec<ConceptId>,
        cycles: &mut Vec<Vec<ConceptId>>,
    ) {
        if matches!(marks.get(node), Some(Mark::Permanent)) {
            return;
        }
        if matches!(marks.get(node), Some(Mark::Temporary)) {
            if let Some(index) = stack.iter().position(|candidate| candidate == node) {
                cycles.push(stack[index..].to_vec());
            }
            return;
        }
        marks.insert(node.clone(), Mark::Temporary);
        stack.push(node.clone());
        if let Some(children) = graph.get(node) {
            for child in children {
                visit(child, graph, marks, stack, cycles);
            }
        }
        stack.pop();
        marks.insert(node.clone(), Mark::Permanent);
    }

    for node in graph.keys() {
        visit(node, graph, &mut marks, &mut stack, &mut cycles);
    }

    cycles
}
