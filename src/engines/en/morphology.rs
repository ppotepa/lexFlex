use crate::core::interlingua::*;
use crate::data::morphology::{self, MorphParadigm};
use crate::error::GenerateError;

pub struct EnglishMorphology {
    verb_paradigms: Vec<MorphParadigm>,
    _noun_paradigms: Vec<MorphParadigm>,
}

impl EnglishMorphology {
    pub fn new(
        verb_paradigms: Vec<MorphParadigm>,
        noun_paradigms: Vec<MorphParadigm>,
    ) -> Self {
        Self {
            verb_paradigms,
            _noun_paradigms: noun_paradigms,
        }
    }

    pub fn inflect_verb(
        &self,
        lemma: &str,
        tense: Tense,
        person: Option<Person>,
        number: Option<Number>,
    ) -> Result<String, GenerateError> {
        let irregular = match (lemma, tense, person, number) {
            // "be called" - idiomatic expression for "mieć na imię"
            ("be called", Tense::Present, Some(Person::First), Some(Number::Singular)) => Some("am called"),
            ("be called", Tense::Present, Some(Person::Second), Some(Number::Singular)) => Some("are called"),
            ("be called", Tense::Present, Some(Person::Third), Some(Number::Singular)) => Some("is called"),
            ("be called", Tense::Present, _, Some(Number::Plural)) => Some("are called"),
            ("be called", Tense::Present, _, _) => Some("are called"),
            ("be called", Tense::Past, _, Some(Number::Singular)) => Some("was called"),
            ("be called", Tense::Past, _, Some(Number::Plural)) => Some("were called"),
            ("be called", Tense::Past, _, _) => Some("was called"),
            ("be called", Tense::Future, _, _) => Some("will be called"),
            
            ("be", Tense::Present, Some(Person::First), Some(Number::Singular)) => Some("am"),
            ("be", Tense::Present, Some(Person::Third), Some(Number::Singular)) => Some("is"),
            ("be", Tense::Present, _, Some(Number::Plural)) => Some("are"),
            ("be", Tense::Present, _, _) => Some("are"),
            ("be", Tense::Past, _, Some(Number::Singular)) => Some("was"),
            ("be", Tense::Past, _, Some(Number::Plural)) => Some("were"),
            ("be", Tense::Past, _, _) => Some("was"),
            ("have", Tense::Present, Some(Person::Third), Some(Number::Singular)) => Some("has"),
            ("have", Tense::Present, _, _) => Some("have"),
            ("have", Tense::Past, _, _) => Some("had"),
            ("do", Tense::Present, Some(Person::Third), Some(Number::Singular)) => Some("does"),
            ("do", Tense::Past, _, _) => Some("did"),
            ("give", Tense::Past, _, _) => Some("gave"),
            ("go", Tense::Past, _, _) => Some("went"),
            ("come", Tense::Past, _, _) => Some("came"),
            ("see", Tense::Past, _, _) => Some("saw"),
            ("eat", Tense::Past, _, _) => Some("ate"),
            ("drink", Tense::Past, _, _) => Some("drank"),
            ("say", Tense::Past, _, _) => Some("said"),
            ("make", Tense::Past, _, _) => Some("made"),
            ("take", Tense::Past, _, _) => Some("took"),
            ("know", Tense::Past, _, _) => Some("knew"),
            ("think", Tense::Past, _, _) => Some("thought"),
            ("love", Tense::Past, _, _) => Some("loved"),
            ("read", Tense::Past, _, _) => Some("read"),
            ("write", Tense::Past, _, _) => Some("wrote"),
            ("buy", Tense::Past, _, _) => Some("bought"),
            ("hear", Tense::Past, _, _) => Some("heard"),
            ("break", Tense::Past, _, _) => Some("broke"),
            _ => None,
        };

        if let Some(form) = irregular {
            return Ok(form.to_string());
        }

        let paradigm = self.verb_paradigms.iter().find(|p| p.name == "verb_regular");

        if let Some(p) = paradigm {
            let features = FeatureBundle {
                tense: Some(tense),
                person,
                number,
                ..Default::default()
            };
            if let Some(form) = morphology::apply_rules(lemma, &p.rules, &features) {
                return Ok(form);
            }
        }

        match tense {
            Tense::Past => Ok(format!("{}ed", lemma)),
            Tense::Present => {
                if person == Some(Person::Third) && number == Some(Number::Singular) {
                    Ok(format!("{}s", lemma))
                } else {
                    Ok(lemma.to_string())
                }
            }
            Tense::Future => Ok(lemma.to_string()),
        }
    }

    pub fn inflect_noun(
        &self,
        lemma: &str,
        number: Number,
    ) -> Result<String, GenerateError> {
        match number {
            Number::Singular => Ok(lemma.to_string()),
            Number::Plural => {
                if lemma == "wife" || lemma == "life" || lemma == "knife" || lemma == "wolf" || lemma == "leaf" {
                    // Common *f / *fe -> ves irregulars (data-driven would come from paradigm; algorithmic here)
                    let stem = if lemma.ends_with("fe") { &lemma[..lemma.len()-2] } else { &lemma[..lemma.len()-1] };
                    Ok(format!("{}ves", stem))
                } else if lemma.ends_with('s') || lemma.ends_with("sh") || lemma.ends_with("ch") || lemma.ends_with('x') {
                    Ok(format!("{}es", lemma))
                } else if lemma.ends_with('y') && !lemma.ends_with("ay") && !lemma.ends_with("ey") && !lemma.ends_with("oy") && !lemma.ends_with("uy") {
                    let stem = &lemma[..lemma.len() - 1];
                    Ok(format!("{}ies", stem))
                } else if lemma.ends_with("fe") {
                    let stem = &lemma[..lemma.len()-2];
                    Ok(format!("{}ves", stem))
                } else if lemma.ends_with('f') && !lemma.ends_with("ff") {
                    let stem = &lemma[..lemma.len()-1];
                    Ok(format!("{}ves", stem))
                } else {
                    Ok(format!("{}s", lemma))
                }
            }
            Number::Dual => Ok(lemma.to_string()),
        }
    }
}
