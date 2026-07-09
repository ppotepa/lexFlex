# Intents — Extracting User Goals

**⚠️ FEATURE - v0.2+ (NOT IN MVP v0.1)**

Intent extraction is a feature planned for v0.2+. This document specifies the design for future implementation.

---

## Overview

An **intent** represents what the user wants to achieve through their utterance. While speech acts describe the communicative function, intents describe the underlying goal.

---

## Intent Types

```rust
pub enum Intent {
    /// User wants information
    Inquire {
        about: Entity,
        question_type: QuestionType,
        specificity: Specificity,
    },
    
    /// User wants something done
    Desire {
        what: InterlinguaNode,
        from: Option<Entity>,
        urgency: Urgency,
    },
    
    /// User is providing information
    Inform {
        topic: Entity,
        content: InterlinguaNode,
    },
    
    /// User is expressing emotion
    ExpressEmotion {
        emotion: Emotion,
        target: Option<Entity>,
    },
    
    /// User is performing social act
    SocialAct {
        act: SpeechAct,
    },
    
    /// User is confirming/denying
    Confirm {
        proposition: InterlinguaNode,
        confirmed: bool,
    },
    
    /// User is correcting
    Correct {
        original: InterlinguaNode,
        correction: InterlinguaNode,
    },
    
    /// User is negotiating
    Negotiate {
        topic: Entity,
        position: InterlinguaNode,
    },
}

pub enum Specificity {
    /// Specific answer expected: "What time is it?"
    Specific,
    
    /// Open-ended: "Tell me about Poland"
    OpenEnded,
    
    /// Clarification: "What do you mean?"
    Clarification,
}

pub enum Urgency {
    Immediate,   // "Help! Now!"
    High,        // "I need this today"
    Normal,      // "When you can"
    Low,         // "Someday"
}
```

## Intent Extraction

```rust
pub struct IntentExtractor;

impl IntentExtractor {
    pub fn extract(
        interlingua: &InterlinguaNode,
        speech_act: &SpeechAct,
    ) -> Intent {
        match speech_act {
            SpeechAct::Question { question_type, .. } => {
                Intent::Inquire {
                    about: Self::extract_topic(interlingua),
                    question_type: question_type.clone(),
                    specificity: Self::detect_specificity(interlingua),
                }
            }
            
            SpeechAct::Request { action, urgency } => {
                Intent::Desire {
                    what: action.clone(),
                    from: None,
                    urgency: *urgency,
                }
            }
            
            SpeechAct::Assert { proposition, .. } => {
                Intent::Inform {
                    topic: Self::extract_topic(interlingua),
                    content: proposition.clone(),
                }
            }
            
            SpeechAct::ExpressEmotion { emotion, .. } => {
                Intent::ExpressEmotion {
                    emotion: emotion.clone(),
                    target: Self::extract_target(interlingua),
                }
            }
            
            SpeechAct::Greet | SpeechAct::Farewell | SpeechAct::Thank { .. } | SpeechAct::Apologize { .. } => {
                Intent::SocialAct {
                    act: speech_act.clone(),
                }
            }
            
            _ => Intent::Inform {
                topic: Self::extract_topic(interlingua),
                content: interlingua.clone(),
            },
        }
    }
    
    fn extract_topic(interlingua: &InterlinguaNode) -> Entity {
        match interlingua {
            InterlinguaNode::Natural(utterance) => {
                // Extract topic from discourse
                utterance.discourse.topic
                    .clone()
                    .unwrap_or_else(|| {
                        // Fallback: first entity in first sentence
                        utterance.sentences.first()
                            .and_then(|s| s.frames.first())
                            .and_then(|f| f.get_subject())
                            .unwrap_or(Entity::Unknown)
                    })
            }
            _ => Entity::Unknown,
        }
    }
}
```

## Intent Parameters

Many intents have parameters that need to be extracted:

```rust
pub struct IntentParameters {
    /// Named parameters extracted from the utterance
    pub named: HashMap<String, ParameterValue>,
    
    /// Positional parameters (in order of appearance)
    pub positional: Vec<ParameterValue>,
    
    /// Temporal parameters
    pub temporal: Vec<TemporalReference>,
    
    /// Spatial parameters
    pub spatial: Vec<SpatialReference>,
}

pub enum ParameterValue {
    Entity(Entity),
    Number(f64),
    String(String),
    Boolean(bool),
    Date(Timestamp),
    Duration(Duration),
    Set(Vec<ParameterValue>),
}
```

### Parameter Extraction Examples

```
"Book a flight from Warsaw to London for tomorrow"

Intent: Desire {
    what: BOOK_FLIGHT,
    from: None,
}

Parameters: {
    "origin": Entity(WARSAW),
    "destination": Entity(LONDON),
    "date": TemporalReference::Relative { offset: 1 day, anchor: Now },
}

---

"Set an alarm for 7 AM"

Intent: Desire {
    what: SET_ALARM,
    from: None,
}

Parameters: {
    "time": TemporalReference::Absolute { hour: 7, minute: 0 },
}

---

"What's the weather in Kraków?"

Intent: Inquire {
    about: Entity(WEATHER),
    question_type: Wh { wh_role: Theme },
    specificity: Specific,
}

Parameters: {
    "location": Entity(KRAKOW),
}
```

## Multi-Intent Utterances

A single utterance can contain multiple intents:

```
"I need a coffee and can you tell me the time?"

Intent 1: Desire { what: COFFEE }
Intent 2: Inquire { about: TIME }

The system should handle both:
  1. Acknowledge coffee request
  2. Answer time question
```

```rust
pub struct MultiIntent {
    pub intents: Vec<Intent>,
    pub primary: usize,        // index of primary intent
    pub coordination: CoordinationType,
}

### Multi-Intent Coordination

When a single utterance contains multiple intents, they can be coordinated:

```rust
pub enum IntentCoordinationType {
    And,    // both intents equally important
    Then,   // sequential: first intent, then second
    But,    // contrastive: first intent, but also second
}
```

**Note**: This is different from syntactic `CoordinationType` in [INTERLINGUA.md](./INTERLINGUA.md#coordination), which handles sentence-level coordination (ClausalCoordination, VPCoordination, NPCoordination, Gapping, VPellipsis).
```

## Intent Confidence

Not all intent extractions are equally certain:

```rust
pub struct IntentResult {
    pub intent: Intent,
    pub confidence: f64,          // 0.0 - 1.0
    pub alternatives: Vec<(Intent, f64)>,
    pub parameters: IntentParameters,
}

impl IntentExtractor {
    pub fn extract_with_confidence(
        interlingua: &InterlinguaNode,
        speech_act: &SpeechAct,
    ) -> IntentResult {
        let primary = Self::extract(interlingua, speech_act);
        
        // Calculate confidence based on:
        // - Clarity of speech act
        // - Number of parameters extracted
        // - Discourse context support
        
        let confidence = Self::calculate_confidence(&primary, interlingua);
        
        // Generate alternatives
        let alternatives = Self::generate_alternatives(interlingua, speech_act);
        
        IntentResult {
            intent: primary,
            confidence,
            alternatives,
            parameters: Self::extract_parameters(interlingua),
        }
    }
}
```

## Context-Dependent Intents

The same utterance can have different intents depending on context:

```
Context 1: User asks "What time is it?"
  → Intent: Inquire { about: TIME }
  → Response: "It's 3 PM"

Context 2: User says "What time is it?" while looking at broken clock
  → Intent: Desire { what: FIX_CLOCK }
  → Response: "I'll help you fix it"

Context 3: Parent says "What time is it?" to child playing video games
  → Intent: Command { action: STOP_PLAYING }
  → Response: (child stops playing)
```

```rust
impl IntentExtractor {
    fn resolve_with_context(
        intent: Intent,
        discourse: &Discourse,
        memory: &LongTermMemory,
    ) -> Intent {
        // Check if current topic suggests different interpretation
        if let Some(topic) = discourse.topic() {
            match (&intent, topic.concept) {
                (Intent::Inquire { about, .. }, TIME_CONCEPT) 
                    if discourse.has_entity(BROKEN_CLOCK) => {
                    // Reinterpret as desire to fix
                    return Intent::Desire {
                        what: InterlinguaNode::Application {
                            function: Box::new(InterlinguaNode::Variable("FIX")),
                            args: vec![InterlinguaNode::Variable("clock")],
                        },
                        from: None,
                        urgency: Urgency::Normal,
                    };
                }
                _ => {}
            }
        }
        
        intent
    }
}
```

## Integration with Response Planning

Intents drive response planning:

```rust
impl ResponsePlanner {
    pub fn plan_for_intent(
        intent: &Intent,
        discourse: &Discourse,
        memory: &LongTermMemory,
    ) -> ResponsePlan {
        match intent {
            Intent::Inquire { about, question_type, .. } => {
                let answer = Self::find_answer(about, question_type, memory);
                
                ResponsePlan {
                    actions: vec![
                        ResponseAction::Answer { content: answer },
                    ],
                    tone: Register::Neutral,
                }
            }
            
            Intent::Desire { what, .. } => {
                if Self::can_fulfill(what) {
                    ResponsePlan {
                        actions: vec![
                            ResponseAction::Acknowledge,
                            ResponseAction::PerformAction { action: what.clone() },
                        ],
                        tone: Register::Neutral,
                    }
                } else {
                    ResponsePlan {
                        actions: vec![
                            ResponseAction::Decline { reason: "cannot fulfill" },
                        ],
                        tone: Register::Neutral,
                    }
                }
            }
            
            Intent::Inform { topic, content } => {
                // Store in memory
                ResponsePlan {
                    actions: vec![
                        ResponseAction::Acknowledge { message: "I see" },
                    ],
                    tone: Register::Neutral,
                }
            }
            
            Intent::SocialAct { act } => {
                match act {
                    SpeechAct::Greet => ResponsePlan::greet_back(),
                    SpeechAct::Thank { .. } => ResponsePlan::youre_welcome(),
                    SpeechAct::Farewell => ResponsePlan::farewell(),
                    _ => ResponsePlan::default(),
                }
            }
            
            _ => ResponsePlan::default(),
        }
    }
}
```

## Summary

Intent extraction:

1. **Identifies user goals** — what the user wants to achieve
2. **Extracts parameters** — entities, temporal, spatial references
3. **Handles multi-intent** — single utterance, multiple goals
4. **Context-dependent** — same words, different intent based on context
5. **Confidence scoring** — not all extractions are equally certain
6. **Drives response planning** — different intents require different responses
