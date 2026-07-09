# Response Planning — Generating Bot Responses

**⚠️ FEATURE - v0.2+ (NOT IN MVP v0.1)**

Response planning is a feature planned for v0.2+. This document specifies the design for future implementation.

---

## Overview

Response planning bridges understanding and generation. Given a user's intent and the current dialogue state, the response planner decides **what to say** before the generator decides **how to say it**.

---

## Response Plan

```rust
pub struct ResponsePlan {
    /// Sequence of actions to perform
    pub actions: Vec<ResponseAction>,
    
    /// Target register (formal, informal, etc.)
    pub tone: Register,
    
    /// Whether clarification is needed before responding
    pub needs_clarification: bool,
    
    /// Optional: ground the response in specific facts
    pub grounding: Vec<Fact>,
}

pub enum ResponseAction {
    /// Provide direct answer
    Answer {
        content: InterlinguaNode,
    },
    
    /// Acknowledge what user said
    Acknowledge {
        message: InterlinguaNode,
    },
    
    /// Ask for clarification
    AskClarification {
        question: InterlinguaNode,
    },
    
    /// Perform an action (API call, file operation, etc.)
    PerformAction {
        action: Action,
    },
    
    /// Decline a request
    Decline {
        reason: InterlinguaNode,
    },
    
    /// Change topic
    ChangeTopic {
        new_topic: Entity,
    },
    
    /// Express empathy
    Empathize {
        emotion: Emotion,
        message: InterlinguaNode,
    },
    
    /// Correct user's misconception
    Correct {
        correction: InterlinguaNode,
    },
}
```

## Response Planner

```rust
pub struct ResponsePlanner;

impl ResponsePlanner {
    pub fn plan(
        intent: &Intent,
        dialogue_state: &DialogueState,
        discourse: &Discourse,
        memory: &LongTermMemory,
        goals: &[DialogueGoal],
    ) -> ResponsePlan {
        // Check active goals first
        if let Some(goal_response) = Self::plan_for_goals(goals, intent) {
            return goal_response;
        }
        
        // Plan based on intent
        match intent {
            Intent::Inquire { about, question_type, specificity } => {
                Self::plan_answer(about, question_type, specificity, memory)
            }
            
            Intent::Desire { what, from, urgency } => {
                Self::plan_fulfillment(what, from, urgency, memory)
            }
            
            Intent::Inform { topic, content } => {
                Self::plan_acknowledgment(topic, content, memory)
            }
            
            Intent::ExpressEmotion { emotion, target } => {
                Self::plan_empathy(emotion, target)
            }
            
            Intent::SocialAct { act } => {
                Self::plan_social_response(act)
            }
            
            Intent::Confirm { proposition, confirmed } => {
                Self::plan_confirmation(proposition, *confirmed)
            }
            
            Intent::Correct { original, correction } => {
                Self::plan_correction_acceptance(original, correction)
            }
            
            Intent::Negotiate { topic, position } => {
                Self::plan_negotiation(topic, position)
            }
        }
    }
}
```

## Planning by Intent Type

### Answering Questions

```rust
impl ResponsePlanner {
    fn plan_answer(
        about: &Entity,
        question_type: &QuestionType,
        specificity: &Specificity,
        memory: &LongTermMemory,
    ) -> ResponsePlan {
        // Find relevant information
        let facts = memory.retrieve_relevant(about, 5);
        
        if facts.is_empty() {
            return ResponsePlan {
                actions: vec![
                    ResponseAction::Decline {
                        reason: InterlinguaNode::from_text("I don't know about that"),
                    }
                ],
                tone: Register::Neutral,
                needs_clarification: false,
                grounding: vec![],
            };
        }
        
        match question_type {
            QuestionType::YesNo => {
                // Find boolean answer
                let answer = Self::evaluate_yes_no(about, &facts);
                
                ResponsePlan {
                    actions: vec![ResponseAction::Answer {
                        content: InterlinguaNode::Literal(Literal::Boolean(answer)),
                    }],
                    tone: Register::Neutral,
                    needs_clarification: false,
                    grounding: facts,
                }
            }
            
            QuestionType::Wh { .. } => {
                // Find specific entity
                let answer = Self::find_entity(about, &facts);
                
                ResponsePlan {
                    actions: vec![ResponseAction::Answer {
                        content: answer.into(),
                    }],
                    tone: Register::Neutral,
                    needs_clarification: false,
                    grounding: facts,
                }
            }
            
            QuestionType::Alternative { options } => {
                // Evaluate each option
                let evaluations: Vec<_> = options.iter()
                    .map(|opt| (opt.clone(), Self::evaluate(opt, &facts)))
                    .collect();
                
                let best = evaluations.iter()
                    .max_by_key(|(_, score)| *score)
                    .map(|(opt, _)| opt.clone());
                
                ResponsePlan {
                    actions: vec![ResponseAction::Answer {
                        content: best.unwrap_or_else(|| options[0].clone()).into(),
                    }],
                    tone: Register::Neutral,
                    needs_clarification: false,
                    grounding: facts,
                }
            }
        }
    }
}
```

### Fulfilling Requests

```rust
impl ResponsePlanner {
    fn plan_fulfillment(
        what: &InterlinguaNode,
        from: &Option<Entity>,
        urgency: &Urgency,
        memory: &LongTermMemory,
    ) -> ResponsePlan {
        // Check if we can fulfill the request
        if !Self::can_fulfill(what) {
            return ResponsePlan {
                actions: vec![
                    ResponseAction::Acknowledge {
                        message: InterlinguaNode::from_text("I understand"),
                    },
                    ResponseAction::Decline {
                        reason: InterlinguaNode::from_text("but I can't do that"),
                    },
                ],
                tone: Register::Neutral,
                needs_clarification: false,
                grounding: vec![],
            };
        }
        
        // Plan fulfillment
        let mut actions = vec![
            ResponseAction::Acknowledge {
                message: InterlinguaNode::from_text("Sure"),
            },
            ResponseAction::PerformAction {
                action: Action::from_interlingua(what),
            },
        ];
        
        // Adjust tone based on urgency
        let tone = match urgency {
            Urgency::Immediate => Register::Neutral,
            Urgency::High => Register::Neutral,
            Urgency::Normal => Register::Neutral,
            Urgency::Low => Register::Informal,
        };
        
        ResponsePlan {
            actions,
            tone,
            needs_clarification: false,
            grounding: vec![],
        }
    }
}
```

### Empathetic Responses

```rust
impl ResponsePlanner {
    fn plan_empathy(
        emotion: &Emotion,
        target: &Option<Entity>,
    ) -> ResponsePlan {
        let empathetic_message = match emotion {
            Emotion::Joy => InterlinguaNode::from_text("That's great!"),
            Emotion::Sadness => InterlinguaNode::from_text("I'm sorry to hear that"),
            Emotion::Anger => InterlinguaNode::from_text("I understand your frustration"),
            Emotion::Fear => InterlinguaNode::from_text("That sounds concerning"),
            Emotion::Surprise => InterlinguaNode::from_text("That's surprising!"),
            Emotion::Neutral => InterlinguaNode::from_text("I see"),
        };
        
        ResponsePlan {
            actions: vec![
                ResponseAction::Empathize {
                    emotion: emotion.clone(),
                    message: empathetic_message,
                },
            ],
            tone: Register::Informal,
            needs_clarification: false,
            grounding: vec![],
        }
    }
}
```

### Social Responses

```rust
impl ResponsePlanner {
    fn plan_social_response(act: &SpeechAct) -> ResponsePlan {
        match act {
            SpeechAct::Greet => ResponsePlan {
                actions: vec![ResponseAction::Answer {
                    content: InterlinguaNode::from_text("Hello! How can I help you?"),
                }],
                tone: Register::Neutral,
                needs_clarification: false,
                grounding: vec![],
            },
            
            SpeechAct::Farewell => ResponsePlan {
                actions: vec![ResponseAction::Answer {
                    content: InterlinguaNode::from_text("Goodbye! Have a great day!"),
                }],
                tone: Register::Neutral,
                needs_clarification: false,
                grounding: vec![],
            },
            
            SpeechAct::Thank { .. } => ResponsePlan {
                actions: vec![ResponseAction::Answer {
                    content: InterlinguaNode::from_text("You're welcome!"),
                }],
                tone: Register::Neutral,
                needs_clarification: false,
                grounding: vec![],
            },
            
            SpeechAct::Apologize { .. } => ResponsePlan {
                actions: vec![ResponseAction::Answer {
                    content: InterlinguaNode::from_text("No problem!"),
                }],
                tone: Register::Informal,
                needs_clarification: false,
                grounding: vec![],
            },
            
            _ => ResponsePlan::default(),
        }
    }
}
```

## Response Style Adaptation

```rust
impl ResponsePlanner {
    fn adapt_style(
        plan: ResponsePlan,
        user_profile: &UserProfile,
        discourse: &Discourse,
    ) -> ResponsePlan {
        let mut adapted = plan;
        
        // Match user's register
        if let Some(user_register) = user_profile.preferred_register {
            adapted.tone = user_register;
        }
        
        // Match user's verbosity preference
        match user_profile.preferred_verbosity {
            Verbosity::Brief => {
                adapted.actions = Self::shorten_responses(&adapted.actions);
            }
            Verbosity::Detailed => {
                adapted.actions = Self::expand_responses(&adapted.actions, discourse);
            }
            Verbosity::Normal => {}
        }
        
        // Avoid topics user doesn't like
        for topic in &user_profile.avoid_topics {
            adapted.actions.retain(|a| !a.mentions(topic));
        }
        
        adapted
    }
}
```

## Response Validation

```rust
impl ResponsePlanner {
    /// Validate response before generation
    pub fn validate(plan: &ResponsePlan, discourse: &Discourse) -> ValidationResult {
        let mut issues = vec![];
        
        // Check for contradictions with known facts
        for action in &plan.actions {
            if let ResponseAction::Answer { content } = action {
                if Self::contradicts_known_facts(content, discourse) {
                    issues.push(ValidationIssue::Contradiction);
                }
            }
        }
        
        // Check for repetition
        if Self::is_repetitive(plan, discourse) {
            issues.push(ValidationIssue::Repetition);
        }
        
        // Check appropriateness
        if Self::is_inappropriate(plan, discourse) {
            issues.push(ValidationIssue::Inappropriate);
        }
        
        ValidationResult { issues }
    }
}
```

## Summary

Response planning:

1. **Decides what to say** — not how to say it (that's the generator's job)
2. **Intent-driven** — different intents require different response types
3. **Goal-aware** — active dialogue goals take priority
4. **Empathetic** — adapts tone to user's emotional state
5. **Style-adaptive** — matches user's preferences (register, verbosity)
6. **Validated** — checks for contradictions, repetition, appropriateness
7. **Produces Interlingua** — language-neutral, ready for any target language
