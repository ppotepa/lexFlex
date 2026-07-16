use crate::{
    AtomicCategoryKind, CompiledLexicalSense, FeatureStructure, LanguageCompileError, LexicalSense,
    SyntacticCategory, ValencySlot,
};
use std::collections::BTreeSet;

const MAX_COMPILED_CATEGORY_DEPTH: usize = 64;

fn compile_argument_category(
    slot: &ValencySlot,
) -> Result<SyntacticCategory, LanguageCompileError> {
    match (&slot.surface_relation, &slot.argument_category) {
        (None, category) => Ok(category.clone()),
        (
            Some(relation),
            SyntacticCategory::Atom {
                semantic_type,
                features,
                ..
            },
        ) => Ok(SyntacticCategory::Atom {
            kind: AtomicCategoryKind::MarkedArgument {
                relation: relation.clone(),
            },
            semantic_type: semantic_type.clone(),
            features: features.clone(),
        }),
        (Some(_), SyntacticCategory::Function { .. }) => {
            Err(LanguageCompileError::MarkedArgumentRequiresAtom {
                slot_id: slot.id.clone(),
            })
        }
    }
}

pub fn compile_lexical_sense(
    sense: &LexicalSense,
) -> Result<CompiledLexicalSense, LanguageCompileError> {
    let mut valency = sense.valency.clone();
    valency.sort_by(|left, right| {
        left.application_rank
            .cmp(&right.application_rank)
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut seen_ranks = BTreeSet::new();
    for slot in &valency {
        if !seen_ranks.insert(slot.application_rank) {
            return Err(LanguageCompileError::DuplicateValencyRank {
                sense_id: sense.id.clone(),
                rank: slot.application_rank,
            });
        }
    }

    let mut category = sense
        .base_category
        .with_root_features(&sense.features)
        .map_err(|conflict| LanguageCompileError::FeatureConflict {
            sense_id: sense.id.clone(),
            conflict,
        })?;

    for slot in valency.iter().rev() {
        let argument = compile_argument_category(slot)?;
        category = SyntacticCategory::Function {
            result: Box::new(category),
            argument: Box::new(argument),
            direction: slot.direction,
            semantic_parameter: slot.parameter.clone(),
            features: FeatureStructure::default(),
        };
    }

    let depth = category.depth();
    if depth > MAX_COMPILED_CATEGORY_DEPTH {
        return Err(LanguageCompileError::CategoryDepthExceeded {
            sense_id: sense.id.clone(),
            depth,
        });
    }

    Ok(CompiledLexicalSense {
        id: sense.id.clone(),
        lexeme_id: sense.lexeme_id.clone(),
        anchor: sense.anchor.clone(),
        category,
        meaning: sense.meaning.clone(),
        lexical_features: sense.features.clone(),
        priority: sense.priority,
    })
}
