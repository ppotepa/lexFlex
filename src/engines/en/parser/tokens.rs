use super::*;

impl EnglishParser {
    pub(super) fn tokenize(&self, input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut offset = 0;

        for word in input.split_whitespace() {
            let clean = word.trim_matches(|c: char| c.is_ascii_punctuation());
            let form_lower = clean.to_lowercase();
            let (pos, features, lemma) = self.analyze_token(&form_lower);

            tokens.push(Token {
                form: clean.to_string(),
                lemma: Some(lemma),
                pos,
                features,
                span: (offset, offset + word.len()),
                word_node_id: None,
            });

            offset += word.len() + 1;
        }

        tokens
    }

    fn analyze_token(&self, form: &str) -> (PartOfSpeech, FeatureBundle, String) {
        match form {
            "with" | "in" | "on" | "at" | "from" | "to" => {
                return (
                    PartOfSpeech::Preposition,
                    FeatureBundle::default(),
                    form.to_string(),
                );
            }
            "and" => {
                return (
                    PartOfSpeech::Conjunction,
                    FeatureBundle::default(),
                    "and".to_string(),
                );
            }
            "not" | "n't" => {
                return (
                    PartOfSpeech::Negation,
                    FeatureBundle::default(),
                    "not".to_string(),
                );
            }
            "a" | "an" | "the" => {
                let def = if form.eq("the") {
                    Definiteness::Definite
                } else {
                    Definiteness::Indefinite
                };
                return (
                    PartOfSpeech::Determiner,
                    FeatureBundle {
                        definiteness: Some(def),
                        ..Default::default()
                    },
                    form.to_string(),
                );
            }
            _ => {}
        }

        if let Some(entry) = self.lexicon.lookup_by_form(form) {
            let pos = self.lexicon.parse_pos(&entry.pos);
            return (pos, entry.features.clone(), entry.lemma.clone());
        }

        if form.ends_with("s") && !form.ends_with("ss") {
            let stem = &form[..form.len() - 1];
            if let Some(entry) = self.lexicon.lookup_by_form(stem) {
                let pos = self.lexicon.parse_pos(&entry.pos);
                return (pos, entry.features.clone(), entry.lemma.clone());
            }
        }

        if form.ends_with("ed") {
            let stem = &form[..form.len() - 2];
            if let Some(entry) = self.lexicon.lookup_by_form(stem) {
                if entry.pos == "Verb" {
                    return (
                        PartOfSpeech::Verb,
                        FeatureBundle {
                            tense: Some(Tense::Past),
                            ..entry.features.clone()
                        },
                        entry.lemma.clone(),
                    );
                }
            }
        }

        (
            PartOfSpeech::Unknown,
            FeatureBundle::default(),
            form.to_string(),
        )
    }
}
