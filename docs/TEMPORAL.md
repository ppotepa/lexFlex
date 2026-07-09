# Temporal Reasoning — Time References and Event Ordering

Temporal reasoning allows lexFlex to understand and generate time references across languages, resolving deictic expressions and maintaining event chronology.

---

## Temporal Reference Types

```rust
pub enum TemporalReference {
    /// Absolute time: "2024-01-15", "Monday", "w styczniu"
    Absolute {
        timestamp: Timestamp,
    },
    
    /// Relative to anchor: "yesterday", "in 3 days", "za godzinę"
    Relative {
        offset: Duration,
        anchor: TemporalAnchor,
    },
    
    /// Deictic (context-dependent): "now", "then", "teraz", "wtedy"
    Deictic {
        word: String,
        resolved: Option<Timestamp>,
    },
    
    /// Duration: "for 2 hours", "przez tydzień"
    Duration {
        length: Duration,
    },
    
    /// Frequency: "every day", "twice a week", "codziennie"
    Frequency {
        count: u32,
        period: Duration,
    },
    
    /// Sequence: "before X", "after Y", "przed obiadem"
    Sequence {
        relation: TemporalRelation,
        reference: Box<TemporalReference>,
    },
    
    /// Interval: "from Monday to Friday", "od 9 do 17"
    Interval {
        start: Box<TemporalReference>,
        end: Box<TemporalReference>,
    },
}
```

## Temporal Anchor

The reference point for relative time expressions.

```rust
pub enum TemporalAnchor {
    /// Relative to current moment (when utterance is processed)
    Now,
    
    /// Relative to speech time (when utterance was produced)
    /// Important for recorded messages, letters, etc.
    SpeechTime,
    
    /// Relative to a specific event in discourse
    Event {
        entity_id: EntityId,
        description: String,
    },
    
    /// Relative to another temporal reference
    Temporal(Box<TemporalReference>),
}
```

## Temporal Relations

```rust
pub enum TemporalRelation {
    /// X happens before Y
    Before,
    
    /// X happens after Y
    After,
    
    /// X happens during Y
    During,
    
    /// X and Y happen at the same time
    Simultaneous,
    
    /// X happens until Y
    Until,
    
    /// X happens since Y
    Since,
}
```

## Timestamp and Duration

```rust
pub struct Timestamp {
    pub year: i32,
    pub month: u8,      // 1-12
    pub day: u8,        // 1-31
    pub hour: Option<u8>,    // 0-23
    pub minute: Option<u8>,  // 0-59
    pub second: Option<u8>,  // 0-59
    pub timezone: Option<String>,
}

pub struct Duration {
    pub years: u32,
    pub months: u32,
    pub days: u32,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

impl Duration {
    pub fn days(n: u32) -> Self {
        Duration { days: n, ..Default::default() }
    }
    
    pub fn hours(n: u32) -> Self {
        Duration { hours: n, ..Default::default() }
    }
    
    pub fn weeks(n: u32) -> Self {
        Duration { days: n * 7, ..Default::default() }
    }
}
```

## Temporal Resolver

Resolves temporal references to absolute timestamps.

```rust
pub struct TemporalResolver {
    /// Current time (when processing occurs)
    pub current_time: Timestamp,
    
    /// Speech time (when utterance was produced)
    pub speech_time: Option<Timestamp>,
    
    /// Known events and their times
    pub event_times: HashMap<EntityId, Timestamp>,
}

impl TemporalResolver {
    pub fn new(current_time: Timestamp) -> Self {
        TemporalResolver {
            current_time,
            speech_time: None,
            event_times: HashMap::new(),
        }
    }
    
    /// Resolve temporal reference to absolute timestamp
    pub fn resolve(&self, temporal: &TemporalReference) -> Option<Timestamp> {
        match temporal {
            TemporalReference::Absolute { timestamp } => {
                Some(timestamp.clone())
            }
            
            TemporalReference::Relative { offset, anchor } => {
                let anchor_time = self.resolve_anchor(anchor)?;
                Some(self.add_duration(anchor_time, offset))
            }
            
            TemporalReference::Deictic { word, resolved } => {
                if let Some(ts) = resolved {
                    return Some(ts.clone());
                }
                
                // Try to resolve common deictic words
                match word.as_str() {
                    // Polish
                    "teraz" | "obecnie" => Some(self.current_time.clone()),
                    "wczoraj" => Some(self.subtract_days(self.current_time.clone(), 1)),
                    "jutro" => Some(self.add_days(self.current_time.clone(), 1)),
                    "przedwczoraj" => Some(self.subtract_days(self.current_time.clone(), 2)),
                    "pojutrze" => Some(self.add_days(self.current_time.clone(), 2)),
                    "dziś" | "dzisiaj" => Some(self.start_of_day(self.current_time.clone())),
                    
                    // English
                    "now" | "currently" => Some(self.current_time.clone()),
                    "yesterday" => Some(self.subtract_days(self.current_time.clone(), 1)),
                    "tomorrow" => Some(self.add_days(self.current_time.clone(), 1)),
                    "today" => Some(self.start_of_day(self.current_time.clone())),
                    
                    _ => None,
                }
            }
            
            TemporalReference::Sequence { relation, reference } => {
                let ref_time = self.resolve(reference)?;
                
                match relation {
                    TemporalRelation::Before => {
                        // "before X" — we don't know exactly when, just before
                        None
                    }
                    TemporalRelation::After => {
                        None
                    }
                    _ => None,
                }
            }
            
            _ => None,
        }
    }
    
    fn resolve_anchor(&self, anchor: &TemporalAnchor) -> Option<Timestamp> {
        match anchor {
            TemporalAnchor::Now => Some(self.current_time.clone()),
            
            TemporalAnchor::SpeechTime => {
                self.speech_time.clone().or_else(|| Some(self.current_time.clone()))
            }
            
            TemporalAnchor::Event { entity_id, .. } => {
                self.event_times.get(entity_id).cloned()
            }
            
            TemporalAnchor::Temporal(temporal) => {
                self.resolve(temporal)
            }
        }
    }
    
    fn add_duration(&self, base: Timestamp, duration: &Duration) -> Timestamp {
        // Implementation: add duration to timestamp
        // Handle month/year overflow, leap years, etc.
        // This is a simplified version
        let mut result = base;
        
        result.day += duration.days as u8;
        result.hour = result.hour.map(|h| h + duration.hours as u8);
        result.minute = result.minute.map(|m| m + duration.minutes as u8);
        
        // Normalize (handle overflow)
        self.normalize_timestamp(result)
    }
    
    fn subtract_days(&self, base: Timestamp, days: u32) -> Timestamp {
        let mut result = base;
        result.day = result.day.saturating_sub(days as u8);
        
        // Handle underflow (borrow from month)
        if result.day == 0 {
            result.month = result.month.saturating_sub(1);
            result.day = 28; // Simplified
        }
        
        result
    }
    
    fn add_days(&self, base: Timestamp, days: u32) -> Timestamp {
        self.add_duration(base, &Duration::days(days))
    }
    
    fn start_of_day(&self, ts: Timestamp) -> Timestamp {
        Timestamp {
            hour: Some(0),
            minute: Some(0),
            second: Some(0),
            ..ts
        }
    }
    
    fn normalize_timestamp(&self, ts: Timestamp) -> Timestamp {
        // Handle overflow in hours, minutes, seconds
        // Handle month/year boundaries
        // This is a placeholder for proper date arithmetic
        ts
    }
}
```

## Temporal in Sentences

See [INTERLINGUA.md](./INTERLINGUA.md#sentence) for the canonical Sentence definition, which includes temporal fields:
- `temporal: Option<TemporalReference>` - explicit temporal reference from adverbs
- `event_time: Option<Timestamp>` - when the action occurred
- `reference_time: Option<Timestamp>` - time frame for the event
- `tense: Tense` - grammatical time marking
- `aspect: Aspect` - how the action unfolds over time

## Temporal Adverbs by Language

### Polish Temporal Adverbs

```rust
pub enum PolishTemporalAdverb {
    // Deictic
    Teraz,           // now
    Wczoraj,         // yesterday
    Jutro,           // tomorrow
    Przedwczoraj,    // day before yesterday
    Pojutrze,        // day after tomorrow
    Dzisiaj,         // today
    Dawniej,         // in the past
    Kiedyś,          // someday
    
    // Relative
    Za(WithDuration),      // "za godzinę" (in an hour)
    Temu(WithDuration),    // "tydzień temu" (a week ago)
    W(DateTime),           // "w poniedziałek" (on Monday)
    Od(DateTime),          // "od wczoraj" (since yesterday)
    Do(DateTime),          // "do jutra" (until tomorrow)
    
    // Frequency
    Codziennie,      // every day
    CoTydzień,       // every week
    CoMiesiąc,       // every month
    RazNaDzień,      // once a day
    DwaRazyWTygodniu, // twice a week
    
    // Duration
    Przez(Duration), // "przez godzinę" (for an hour)
    
    // Sequence
    Przed(Event),    // "przed obiadem" (before dinner)
    Po(Event),       // "po pracy" (after work)
    Podczas(Event),   // "podczas spotkania" (during the meeting)
}
```

### English Temporal Adverbs

```rust
pub enum EnglishTemporalAdverb {
    // Deictic
    Now,
    Yesterday,
    Tomorrow,
    Today,
    Currently,
    Recently,
    
    // Relative
    In(WithDuration),      // "in an hour"
    Ago(WithDuration),     // "a week ago"
    On(DateTime),          // "on Monday"
    Since(DateTime),       // "since yesterday"
    Until(DateTime),       // "until tomorrow"
    
    // Frequency
    Daily,
    Weekly,
    Monthly,
    OnceADay,
    TwiceAWeek,
    EveryDay,
    
    // Duration
    For(Duration),         // "for an hour"
    
    // Sequence
    Before(Event),
    After(Event),
    During(Event),
}
```

## Temporal Parsing Examples

```
Input: "wczoraj"
→ TemporalReference::Deictic {
    word: "wczoraj",
    resolved: Some(Timestamp { year: 2024, month: 1, day: 14, ... })
}

Input: "za godzinę"
→ TemporalReference::Relative {
    offset: Duration { hours: 1, .. },
    anchor: TemporalAnchor::Now,
}

Input: "w 2020 roku"
→ TemporalReference::Absolute {
    timestamp: Timestamp { year: 2020, month: 1, day: 1, ... }
}

Input: "przed obiadem"
→ TemporalReference::Sequence {
    relation: TemporalRelation::Before,
    reference: Box::new(TemporalReference::Event("obiad")),
}

Input: "codziennie"
→ TemporalReference::Frequency {
    count: 1,
    period: Duration { days: 1, .. }
}

Input: "od poniedziałku do piątku"
→ TemporalReference::Interval {
    start: Box::new(TemporalReference::Absolute { ... }),
    end: Box::new(TemporalReference::Absolute { ... }),
}
```

## Temporal Generation Examples

```
Interlingua: TemporalReference::Deictic { word: "yesterday" }
→ PL: "wczoraj"
→ EN: "yesterday"

Interlingua: TemporalReference::Relative { offset: Duration(hours: 2), anchor: Now }
→ PL: "za dwie godziny"
→ EN: "in two hours"

Interlingua: TemporalReference::Frequency { count: 1, period: Duration(days: 1) }
→ PL: "codziennie"
→ EN: "every day" / "daily"
```

## Event Time vs Reference Time vs Speech Time

Reichenbach's model of tense:

```rust
pub struct TemporalModel {
    /// S: Speech time — when the utterance is produced
    pub speech_time: Timestamp,
    
    /// E: Event time — when the event occurred
    pub event_time: Timestamp,
    
    /// R: Reference time — the time frame being discussed
    pub reference_time: Timestamp,
}
```

### Tense as Temporal Relations

```
Past tense (PL: "dał", EN: "gave"):
  E < R < S
  Event before Reference before Speech
  
  "Tomek dał jabłko" — event (giving) happened before now

Present tense (PL: "daje", EN: "gives"):
  E = R = S
  Event at Reference at Speech
  
  "Tomek daje jabłko" — event (giving) happening now

Future tense (PL: "da", EN: "will give"):
  S < R = E
  Speech before Reference at Event
  
  "Tomek da jabłko" — event (giving) will happen after now

Present perfect (EN: "has given"):
  E < R = S
  Event before Reference at Speech
  
  "Tomek has given" — event happened before now, but relevant now

Past perfect (PL: "dał był", EN: "had given"):
  E < R < S
  Event before Reference before Speech
  
  "Tomek had given" — event happened before some past reference point
```

## Temporal Ambiguity

Some temporal expressions are ambiguous and require context:

```
"Spotkamy się jutro"
→ "jutro" = tomorrow (clear)

"Spotkamy się wtedy"
→ "wtedy" = then (ambiguous — when?)
→ Requires context: "Kiedy?" "W piątek." → wtedy = Friday

"Zrobię to później"
→ "później" = later (ambiguous — how much later?)
→ Could mean: in 5 minutes, tomorrow, next week
→ Requires context or clarification
```

## Summary

Temporal reasoning in lexFlex:

1. **Captures all temporal information** — absolute, relative, deictic, duration, frequency, sequence
2. **Resolves deictic expressions** — "yesterday" → specific date
3. **Maintains event chronology** — tracks when things happened
4. **Handles cross-language differences** — different languages express time differently
5. **Models tense semantically** — using Reichenbach's S/R/E model
6. **Detects ambiguity** — identifies when temporal references are unclear

Temporal information is stored in `Sentence.temporal` and resolved by `TemporalResolver` during parsing.
