use crate::diagnostic::ParseError;
use crate::input::ClauseMode;
use crate::lexical::{candidate_order, score_candidate, Candidate, CategoryFeatureMerge};
use crate::meaning::instantiate_meaning;
use crate::parser::context::ParseContext;
use lexflex_model::canonical_hash;

pub fn lexical_stage(ctx: &mut ParseContext<'_>) -> Result<Vec<Vec<Candidate>>, ParseError> {
    let mut output = Vec::with_capacity(ctx.words.len());
    for token in &ctx.words {
        let form_ids = ctx.language.form_index.lookup(&token.normalized);
        if form_ids.is_empty() {
            return Err(ParseError::UnknownSurface {
                token: token.surface.clone(),
            });
        }
        let mut candidates = Vec::new();
        for form_id in form_ids {
            let Some(form) = ctx.language.form(form_id) else {
                continue;
            };
            let sense_ids = ctx.language.sense_index.lookup(&form.lexeme_id);
            for sense_id in sense_ids {
                let Some(sense) = ctx.language.compiled_sense(sense_id) else {
                    continue;
                };
                let interrogative = lexflex_language::FeatureName::new_unchecked("interrogative");
                let part_of_speech = lexflex_language::FeatureName::new_unchecked("part_of_speech");
                let verb = lexflex_language::FeatureValue::new_unchecked("verb");
                let case_sensitive_interrogative = form.features.get(&interrogative).is_some()
                    && form.features.get(&part_of_speech) == Some(&verb);
                if (sense.category.features().get(&interrogative).is_some()
                    || form.features.get(&interrogative).is_some())
                    && (form.features.get(&interrogative).is_none()
                        || ctx.tokenization.mode != ClauseMode::Interrogative
                        || (case_sensitive_interrogative && form.surface != token.surface))
                {
                    continue;
                }
                let Some(category) = sense.category.unify_features(&form.features) else {
                    continue;
                };
                let seed =
                    canonical_hash(&(token.id.as_str(), form.id.as_str(), sense.id.as_str()))?;
                let seed = seed.as_str().to_owned();
                let (category, meaning) = instantiate_meaning(&seed, &category, sense)?;
                if meaning.semantic_nodes > ctx.budget.max_semantic_nodes {
                    return Err(ParseError::BudgetExceeded(
                        crate::diagnostic::ParseBudgetLimit::SemanticNodeLimit,
                    ));
                }
                candidates.push(Candidate {
                    token: token.clone(),
                    sense: sense.clone(),
                    category,
                    meaning,
                    score: score_candidate(form, sense),
                });
            }
        }
        ctx.metrics.lexical_candidate_count += candidates.len();
        candidates.sort_by(candidate_order);
        if candidates.len() > ctx.budget.max_lexical_candidates_per_token {
            return Err(ParseError::BudgetExceeded(
                crate::diagnostic::ParseBudgetLimit::LexicalCandidateLimit,
            ));
        }
        output.push(candidates);
    }
    Ok(output)
}
