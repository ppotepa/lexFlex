use crate::core::interlingua::*;
use crate::data::lexicon::Lexicon;
use crate::data::morphology::{self, MorphParadigm};
use crate::error::GenerateError;

#[derive(Clone)]
pub struct PolishMorphology {
    noun_paradigms: Vec<MorphParadigm>,
    verb_paradigms: Vec<MorphParadigm>,
    adj_paradigms: Vec<MorphParadigm>,
}

impl PolishMorphology {
    pub fn new(
        noun_paradigms: Vec<MorphParadigm>,
        verb_paradigms: Vec<MorphParadigm>,
        adj_paradigms: Vec<MorphParadigm>,
    ) -> Self {
        Self {
            noun_paradigms,
            verb_paradigms,
            adj_paradigms,
        }
    }

    pub fn inflect_noun(
        &self,
        lemma: &str,
        case: Case,
        number: Number,
        gender: Option<Gender>,
        paradigm_name: Option<&str>,
    ) -> Result<String, GenerateError> {
        let paradigm = if let Some(name) = paradigm_name {
            self.noun_paradigms.iter().find(|p| p.name == name)
        } else {
            self.select_noun_paradigm(lemma, gender)
        };

        let paradigm = paradigm.ok_or_else(|| GenerateError::InflectionFailed {
            lemma: lemma.to_string(),
            reason: "No matching noun paradigm".to_string(),
        })?;

        let features = FeatureBundle {
            case: Some(case),
            number: Some(number),
            gender,
            ..Default::default()
        };

        morphology::apply_rules(lemma, &paradigm.rules, &features).ok_or_else(|| {
            GenerateError::InflectionFailed {
                lemma: lemma.to_string(),
                reason: format!("No rule for {:?} {:?}", case, number),
            }
        })
    }

    pub fn inflect_verb(
        &self,
        lemma: &str,
        tense: Tense,
        aspect: Option<Aspect>,
        person: Option<Person>,
        number: Option<Number>,
        gender: Option<Gender>,
        paradigm_name: Option<&str>,
    ) -> Result<String, GenerateError> {
        let paradigm = if let Some(name) = paradigm_name {
            self.verb_paradigms.iter().find(|p| p.name == name)
        } else {
            self.select_verb_paradigm(lemma)
        };

        let paradigm = paradigm.ok_or_else(|| GenerateError::InflectionFailed {
            lemma: lemma.to_string(),
            reason: "No matching verb paradigm".to_string(),
        })?;

        let features = FeatureBundle {
            tense: Some(tense),
            aspect,
            person,
            number,
            gender,
            ..Default::default()
        };

        morphology::apply_rules(lemma, &paradigm.rules, &features).ok_or_else(|| {
            GenerateError::InflectionFailed {
                lemma: lemma.to_string(),
                reason: format!("No rule for {:?} {:?} {:?} {:?}", tense, aspect, person, number),
            }
        })
    }

    pub fn inflect_adjective(
        &self,
        lemma: &str,
        case: Case,
        number: Number,
        gender: Gender,
        degree: Option<crate::core::interlingua::Degree>,
    ) -> Result<String, GenerateError> {
        let paradigm = self
            .adj_paradigms
            .iter()
            .find(|p| p.name == "adj_y" || p.name == "adj_standard");

        let paradigm = paradigm.ok_or_else(|| GenerateError::InflectionFailed {
            lemma: lemma.to_string(),
            reason: "No matching adjective paradigm".to_string(),
        })?;

        // Supplet from lexicon feature if present (data-driven, RON for regular).
        let mut stem = lemma.to_string();
        let mut use_degree_in_features = degree;
        if let Some(d) = degree {
            // For supplet cases, RON or lexicon feature provides stem (see data extend); here keep simple for regular + naj prefix.
            if d == crate::core::interlingua::Degree::Superlative {
                stem = format!("naj{}", stem);
            }
        }

        let features = FeatureBundle {
            case: Some(case),
            number: Some(number),
            gender: Some(gender),
            degree: use_degree_in_features,
            ..Default::default()
        };

        let mut form = morphology::apply_rules(&stem, &paradigm.rules, &features)
            .unwrap_or_else(|| stem.clone());

        if form == stem && degree.is_some() && stem == lemma {
            form = stem;
        }

        Ok(form)
    }

    fn select_noun_paradigm(
        &self,
        lemma: &str,
        gender: Option<Gender>,
    ) -> Option<&MorphParadigm> {
        // Algorithmic paradigm selection based on suffix pattern + gender.
        // More specific patterns first.
        if lemma.ends_with("um") {
            return self.noun_paradigms.iter().find(|p| p.name == "neuter_um");
        }
        if lemma.ends_with("ec") {
            return self.noun_paradigms.iter().find(|p| p.name == "masculine_ec");
        }
        if lemma.ends_with("ia") {
            return self.noun_paradigms.iter().find(|p| p.name == "feminine_ia");
        }
        if lemma.ends_with('o') {
            return self.noun_paradigms.iter().find(|p| p.name == "neuter_o");
        }
        if lemma.ends_with('e') {
            return self.noun_paradigms.iter().find(|p| p.name == "neuter_e");
        }
        if lemma.ends_with('a') {
            return self.noun_paradigms.iter().find(|p| p.name == "feminine_a");
        }
        // Consonant-stem: check gender for personal vs common
        match gender {
            Some(Gender::MasculinePersonal) => {
                self.noun_paradigms.iter().find(|p| p.name == "masculine_personal")
            }
            Some(Gender::Feminine) => {
                self.noun_paradigms.iter().find(|p| p.name == "feminine_a")
            }
            Some(Gender::Neuter) => {
                self.noun_paradigms.iter().find(|p| p.name == "neuter_o")
            }
            _ => self.noun_paradigms.iter().find(|p| p.name == "masculine_consonant"),
        }
    }

    fn select_verb_paradigm(&self, lemma: &str) -> Option<&MorphParadigm> {
        // Algorithmic paradigm selection based on suffix pattern of lemma.
        // Order matters: more specific patterns first.
        if lemma.ends_with("ować") {
            return self.verb_paradigms.iter().find(|p| p.name == "verb_ować");
        }
        if lemma.ends_with("ywać") || lemma.ends_with("awać") {
            return self.verb_paradigms.iter().find(|p| p.name == "verb_wać");
        }
        if lemma == "mieć" {
            return self.verb_paradigms.iter().find(|p| p.name == "verb_miec");
        }
        if lemma == "być" {
            return self.verb_paradigms.iter().find(|p| p.name == "verb_byc");
        }
        if lemma == "dać" {
            return self.verb_paradigms.iter().find(|p| p.name == "verb_dac");
        }
        if lemma.ends_with("eć") {
            return self.verb_paradigms.iter().find(|p| p.name == "verb_ec");
        }
        if lemma.ends_with("ść") {
            return self.verb_paradigms.iter().find(|p| p.name == "verb_sc");
        }
        if lemma.ends_with("ić") {
            return self.verb_paradigms.iter().find(|p| p.name == "verb_ic");
        }
        if lemma.ends_with("ać") {
            return self.verb_paradigms.iter().find(|p| p.name == "verb_ac");
        }
        self.verb_paradigms.first()
    }

    /// Use RON-loaded paradigms (via data::morphology rules) to analyze inflected verb form for parser.
    /// Returns features (tense etc) derived from matching rule conditions.
    pub fn analyze_verb_form(&self, form: &str) -> Option<FeatureBundle> {
        crate::data::morphology::analyze_via_paradigms(form, &self.verb_paradigms)
    }

    /// Bidirectional analyzer for adjectives using RON (incl DegreeIs) + reverse.
    /// Used to recover base lemma + Degree from surface comp/super like "lepszy" / "większy".
    pub fn analyze_adjective_form(&self, form: &str) -> Option<FeatureBundle> {
        crate::data::morphology::analyze_via_paradigms(form, &self.adj_paradigms)
    }

    /// Access verb paradigms for external reverse matching.
    pub fn verb_paradigms(&self) -> &[MorphParadigm] {
        &self.verb_paradigms
    }

    /// Access noun paradigms for external reverse matching.
    pub fn noun_paradigms(&self) -> &[MorphParadigm] {
        &self.noun_paradigms
    }

    /// Algorithmic verb analysis: try all verb paradigms in reverse to find
    /// (lemma_candidate, features) from a surface form. Then validate the
    /// candidate against the lexicon. Returns None if no lexicon match.
    pub fn analyze_verb_with_lexicon(&self, form: &str, lexicon: &Lexicon) -> Option<(String, FeatureBundle)> {
        for paradigm in &self.verb_paradigms {
            for rule in &paradigm.rules {
                if let Some(stem) = crate::data::morphology::reverse_to_stem(form, &rule.operations) {
                    // stem is the lemma candidate — check if it exists in lexicon
                    if let Some(entry) = lexicon.lookup_by_lemma(&stem) {
                        if entry.pos == "Verb" {
                            let mut fb = entry.features.clone();
                            // Enrich with features from rule conditions
                            for cond in &rule.conditions {
                                match cond {
                                    crate::data::morphology::Condition::TenseIs(t) => fb.tense = Some(*t),
                                    crate::data::morphology::Condition::PersonIs(p) => fb.person = Some(*p),
                                    crate::data::morphology::Condition::NumberIs(n) => fb.number = Some(*n),
                                    crate::data::morphology::Condition::GenderIs(g) => fb.gender = Some(*g),
                                    _ => {}
                                }
                            }
                            return Some((stem, fb));
                        }
                    }
                    // Also try form-based lookup (for inflected entries like "ma", "jest")
                    if let Some(entry) = lexicon.lookup_by_form(&stem) {
                        if entry.pos == "Verb" {
                            let mut fb = entry.features.clone();
                            for cond in &rule.conditions {
                                match cond {
                                    crate::data::morphology::Condition::TenseIs(t) => fb.tense = Some(*t),
                                    crate::data::morphology::Condition::PersonIs(p) => fb.person = Some(*p),
                                    crate::data::morphology::Condition::NumberIs(n) => fb.number = Some(*n),
                                    crate::data::morphology::Condition::GenderIs(g) => fb.gender = Some(*g),
                                    _ => {}
                                }
                            }
                            return Some((entry.lemma.clone(), fb));
                        }
                    }
                }
            }
        }
        None
    }

    /// Algorithmic noun analysis: try all noun paradigms in reverse to find
    /// (lemma_candidate, features) from a surface form. Then validate the
    /// candidate against the lexicon. Returns None if no lexicon match.
    pub fn analyze_noun_with_lexicon(&self, form: &str, lexicon: &Lexicon) -> Option<(String, FeatureBundle)> {
        for paradigm in &self.noun_paradigms {
            for rule in &paradigm.rules {
                if let Some(stem) = crate::data::morphology::reverse_to_stem(form, &rule.operations) {
                    // stem is the lemma candidate — check if it exists in lexicon
                    if let Some(entry) = lexicon.lookup_by_lemma(&stem) {
                        if entry.pos == "Noun" {
                            let mut fb = entry.features.clone();
                            // Enrich with features from rule conditions
                            for cond in &rule.conditions {
                                match cond {
                                    crate::data::morphology::Condition::CaseIs(c) => fb.case = Some(*c),
                                    crate::data::morphology::Condition::NumberIs(n) => fb.number = Some(*n),
                                    crate::data::morphology::Condition::GenderIs(g) => fb.gender = Some(*g),
                                    _ => {}
                                }
                            }
                            return Some((stem, fb));
                        }
                    }
                }
            }
        }
        None
    }
}
