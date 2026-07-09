# Dialogue Management — Multi-Turn Conversations

**⚠️ FEATURE - v0.2+ (NOT IN MVP v0.1)**

Dialogue management is a feature planned for v0.2+. This document specifies the design for future implementation.

---

## Overview

The dialogue manager orchestrates multi-turn conversations, tracking state, managing turns, and coordinating between speech acts, intents, and response planning.

---

## Dialogue State

```rust
pub struct DialogueManager {
    /// Current dialogue state
    pub state: DialogueState,
    
    /// Discourse context (shared with parser/generator)
    pub discourse: Discourse,
    
    /// Long-term memory
    pub memory: LongTermMemory,
    
    /// Turn counter
    pub turn_count: usize,
    
    /// Dialogue goals (what the bot is trying to achieve)
    pub goals: Vec<DialogueGoal>,
    
    /// History of processed utterances
    pub history: Vec<DialogueTurn>,
}

pub enum DialogueState {
    /// Waiting for user input
    WaitingForInput,
    
    /// Processing user input
    Processing,
    
    /// Generating response
    GeneratingResponse,
    
    /// Asking clarification question
    Clarifying {
        question: String,
        original_intent: Intent,
    },
    
    /// Executing an action
    Executing {
        action: InterlinguaNode,
        progress: f64,
    },
    
    /// Dialogue completed
    Completed,
}
```

## Dialogue Turn

```rust
pub struct DialogueTurn {
    /// Turn number
    pub turn_id: usize,
    
    /// Who spoke
    pub speaker: Speaker,
    
    /// What was said (surface form)
    pub utterance_text: String,
    
    /// Parsed Interlingua
    pub interlingua: InterlinguaNode,
    
    /// Recognized speech act
    pub speech_act: SpeechAct,
    
    /// Extracted intent
    pub intent: Intent,
    
    /// Detected emotion
    pub emotion: Emotion,
    
    /// Response generated
    pub response: Option<String>,
    
    /// Timestamp
    pub timestamp: Timestamp,
}

pub enum Speaker {
    User,
    Bot,
}
```

## Dialogue Goals

```rust
pub enum DialogueGoal {
    /// Answer user's questions
    AnswerQuestions,
    
    /// Fulfill user's requests
    FulfillRequests,
    
    /// Gather information from user
    GatherInfo {
        required: Vec<InfoRequirement>,
        gathered: HashMap<String, ParameterValue>,
    },
    
    /// Maintain conversation flow
    MaintainFlow,
    
    /// Teach/explain something
    Teach {
        topic: Entity,
        subtopics: Vec<Entity>,
        completed: Vec<Entity>,
    },
    
    /// Complete a multi-step task
    MultiStepTask {
        steps: Vec<TaskStep>,
        current_step: usize,
    },
}

pub struct InfoRequirement {
    pub name: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub prompt: Option<String>,
}
```

## Dialogue Processing Pipeline

```rust
impl DialogueManager {
    /// Process one turn of dialogue
    pub fn process_turn(
        &mut self,
        input: &str,
        language: &LanguageId,
    ) -> Result<DialogueTurn, DialogueError> {
        self.state = DialogueState::Processing;
        
        // 1. Parse input
        let interlingua = self.parse(input, language)?;
        
        // 2. Recognize speech act
        let speech_act = SpeechActRecognizer::recognize(&interlingua);
        
        // 3. Extract intent
        let intent = IntentExtractor::extract(&interlingua, &speech_act);
        
        // 4. Detect emotion
        let emotion = EmotionRecognizer::recognize(&interlingua);
        
        // 5. Update discourse
        self.discourse.add_utterance(&interlingua);
        
        // 6. Update memory
        self.memory.update_from_utterance(&interlingua);
        
        // 7. Check if clarification is needed
        if let Some(clarification) = self.needs_clarification(&intent) {
            self.state = DialogueState::Clarifying {
                question: clarification.clone(),
                original_intent: intent.clone(),
            };
            
            return Ok(DialogueTurn {
                turn_id: self.turn_count,
                speaker: Speaker::User,
                utterance_text: input.to_string(),
                interlingua,
                speech_act,
                intent,
                emotion,
                response: Some(clarification),
                timestamp: Timestamp::now(),
            });
        }
        
        // 8. Plan response
        self.state = DialogueState::GeneratingResponse;
        let response_plan = ResponsePlanner::plan(
            &intent,
            &self.state,
            &self.discourse,
            &self.memory,
            &self.goals,
        );
        
        // 9. Generate response
        let response = self.generate_response(&response_plan, language)?;
        
        // 10. Create turn record
        let turn = DialogueTurn {
            turn_id: self.turn_count,
            speaker: Speaker::User,
            utterance_text: input.to_string(),
            interlingua,
            speech_act,
            intent,
            emotion,
            response: Some(response.clone()),
            timestamp: Timestamp::now(),
        };
        
        // 11. Update state
        self.history.push(turn.clone());
        self.turn_count += 1;
        self.state = DialogueState::WaitingForInput;
        
        // 12. Check goals
        self.check_goals();
        
        Ok(turn)
    }
}
```

## Clarification Strategy

```rust
impl DialogueManager {
    fn needs_clarification(&self, intent: &Intent) -> Option<String> {
        match intent {
            Intent::Desire { what, .. } => {
                // Check if we have all required parameters
                let params = IntentExtractor::extract_parameters(what);
                
                for requirement in self.get_requirements(what) {
                    if requirement.required && !params.has(&requirement.name) {
                        return Some(
                            requirement.prompt.clone()
                                .unwrap_or_else(|| format!("What {}?", requirement.name))
                        );
                    }
                }
                
                None
            }
            
            Intent::Inquire { about, .. } => {
                // Check if the question is too vague
                if Self::is_too_vague(about) {
                    return Some("Could you be more specific?".to_string());
                }
                
                None
            }
            
            _ => None,
        }
    }
}
```

## Multi-Step Tasks

```rust
pub struct MultiStepTaskManager;

impl MultiStepTaskManager {
    pub fn advance_step(
        &mut self,
        goal: &mut DialogueGoal,
        turn: &DialogueTurn,
    ) -> Option<String> {
        if let DialogueGoal::MultiStepTask { steps, current_step } = goal {
            if *current_step < steps.len() {
                let step = &steps[*current_step];
                
                // Check if current step is completed
                if Self::is_step_completed(step, turn) {
                    *current_step += 1;
                    
                    if *current_step < steps.len() {
                        // Prompt for next step
                        return Some(steps[*current_step].prompt.clone());
                    } else {
                        // All steps completed
                        return Some("Task completed!".to_string());
                    }
                }
            }
        }
        
        None
    }
}
```

## Dialogue Acts

The bot can perform various dialogue acts:

```rust
pub enum DialogueAct {
    /// Provide information
    Inform { content: InterlinguaNode },
    
    /// Ask a question
    Ask { question: InterlinguaNode },
    
    /// Confirm understanding
    Confirm { proposition: InterlinguaNode },
    
    /// Request clarification
    Clarify { question: String },
    
    /// Acknowledge
    Acknowledge { message: String },
    
    /// Apologize
    Apologize { reason: Option<String> },
    
    /// Offer help
    OfferHelp { topic: Option<Entity> },
    
    /// Change topic
    ChangeTopic { new_topic: Entity },
    
    /// End conversation
    EndConversation { reason: Option<String> },
}
```

## Conversation Flow Control

```rust
impl DialogueManager {
    /// Decide what to do next
    pub fn next_action(&mut self) -> DialogueAct {
        // Check if there are pending goals
        for goal in &self.goals {
            match goal {
                DialogueGoal::GatherInfo { required, gathered } => {
                    // Find missing required info
                    for req in required {
                        if req.required && !gathered.contains_key(&req.name) {
                            return DialogueAct::Ask {
                                question: InterlinguaNode::from_text(
                                    req.prompt.as_deref().unwrap_or("Tell me more")
                                ),
                            };
                        }
                    }
                }
                
                DialogueGoal::MultiStepTask { steps, current_step } => {
                    if *current_step < steps.len() {
                        return DialogueAct::Inform {
                            content: InterlinguaNode::from_text(&steps[*current_step].prompt),
                        };
                    }
                }
                
                _ => {}
            }
        }
        
        // No pending goals — wait for user
        DialogueAct::Acknowledge { message: "How can I help?".to_string() }
    }
}
```

## Context Carryover

```rust
impl DialogueManager {
    /// Carry context from previous turns
    pub fn resolve_context(&self, utterance: &mut InterlinguaNode) {
        // Resolve pronouns from discourse
        if let InterlinguaNode::Natural(ref mut utt) = utterance {
            for sentence in &mut utt.sentences {
                for entity in sentence.all_entities_mut() {
                    if entity.reference == Reference::Unresolved {
                        // Try to resolve from discourse
                        if let Some(resolved) = self.discourse.resolve(entity) {
                            entity.reference = Reference::Anaphoric(resolved.id);
                        }
                    }
                }
            }
        }
        
        // Resolve ellipsis
        // "Me too" → same action as previous speaker
        // "Same here" → same state as previous speaker
    }
}
```

## Error Recovery in Dialogue

```rust
impl DialogueManager {
    pub fn handle_error(&mut self, error: DialogueError) -> DialogueAct {
        match error {
            DialogueError::ParseFailed(_) => {
                DialogueAct::Clarify {
                    question: "I didn't understand. Could you rephrase that?".to_string(),
                }
            }
            
            DialogueError::AmbiguousIntent(_) => {
                DialogueAct::Clarify {
                    question: "Did you mean X or Y?".to_string(),
                }
            }
            
            DialogueError::CannotFulfill(_) => {
                DialogueAct::Apologize {
                    reason: Some("I can't do that right now".to_string()),
                }
            }
            
            DialogueError::LostContext => {
                DialogueAct::Clarify {
                    question: "Could you remind me what we were talking about?".to_string(),
                }
            }
        }
    }
}
```

## Summary

Dialogue management:

1. **Tracks state** — knows where we are in the conversation
2. **Manages turns** — processes user input, generates responses
3. **Handles goals** — gathers info, completes multi-step tasks
4. **Requests clarification** — when input is ambiguous or incomplete
5. **Carries context** — resolves pronouns and ellipsis from previous turns
6. **Recovers from errors** — graceful degradation when things go wrong
7. **Coordinates components** — speech acts, intents, response planning, memory
