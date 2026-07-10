use crate::core::interlingua::*;
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
        if lemma.ends_with('o') || lemma.ends_with('e') {
            return self.noun_paradigms.iter().find(|p| p.name == "neuter_o");
        }
        if lemma.ends_with('a') {
            return self.noun_paradigms.iter().find(|p| p.name == "feminine_a");
        }
        match gender {
            Some(Gender::Masculine) | Some(Gender::MasculinePersonal)
            | Some(Gender::MasculineAnimate) | Some(Gender::MasculineInanimate) => {
                self.noun_paradigms
                    .iter()
                    .find(|p| p.name == "masculine_consonant")
            }
            Some(Gender::Feminine) => {
                self.noun_paradigms.iter().find(|p| p.name == "feminine_a")
            }
            Some(Gender::Neuter) => {
                self.noun_paradigms.iter().find(|p| p.name == "neuter_o")
            }
            _ => self.noun_paradigms.first(),
        }
    }

    fn select_verb_paradigm(&self, lemma: &str) -> Option<&MorphParadigm> {
        if lemma.ends_with("ać") {
            return self.verb_paradigms.iter().find(|p| p.name == "verb_ac");
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
        if lemma == "dać" {
            return self.verb_paradigms.iter().find(|p| p.name == "verb_dac");
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
}
