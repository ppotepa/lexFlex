use crate::core::context;
use crate::core::deduction::{self, DeductionContext};
use crate::core::graph::{self, EdgeKind, GraphNode, LinguisticGraph, TrackedEntity};
use crate::core::interlingua::*;
use crate::core::ontology::Ontology;
use crate::data::descriptor::LanguageDescriptor;
use crate::data::lexicon::Lexicon;
use crate::engines::pl::morphology::PolishMorphology;
use crate::error::ParseError;
use crate::core::unknown_concept::{resolve_concept_for_unknown, is_entity_candidate_token};

mod build_partial;
mod build_frame;
mod pp;
mod questions;
mod tokens;

pub struct PolishParser {
    lexicon: Lexicon,
    morphology: PolishMorphology,
    ontology: Ontology,
    #[allow(dead_code)]
    descriptor: LanguageDescriptor,
    concept_ids: Vec<String>,
}

impl PolishParser {
    pub fn new(lexicon: Lexicon, morphology: PolishMorphology, ontology: Ontology, descriptor: LanguageDescriptor, concept_ids: Vec<String>) -> Self {
        Self {
            lexicon,
            morphology,
            ontology,
            descriptor,
            concept_ids,
        }
    }

    pub fn parse(&self, input: &str) -> Result<Utterance, ParseError> {
        if input.trim().is_empty() {
            return Err(ParseError::EmptyInput);
        }

        let sentence_parts = context::split_sentence_boundaries(input);
        if sentence_parts.len() > 1 {
            let mut all_sentences = Vec::new();
            for part in sentence_parts {
                if let Ok(utt) = self.parse_single(&part) {
                    all_sentences.extend(utt.sentences);
                }
            }
            if all_sentences.is_empty() {
                return Err(ParseError::NoVerbFound);
            }
            let mut utterance = Utterance {
                sentences: all_sentences,
                discourse: None,
                utterance_node_id: None,
            };
            context::track_discourse(&mut utterance);
            return Ok(utterance);
        }

        let mut utterance = self.parse_single(input)?;
        context::track_discourse(&mut utterance);
        Ok(utterance)
    }

    fn parse_single(&self, input: &str) -> Result<Utterance, ParseError> {
        let tokens = self.tokenize(input);

        // Detect clause boundaries and split into clause groups
        let clause_groups = self.split_into_clauses(&tokens);

        if clause_groups.len() <= 1 {
            // Single clause — proceed as before
            let partial = self.build_partial_structure(&tokens)?;
            let context = DeductionContext::new(&self.lexicon, &self.ontology, LanguageId::new("pl"));
            let mut utterance = deduction::deduce(partial, &context)
                .map_err(|_| ParseError::NoVerbFound)?;

            // Propagate tense from tokens
            if let Some(s) = utterance.sentences.first_mut() {
                for t in &tokens {
                    if t.features.tense == Some(Tense::Past) {
                        s.tense = Some(Tense::Past);
                        break;
                    }
                }
            }
            return Ok(utterance);
        }

        // Multiple clauses — parse each separately (within one sentence)
        let mut all_sentences = vec![];
        for clause_tokens in &clause_groups {
            if clause_tokens.is_empty() { continue; }
            // Skip if no verb in this clause
            if !clause_tokens.iter().any(|t| t.pos == PartOfSpeech::Verb) { continue; }

            match self.build_partial_structure(clause_tokens) {
                Ok(partial) => {
                    let context = DeductionContext::new(&self.lexicon, &self.ontology, LanguageId::new("pl"));
                    if let Ok(mut utterance) = deduction::deduce(partial, &context) {
                        // Propagate tense
                        if let Some(s) = utterance.sentences.first_mut() {
                            for t in clause_tokens.iter() {
                                if t.features.tense == Some(Tense::Past) {
                                    s.tense = Some(Tense::Past);
                                    break;
                                }
                            }
                        }
                        all_sentences.extend(utterance.sentences);
                    }
                }
                Err(_) => continue,
            }
        }

        if all_sentences.is_empty() {
            return Err(ParseError::NoVerbFound);
        }

        Ok(Utterance {
            sentences: all_sentences,
            discourse: None,
            utterance_node_id: None,
        })
    }

    /// Split tokens into clause groups based on clause boundary conjunctions.
    /// Returns a vec of token vectors, one per clause.
    fn split_into_clauses(&self, tokens: &[Token]) -> Vec<Vec<Token>> {
        let initial_subordinators = ["bo", "ponieważ", "kiedy", "gdy", "jeśli", "jeżeli", "że"];
        if tokens
            .first()
            .map(|t| initial_subordinators.contains(&t.form.to_lowercase().as_str()))
            .unwrap_or(false)
        {
            let verb_positions: Vec<usize> = tokens
                .iter()
                .enumerate()
                .filter_map(|(i, t)| (t.pos == PartOfSpeech::Verb).then_some(i))
                .collect();
            if verb_positions.len() >= 2 {
                let first_verb = verb_positions[0];
                let second_verb = verb_positions[1];
                let split_idx = (first_verb + 1..second_verb)
                    .rev()
                    .find(|&i| is_entity_candidate_token(&tokens[i]))
                    .unwrap_or(second_verb);
                if split_idx > 0 && split_idx < tokens.len() {
                    return vec![tokens[..split_idx].to_vec(), tokens[split_idx..].to_vec()];
                }
            }
        }

        // Conjunctions that always mark clause boundaries
        let always_boundary = ["ale", "a", "że", "bo", "ponieważ", "jeśli", "jeżeli", "gdy", "kiedy", "dlatego"];
        // Conjunctions that mark clause boundaries only when there's a verb on both sides
        let sometimes_boundary = ["i", "oraz", "lub", "albo"];

        let mut boundaries: Vec<usize> = vec![];

        for (i, token) in tokens.iter().enumerate() {
            let form = token.form.to_lowercase();
            if always_boundary.contains(&form.as_str()) {
                boundaries.push(i);
            } else if sometimes_boundary.contains(&form.as_str()) {
                let verb_before = tokens[..i].iter().any(|t| t.pos == PartOfSpeech::Verb);
                let verb_after = tokens[i+1..].iter().any(|t| t.pos == PartOfSpeech::Verb);
                if verb_before && verb_after {
                    boundaries.push(i);
                }
            }
        }

        if boundaries.is_empty() {
            return vec![tokens.to_vec()];
        }

        let mut groups: Vec<Vec<Token>> = vec![];
        let mut start = 0;

        for &boundary in &boundaries {
            if boundary > start {
                groups.push(tokens[start..boundary].to_vec());
            }
            start = boundary + 1;
        }

        if start < tokens.len() {
            groups.push(tokens[start..].to_vec());
        }

        groups
    }

}

fn parse_role_str(s: &str) -> Option<SemanticRole> {
    crate::core::utils::parse_role_str(s)
}

#[cfg(test)]
mod tests;
