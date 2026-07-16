use crate::diagnostic::{ParseBudgetLimit, ParseError};
use crate::explain::DerivationNode;
use crate::meaning::{apply_meaning, MeaningInstance};
use lexflex_language::{CategoryType, CategoryTypeVariableId, SlashDirection, SyntacticCategory};
use lexflex_model::{canonical_hash, ConceptCatalog, TypeRelation};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(crate) struct ChartItem {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) category: SyntacticCategory,
    pub(crate) meaning: MeaningInstance,
    pub(crate) score: i64,
    pub(crate) derivation: DerivationNode,
}

pub(crate) fn insert_item(
    cell: &mut Vec<ChartItem>,
    item: ChartItem,
    limit: usize,
) -> Result<(), ParseError> {
    let key = item_key(&item);
    if let Some(existing) = cell.iter_mut().find(|existing| item_key(existing) == key) {
        if item.score < existing.score {
            *existing = item;
        }
        return Ok(());
    }
    if cell.len() >= limit {
        return Err(ParseError::BudgetExceeded(ParseBudgetLimit::CellItemLimit));
    }
    cell.push(item);
    cell.sort_by(|left, right| {
        left.score
            .cmp(&right.score)
            .then_with(|| item_key(left).cmp(&item_key(right)))
    });
    Ok(())
}

fn item_key(item: &ChartItem) -> (String, String) {
    let category_hash = canonical_hash(&item.category);
    let meaning_hash = canonical_hash(&(&item.meaning.expression, &item.meaning.query_variables));
    (category_hash, meaning_hash)
}

pub(crate) fn compose(
    left: &ChartItem,
    right: &ChartItem,
    catalog: &ConceptCatalog,
    max_semantic_nodes: usize,
) -> Result<Option<ChartItem>, ParseError> {
    if let Some((category, semantic_parameter)) =
        apply_function(&left.category, &right.category, catalog, true)
    {
        let meaning = apply_meaning(
            &left.meaning,
            &semantic_parameter,
            &right.meaning,
            max_semantic_nodes,
        )?;
        return Ok(Some(ChartItem {
            start: left.start,
            end: right.end,
            category,
            meaning,
            score: left.score + right.score + 1,
            derivation: DerivationNode::Applied {
                left: Box::new(left.derivation.clone()),
                right: Box::new(right.derivation.clone()),
            },
        }));
    }

    if let Some((category, semantic_parameter)) =
        apply_function(&right.category, &left.category, catalog, false)
    {
        let meaning = apply_meaning(
            &right.meaning,
            &semantic_parameter,
            &left.meaning,
            max_semantic_nodes,
        )?;
        return Ok(Some(ChartItem {
            start: left.start,
            end: right.end,
            category,
            meaning,
            score: left.score + right.score + 1,
            derivation: DerivationNode::Applied {
                left: Box::new(left.derivation.clone()),
                right: Box::new(right.derivation.clone()),
            },
        }));
    }

    Ok(None)
}

fn apply_function(
    function: &SyntacticCategory,
    argument: &SyntacticCategory,
    catalog: &ConceptCatalog,
    left_to_right: bool,
) -> Option<(SyntacticCategory, lexflex_model::ParameterId)> {
    let SyntacticCategory::Function {
        result,
        argument: expected_argument,
        direction,
        semantic_parameter,
        ..
    } = function
    else {
        return None;
    };
    let direction_ok = matches!(
        (direction, left_to_right),
        (SlashDirection::Forward, true) | (SlashDirection::Backward, false)
    );
    let mut substitutions = BTreeMap::new();
    if direction_ok && category_matches(expected_argument, argument, catalog, &mut substitutions) {
        Some((
            apply_substitutions(result, &substitutions),
            semantic_parameter.clone(),
        ))
    } else {
        None
    }
}

fn category_matches(
    expected: &SyntacticCategory,
    actual: &SyntacticCategory,
    catalog: &ConceptCatalog,
    substitutions: &mut BTreeMap<CategoryTypeVariableId, lexflex_model::SemanticType>,
) -> bool {
    match (expected, actual) {
        (
            SyntacticCategory::Atom {
                kind: expected_kind,
                semantic_type: expected_type,
                features: expected_features,
            },
            SyntacticCategory::Atom {
                kind: actual_kind,
                semantic_type: actual_type,
                features: actual_features,
            },
        ) => {
            expected_kind == actual_kind
                && category_type_matches(expected_type, actual_type, catalog, substitutions)
                && actual_features.contains_all(expected_features)
        }
        (
            SyntacticCategory::Function {
                result: expected_result,
                argument: expected_argument,
                direction: expected_direction,
                semantic_parameter: expected_parameter,
                features: expected_features,
            },
            SyntacticCategory::Function {
                result: actual_result,
                argument: actual_argument,
                direction: actual_direction,
                semantic_parameter: actual_parameter,
                features: actual_features,
            },
        ) => {
            expected_direction == actual_direction
                && expected_parameter == actual_parameter
                && category_matches(expected_result, actual_result, catalog, substitutions)
                && category_matches(expected_argument, actual_argument, catalog, substitutions)
                && actual_features.contains_all(expected_features)
        }
        _ => false,
    }
}

fn category_type_matches(
    expected: &CategoryType,
    actual: &CategoryType,
    catalog: &ConceptCatalog,
    substitutions: &mut BTreeMap<CategoryTypeVariableId, lexflex_model::SemanticType>,
) -> bool {
    match (expected, actual) {
        (CategoryType::Concrete(expected), CategoryType::Concrete(actual)) => {
            TypeRelation::new(catalog).accepts(expected, actual)
        }
        (CategoryType::Variable(variable), CategoryType::Concrete(actual)) => {
            match substitutions.get(variable) {
                Some(existing) => TypeRelation::new(catalog).equivalent(existing, actual),
                None => {
                    substitutions.insert(variable.clone(), actual.clone());
                    true
                }
            }
        }
        (CategoryType::Concrete(expected), CategoryType::Variable(variable)) => {
            match substitutions.get(variable) {
                Some(existing) => TypeRelation::new(catalog).equivalent(expected, existing),
                None => {
                    substitutions.insert(variable.clone(), expected.clone());
                    true
                }
            }
        }
        (CategoryType::Variable(left), CategoryType::Variable(right)) => left == right,
    }
}

fn apply_substitutions(
    category: &SyntacticCategory,
    substitutions: &BTreeMap<CategoryTypeVariableId, lexflex_model::SemanticType>,
) -> SyntacticCategory {
    category.map_types(&mut |category_type| match category_type {
        CategoryType::Concrete(value) => CategoryType::Concrete(value.clone()),
        CategoryType::Variable(variable) => substitutions
            .get(variable)
            .cloned()
            .map(CategoryType::Concrete)
            .unwrap_or_else(|| CategoryType::Variable(variable.clone())),
    })
}
