const POLISH_MARKERS: &[&str] = &[
    "ą", "ć", "ę", "ł", "ń", "ó", "ś", "ź", "ż", "czy", "co", "czym", "gdzie",
    "jest", "jaka", "jakie", "kto", "ma", "to",
];

const ENGLISH_MARKERS: &[&str] = &[
    "a", "an", "are", "does", "has", "how", "is", "the", "what", "where", "who",
];

pub fn detect_language(text: &str, fallback: &str) -> String {
    let lower = text.to_lowercase();
    let tokens = lower
        .split(|ch: char| !ch.is_alphabetic())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    let polish = language_score(&lower, &tokens, POLISH_MARKERS);
    let english = language_score(&lower, &tokens, ENGLISH_MARKERS);
    match polish.cmp(&english) {
        std::cmp::Ordering::Greater => "pl".to_string(),
        std::cmp::Ordering::Less => "en".to_string(),
        std::cmp::Ordering::Equal if fallback.eq_ignore_ascii_case("en") => "en".to_string(),
        std::cmp::Ordering::Equal => "pl".to_string(),
    }
}

pub fn is_polish(language: &str) -> bool {
    language.eq_ignore_ascii_case("pl")
}

fn language_score(text: &str, tokens: &[&str], markers: &[&str]) -> usize {
    markers
        .iter()
        .map(|marker| {
            if marker.chars().count() == 1 && !marker.is_ascii() {
                text.matches(marker).count() * 3
            } else {
                tokens.iter().filter(|token| **token == *marker).count()
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::detect_language;

    #[test]
    fn detects_language_per_message() {
        assert_eq!(detect_language("What is Paris?", "pl"), "en");
        assert_eq!(detect_language("Co to jest Paryż?", "en"), "pl");
        assert_eq!(detect_language("Paris", "en"), "en");
    }
}
