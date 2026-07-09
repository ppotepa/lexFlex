# Speech Acts — Recognizing Communicative Intent

**⚠️ FEATURE - v0.2+ (NOT IN MVP v0.1)**

Speech act recognition is a feature planned for v0.2+. This document specifies the design for future implementation.

---

## Overview

Speech acts are the fundamental units of communication. When a user says "Can you pass the salt?", they're not asking about ability — they're making a **request**. This document covers speech act recognition for conversational AI.

---

## Speech Act Types

```rust
pub enum SpeechAct {
    /// Asserting a fact: "Tomek dał jabłko Izie"
    Assert {
        proposition: InterlinguaNode,
        certainty: Certainty,
    },
    
    /// Asking a question: "Czy Tomek dał jabłko?"
    Question {
        question_type: QuestionType,
        expected_answer: Option<AnswerType>,
    },
    
    /// Requesting action: "Podaj mi sól" / "Czy mógłbyś podać sól?"
    Request {
        action: InterlinguaNode,
        politeness: Politeness,
    },
    
    /// Commanding: "Zrób to!" / "Siadaj!"
    Command {
        action: InterlinguaNode,
        urgency: Urgency,
    },
    
    /// Offering: "Chcesz herbaty?" / "Mogę pomóc"
    Offer {
        offered: InterlinguaNode,
    },
    
    /// Promising: "Zrobię to jutro"
    Promise {
        action: InterlinguaNode,
        timeframe: Option<TemporalReference>,
    },
    
    /// Apologizing: "Przepraszam" / "Sorry"
    Apologize {
        reason: Option<InterlinguaNode>,
    },
    
    /// Thanking: "Dziękuję" / "Thanks"
    Thank {
        reason: Option<InterlinguaNode>,
    },
    
    /// Greeting: "Cześć" / "Hello"
    Greet,
    
    /// Farewell: "Pa" / "Goodbye"
    Farewell,
    
    /// Expressing emotion: "Jestem szczęśliwy" / "I'm happy"
    ExpressEmotion {
        emotion: Emotion,
        intensity: Intensity,
    },
}
```

## Question Types

```rust
pub enum QuestionType {
    /// Yes/no question: "Czy Tomek dał jabłko?"
    YesNo,
    
    /// Wh-question: "Co Tomek dał Izie?"
    Wh {
        wh_role: SemanticRole,
    },
    
    /// Alternative question: "Chcesz herbatę czy kawę?"
    Alternative {
        options: Vec<InterlinguaNode>,
    },
    
    /// Rhetorical question (not expecting answer)
    Rhetorical,
    
    /// Tag question: "Dałeś, prawda?"
    Tag {
        main_proposition: InterlinguaNode,
    },
}

pub enum AnswerType {
    Boolean,           // yes/no
    Entity,            // specific entity
    Set,               // list of entities
    Description,       // open-ended description
    Confirmation,      // confirm/deny
}
```

## Recognition Pipeline

```rust
pub struct SpeechActRecognizer;

impl SpeechActRecognizer {
    pub fn recognize(
        utterance: &InterlinguaNode,
    ) -> SpeechAct {
        match utterance {
            InterlinguaNode::Natural(utt) => {
                Self::recognize_natural(utt)
            }
            _ => SpeechAct::Assert {
                proposition: utterance.clone(),
                certainty: Certainty::Certain,
            },
        }
    }
    
    fn recognize_natural(utterance: &Utterance) -> SpeechAct {
        for sentence in &utterance.sentences {
            // Check illocution first
            match sentence.illocution {
                Illocution::YesNoQuestion => {
                    return Self::classify_yes_no_question(sentence, utterance);
                }
                
                Illocution::WhQuestion { wh_role } => {
                    return SpeechAct::Question {
                        question_type: QuestionType::Wh { wh_role },
                        expected_answer: Some(AnswerType::Entity),
                    };
                }
                
                Illocution::Command => {
                    return SpeechAct::Command {
                        action: InterlinguaNode::Natural(
                            Utterance { sentences: vec![sentence.clone()], ..utterance.clone() }
                        ),
                        urgency: Self::detect_urgency(sentence),
                    };
                }
                
                Illocution::Statement => {
                    // Could be assertion, request, offer, promise, etc.
                    return Self::classify_statement(sentence, utterance);
                }
                
                _ => {}
            }
        }
        
        // Default: assertion
        SpeechAct::Assert {
            proposition: utterance.clone().into(),
            certainty: Certainty::Certain,
        }
    }
}
```

## Statement Classification

Statements can be assertions, indirect requests, offers, promises, etc.

```rust
impl SpeechActRecognizer {
    fn classify_statement(
        sentence: &Sentence,
        utterance: &Utterance,
    ) -> SpeechAct {
        // Check for modal verbs → indirect speech acts
        match sentence.modality {
            Modality::Deontic => {
                // Obligation → could be indirect request
                // "Musisz to zrobić" = "You must do it" → indirect command
                
                if sentence.has_modal("móc") || sentence.has_modal("chcieć") {
                    // "Czy mógłbyś..." = indirect request
                    return SpeechAct::Request {
                        action: Self::extract_action(sentence),
                        politeness: Politeness::Polite,
                    };
                }
            }
            
            Modality::Epistemic => {
                // Uncertainty → assertion with hedging
                return SpeechAct::Assert {
                    proposition: utterance.clone().into(),
                    certainty: Certainty::Uncertain,
                };
            }
            
            _ => {}
        }
        
        // Check for performative verbs
        if let Some(frame) = sentence.frames.first() {
            match frame {
                Frame::Communication { message, .. } => {
                    if Self::is_promise(message) {
                        return SpeechAct::Promise {
                            action: message.clone().into(),
                            timeframe: sentence.temporal.clone(),
                        };
                    }
                }
                
                Frame::Transfer { agent, theme, .. } => {
                    if Self::is_speaker(agent, utterance) {
                        // Speaker giving something → offer
                        return SpeechAct::Offer {
                            offered: theme.clone().into(),
                        };
                    }
                }
                
                _ => {}
            }
        }
        
        // Default: assertion
        SpeechAct::Assert {
            proposition: utterance.clone().into(),
            certainty: Certainty::Certain,
        }
    }
}
```

## Yes/No Question Classification

Yes/no questions can be genuine questions, requests, offers, etc.

```rust
impl SpeechActRecognizer {
    fn classify_yes_no_question(
        sentence: &Sentence,
        utterance: &Utterance,
    ) -> SpeechAct {
        // Check for modal + ability verb → indirect request
        // "Czy możesz podać sól?" → Request (not genuine question)
        if sentence.has_modal("móc") && sentence.has_verb_in_infinitive() {
            return SpeechAct::Request {
                action: Self::extract_action(sentence),
                politeness: Politeness::Polite,
            };
        }
        
        // Check for "chcieć" → offer
        // "Chcesz herbaty?" → Offer
        if sentence.has_verb("chcieć") {
            return SpeechAct::Offer {
                offered: Self::extract_object(sentence),
            };
        }
        
        // Check for negation → expectation of "yes"
        // "Nie chcesz herbaty?" → Offer with expectation
        if sentence.polarity == Polarity::Negative {
            return SpeechAct::Offer {
                offered: Self::extract_object(sentence),
            };
        }
        
        // Default: genuine question
        SpeechAct::Question {
            question_type: QuestionType::YesNo,
            expected_answer: Some(AnswerType::Boolean),
        }
    }
}
```

## Polish Speech Act Markers

### Direct Markers

```rust
pub enum PolishSpeechActMarker {
    // Questions
    Czy,                    // yes/no question particle
    QuestionWord(String),   // co, kto, gdzie, kiedy, jak, dlaczego
    
    // Commands
    Imperative,             // verb in imperative form
    Proszę,                 // please
    
    // Greetings
    Cześć,
    DzieńDobry,
    DobryWieczór,
    
    // Farewells
    Pa,
    DoWidzenia,
    NaRazie,
    
    // Thanks
    Dziękuję,
    Dzięki,
    
    // Apology
    Przepraszam,
    
    // Agreement
    Tak,
    Oczywiście,
    
    // Disagreement
    Nie,
    Niestety,
}
```

### Indirect Markers

```
Indirect Request patterns (PL):
  "Czy mógłbyś X?" → Request (polite)
  "Może byś X?" → Request (casual)
  "Nie chcesz X?" → Offer
  "Trzeba X" → Indirect command ("It's necessary to X")

Indirect Assertion patterns:
  "Chyba X" → Uncertain assertion
  "Pewnie X" → Probable assertion
  "Słyszałem, że X" → Reported assertion
```

### English Speech Act Markers

```
Indirect Request patterns (EN):
  "Could you X?" → Request (polite)
  "Would you mind X?" → Request (very polite)
  "Can you X?" → Request (casual)
  "How about X?" → Suggestion/Offer

Indirect Offer patterns:
  "Would you like X?" → Offer
  "Do you want X?" → Offer (casual)
  "Can I get you X?" → Offer
```

## Examples

### Request Disguised as Question

```
Input: "Czy mógłbyś podać sól?"

Parse:
  Illocution: YesNoQuestion
  Frame: TRANSFER { agent: listener, theme: SALT, recipient: speaker }
  Modality: CONDITIONAL

Recognition:
  → Modal "mógłbyś" + infinitive "podać" → indirect request
  → SpeechAct::Request {
      action: "podać sól",
      politeness: Polite,
    }

NOT a genuine question about ability.
```

### Offer Disguised as Question

```
Input: "Chcesz herbaty?"

Parse:
  Illocution: YesNoQuestion
  Frame: DESIRE { experiencer: listener, stimulus: TEA }

Recognition:
  → "chcieć" + question → offer
  → SpeechAct::Offer {
      offered: TEA,
    }
```

### Promise

```
Input: "Zrobię to jutro."

Parse:
  Illocution: Statement
  Frame: CREATION { creator: speaker, created: TASK }
  Tense: FUTURE

Recognition:
  → Speaker + future tense + task → promise
  → SpeechAct::Promise {
      action: "zrobić task",
      timeframe: Some(Tomorrow),
    }
```

### Command vs Request

```
"Zrób to!" → Command (imperative, direct)
"Zrób to, proszę" → Command (imperative, softened)
"Czy mógłbyś to zrobić?" → Request (question form, polite)
"Może byś to zrobił?" → Request (suggestion form, casual)
```

## Integration with Dialogue Manager

Speech acts drive the dialogue manager's response planning:

```rust
impl DialogueManager {
    pub fn process_speech_act(&mut self, speech_act: &SpeechAct) -> ResponsePlan {
        match speech_act {
            SpeechAct::Question { .. } => {
                ResponsePlan::Answer { /* provide answer */ }
            }
            
            SpeechAct::Request { action, politeness } => {
                if self.can_fulfill(action) {
                    ResponsePlan::Accept { action: action.clone() }
                } else {
                    ResponsePlan::Decline { reason: "cannot fulfill" }
                }
            }
            
            SpeechAct::Greet => {
                ResponsePlan::GreetBack
            }
            
            SpeechAct::Thank { .. } => {
                ResponsePlan::Acknowledge { message: "you're welcome" }
            }
            
            SpeechAct::Offer { offered } => {
                if self.wants(offered) {
                    ResponsePlan::AcceptOffer
                } else {
                    ResponsePlan::DeclineOffer { reason: "not interested" }
                }
            }
            
            _ => ResponsePlan::Default,
        }
    }
}
```

## Summary

Speech act recognition:

1. **Classifies communicative intent** — question, request, command, offer, promise, etc.
2. **Handles indirect speech acts** — "Can you X?" = request, not question
3. **Uses illocution + context** — grammatical form + modality + speaker role
4. **Language-specific markers** — Polish "czy", English "could you"
5. **Drives response planning** — different speech acts require different responses
