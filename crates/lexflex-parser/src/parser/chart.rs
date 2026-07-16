use crate::parser::meaning::{apply_meaning, MeaningInstance};
use crate::token::Token;
use lexflex_language::{AtomicCategoryKind, CategoryType, SlashDirection, SyntacticCategory};
use lexflex_model::canonical_hash;
use lexflex_model::SemanticType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub(super) struct ChartItem {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) category: SyntacticCategory,
    pub(super) meaning: MeaningInstance,
    pub(super) score: i64,
    pub(super) derivation: DerivationNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivationNode {
    Lexical {
        token: Token,
        sense: lexflex_language::LexicalSenseId,
    },
    Applied {
        left: Box<DerivationNode>,
        right: Box<DerivationNode>,
    },
}

pub(super) fn insert_item(
    cell: &mut Vec<ChartItem>,
    item: ChartItem,
    limit: usize,
) -> Result<(), crate::diagnostic::ParseError> {
    let key = item_key(&item);
    if let Some(existing) = cell.iter_mut().find(|existing| item_key(existing) == key) {
        if item.score < existing.score {
            *existing = item;
        }
    } else {
        cell.push(item);
    }
    cell.sort_by(|left, right| {
        left.score
            .cmp(&right.score)
            .then_with(|| item_key(left).cmp(&item_key(right)))
    });
    cell.truncate(limit);
    Ok(())
}

fn item_key(item: &ChartItem) -> (String, String) {
    let category_hash = canonical_hash(&item.category);
    let meaning_hash = canonical_hash(&(
        &item.meaning.expression,
        &item.meaning.query_variables,
    ));
    (category_hash, meaning_hash)
}

pub(super) fn compose(left: &ChartItem, right: &ChartItem) -> Option<ChartItem> {
    if let Some(category) = apply_function(&left.category, &right.category, true) {
        let mut meaning = apply_meaning(&left.meaning, &right.meaning);
        meaning.semantic_type = semantic_type_for_category(&category);
        normalize_query_variable_types(&mut meaning);
        return Some(ChartItem {
            start: left.start,
            end: right.end,
            category,
            meaning,
            score: left.score + right.score + 1,
            derivation: DerivationNode::Applied {
                left: Box::new(left.derivation.clone()),
                right: Box::new(right.derivation.clone()),
            },
        });
    }

    if let Some(category) = apply_function(&right.category, &left.category, false) {
        let mut meaning = apply_meaning(&right.meaning, &left.meaning);
        meaning.semantic_type = semantic_type_for_category(&category);
        normalize_query_variable_types(&mut meaning);
        return Some(ChartItem {
            start: left.start,
            end: right.end,
            category,
            meaning,
            score: left.score + right.score + 1,
            derivation: DerivationNode::Applied {
                left: Box::new(left.derivation.clone()),
                right: Box::new(right.derivation.clone()),
            },
        });
    }

    None
}

fn apply_function(
    function: &SyntacticCategory,
    argument: &SyntacticCategory,
    function_on_left: bool,
) -> Option<SyntacticCategory> {
    let SyntacticCategory::Function {
        result,
        argument: expected_argument,
        direction,
        ..
    } = function
    else {
        return None;
    };
    let allowed = matches!(
        (direction, function_on_left),
        (SlashDirection::Forward, true) | (SlashDirection::Backward, false)
    );
    if allowed && category_compatible(expected_argument, argument) {
        Some((**result).clone())
    } else {
        None
    }
}

fn category_compatible(expected: &SyntacticCategory, actual: &SyntacticCategory) -> bool {
    match (expected, actual) {
        (
            SyntacticCategory::Atom {
                kind: left_kind,
                semantic_type: left_type,
                features: left_features,
            },
            SyntacticCategory::Atom {
                kind: right_kind,
                semantic_type: right_type,
                features: right_features,
            },
        ) => {
            left_kind == right_kind
                && category_type_compatible(left_type, right_type)
                && right_features.contains_all(left_features)
        }
        (
            SyntacticCategory::Function {
                result: left_result,
                argument: left_argument,
                direction: left_direction,
                semantic_parameter: left_parameter,
                features: left_features,
            },
            SyntacticCategory::Function {
                result: right_result,
                argument: right_argument,
                direction: right_direction,
                semantic_parameter: right_parameter,
                features: right_features,
            },
        ) => {
            left_direction == right_direction
                && left_parameter == right_parameter
                && category_compatible(left_result, right_result)
                && category_compatible(left_argument, right_argument)
                && right_features.contains_all(left_features)
        }
        _ => false,
    }
}

fn category_type_compatible(expected: &CategoryType, actual: &CategoryType) -> bool {
    match (expected, actual) {
        (CategoryType::Concrete(left), CategoryType::Concrete(right)) => {
            semantic_type_compatible(left, right)
        }
        (CategoryType::Variable(_), _) | (_, CategoryType::Variable(_)) => true,
    }
}

fn semantic_type_compatible(expected: &SemanticType, actual: &SemanticType) -> bool {
    match (expected, actual) {
        (left, right) if left == right => true,
        (SemanticType::Entity, SemanticType::EntityOf(_)) => true,
        _ => false,
    }
}

fn semantic_type_for_category(category: &SyntacticCategory) -> SemanticType {
    match category {
        SyntacticCategory::Atom {
            kind: AtomicCategoryKind::Sentence,
            ..
        } => SemanticType::Boolean,
        SyntacticCategory::Atom { semantic_type, .. } => match semantic_type {
            CategoryType::Concrete(value) => value.clone(),
            CategoryType::Variable(_) => SemanticType::Entity,
        },
        SyntacticCategory::Function { result, .. } => semantic_type_for_category(result),
    }
}

fn normalize_query_variable_types(meaning: &mut MeaningInstance) {
    for value in meaning.query_variables.values_mut() {
        let _ = value;
    }
}
