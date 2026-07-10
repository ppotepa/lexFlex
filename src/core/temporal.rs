use crate::core::interlingua::{TemporalAnchor, TemporalReference};

pub fn resolve_deictic(word: &str) -> Option<TemporalReference> {
    match word {
        "wczoraj" | "yesterday" => Some(TemporalReference::Relative {
            offset_days: -1,
            anchor: TemporalAnchor::Now,
        }),
        "dzisiaj" | "today" | "teraz" | "now" => Some(TemporalReference::Relative {
            offset_days: 0,
            anchor: TemporalAnchor::Now,
        }),
        "jutro" | "tomorrow" => Some(TemporalReference::Relative {
            offset_days: 1,
            anchor: TemporalAnchor::Now,
        }),
        "pojutrzut" | "day after tomorrow" => Some(TemporalReference::Relative {
            offset_days: 2,
            anchor: TemporalAnchor::Now,
        }),
        "dzień temu" | "a day ago" => Some(TemporalReference::Relative {
            offset_days: -1,
            anchor: TemporalAnchor::Now,
        }),
        "tydzień temu" | "a week ago" => Some(TemporalReference::Relative {
            offset_days: -7,
            anchor: TemporalAnchor::Now,
        }),
        "za tydzień" | "in a week" => Some(TemporalReference::Relative {
            offset_days: 7,
            anchor: TemporalAnchor::Now,
        }),
        "za miesiąc" | "in a month" => Some(TemporalReference::Relative {
            offset_days: 30,
            anchor: TemporalAnchor::Now,
        }),
        "miesiąc temu" | "a month ago" => Some(TemporalReference::Relative {
            offset_days: -30,
            anchor: TemporalAnchor::Now,
        }),
        _ => None,
    }
}
