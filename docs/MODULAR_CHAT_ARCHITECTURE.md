# Modular Chat Architecture

## Goal

Build a modular chat layer above `LexFlexAPI` with:

- composable widgets: header, footer, chat window, command popup, settings popup
- per-widget data sources instead of direct access to global app state
- explicit session context
- slash commands with suggestion popup
- minimal first command set:
  - `/translate en pl`
  - `/translate pl en`
  - `/settings`
- runtime setting for `verbosity`

This architecture should let us keep the current TUI implementation, but move the domain and application logic out of the monolithic `src/chat.rs`.

---

## Design Constraints

- `LexFlexAPI` remains the language backend.
- TUI is only one frontend adapter.
- Widget rendering must not own business logic.
- Slash command parsing must be testable without terminal I/O.
- Session state must be serializable independently from the renderer.
- Widget data should be read through stable view models, not raw mutable access to the entire app state.

---

## High-Level Architecture

```text
┌──────────────────────────────────────────────────────────┐
│                        Chat Shell                        │
│                 event loop + focus routing               │
└───────────────┬───────────────────────────────┬──────────┘
                │                               │
                ▼                               ▼
┌──────────────────────────┐       ┌──────────────────────────┐
│      Session Store       │       │      Command Engine      │
│ state, transcript,       │       │ parse slash commands,    │
│ context, settings, focus │       │ suggestions, execution   │
└───────────────┬──────────┘       └──────────────┬───────────┘
                │                                 │
                └──────────────┬──────────────────┘
                               ▼
                   ┌──────────────────────┐
                   │     Chat Service     │
                   │ explain/translate/   │
                   │ learner orchestration│
                   └──────────┬───────────┘
                              ▼
                       ┌──────────────┐
                       │  LexFlexAPI  │
                       └──────────────┘
```

---

## Module Split

Suggested target layout:

```text
src/chat/
  mod.rs
  app.rs
  shell.rs
  session.rs
  service.rs
  commands.rs
  context.rs
  settings.rs
  transcript.rs
  widgets/
    mod.rs
    header.rs
    footer.rs
    chat_window.rs
    command_popup.rs
    settings_popup.rs
  tui/
    mod.rs
    terminal.rs
    input.rs
    render.rs
```

### Responsibilities

- `app.rs`
  - top-level composition root
  - wires shell, session, service, commands

- `shell.rs`
  - app event loop
  - input routing
  - popup visibility and focus transitions

- `session.rs`
  - mutable session state
  - transcript
  - current direction `from/to`
  - request lifecycle

- `service.rs`
  - request execution against `LexFlexAPI`
  - maps domain results into structured chat responses
  - no TUI code

- `commands.rs`
  - slash command parser
  - command completion
  - command execution plans

- `context.rs`
  - long-lived chat context
  - active translation mode
  - future extension for dialogue graph references

- `settings.rs`
  - runtime settings model
  - current `verbosity`

- `transcript.rs`
  - serializable message log
  - save/load support

- `widgets/*`
  - pure view components with dedicated input view models

- `tui/*`
  - ratatui/crossterm adapter only

---

## Core Domain Types

### Session

```rust
pub struct ChatSession {
    pub context: ChatContext,
    pub settings: ChatSettings,
    pub transcript: Transcript,
    pub pending: Option<PendingRequest>,
    pub overlays: OverlayState,
    pub input: ComposerState,
}
```

### Context

```rust
pub struct ChatContext {
    pub source_lang: String,
    pub target_lang: String,
    pub active_mode: ChatMode,
}

pub enum ChatMode {
    Translate,
}
```

For v1, `ChatMode::Translate` is enough. The rest of the existing `explain`, `parse`, `learn` flow can stay in the backend but does not need UI exposure yet.

### Settings

```rust
pub struct ChatSettings {
    pub verbosity: Verbosity,
}

pub enum Verbosity {
    Compact,
    Normal,
    Detailed,
}
```

Meaning:

- `Compact`
  - show translated output only
- `Normal`
  - show translated output and short notes
- `Detailed`
  - show translated output, semantic digest, latency, trace notes

### Transcript

```rust
pub struct Transcript {
    pub messages: Vec<TranscriptMessage>,
    pub next_turn: usize,
}

pub struct TranscriptMessage {
    pub turn: usize,
    pub role: MessageRole,
    pub title: String,
    pub blocks: Vec<MessageBlock>,
    pub meta: MessageMeta,
}

pub enum MessageBlock {
    Paragraph(String),
    KeyValue { key: String, value: String },
    BulletList(Vec<String>),
}

pub struct MessageMeta {
    pub latency_ms: Option<u128>,
    pub command: Option<String>,
}
```

This is more stable than storing one preformatted `body: String`.

---

## Chat Service

`ChatService` is the application layer between session/commands and `LexFlexAPI`.

```rust
pub struct ChatService {
    api: LexFlexAPI,
}

impl ChatService {
    pub fn translate(
        &self,
        input: &str,
        context: &ChatContext,
        settings: &ChatSettings,
    ) -> Result<ChatResponse, ChatServiceError>;
}
```

### Structured Response

```rust
pub struct ChatResponse {
    pub title: String,
    pub blocks: Vec<MessageBlock>,
    pub notes: Vec<String>,
    pub latency_ms: u128,
}
```

`verbosity` controls response shaping:

- `Compact`
  - one `Paragraph` with translated output
- `Normal`
  - translated output + one short metadata block
- `Detailed`
  - translated output + digest + notes + latency

This is the main reason to move away from `String`-only responses.

---

## Command System

The slash command layer should be a separate parser and completion engine.

### Supported Commands in v1

```text
/translate en pl
/translate pl en
/settings
```

### Parsed Command Model

```rust
pub enum SlashCommand {
    Translate { from: String, to: String },
    Settings,
}
```

### Parse Result

```rust
pub enum InputAction {
    UserMessage(String),
    Slash(SlashCommand),
    IncompleteSlash(SlashCommandDraft),
}
```

### Suggestions

```rust
pub struct SlashSuggestion {
    pub label: String,
    pub replacement: String,
    pub description: String,
}
```

Example suggestions:

- `/tr` -> `/translate`
- `/translate` -> `/translate en pl`, `/translate pl en`
- `/set` -> `/settings`

The popup opens whenever the composer starts with `/`.

---

## Overlay Model

```rust
pub struct OverlayState {
    pub active: Option<OverlayKind>,
    pub command_popup: CommandPopupState,
    pub settings_popup: SettingsPopupState,
}

pub enum OverlayKind {
    CommandPalette,
    Settings,
}
```

Rules:

- typing `/` opens `CommandPalette`
- selecting `/settings` opens `Settings`
- `Esc` closes current overlay
- `Enter` accepts highlighted command suggestion when command popup is focused

---

## Widget Model

Each widget gets a narrow datasource trait or immutable view model.

Avoid giving widgets the whole `ChatSession`.

### Header Widget

Purpose:

- session summary
- current translation direction
- active verbosity
- backend status

Datasource:

```rust
pub struct HeaderViewModel {
    pub title: String,
    pub source_lang: String,
    pub target_lang: String,
    pub verbosity: Verbosity,
    pub pending_label: Option<String>,
    pub backend_ready: bool,
}
```

### Chat Window Widget

Purpose:

- render transcript
- manage scroll offset

Datasource:

```rust
pub struct ChatWindowViewModel<'a> {
    pub messages: &'a [TranscriptMessage],
    pub scroll: u16,
}
```

### Footer Widget

Purpose:

- composer
- inline hints
- active mode

Datasource:

```rust
pub struct FooterViewModel<'a> {
    pub input: &'a str,
    pub cursor: usize,
    pub status_line: String,
}
```

### Command Popup Widget

Purpose:

- show slash suggestions
- highlight current selection

Datasource:

```rust
pub struct CommandPopupViewModel<'a> {
    pub query: &'a str,
    pub suggestions: &'a [SlashSuggestion],
    pub selected_index: usize,
}
```

### Settings Popup Widget

Purpose:

- show runtime settings
- edit `verbosity`

Datasource:

```rust
pub struct SettingsPopupViewModel {
    pub verbosity: Verbosity,
    pub selected_row: usize,
}
```

---

## State Flow

### Message Send

```text
user presses Enter
-> shell reads composer input
-> commands parser classifies input
-> if plain text:
   -> service.translate(...)
   -> transcript.append(user)
   -> transcript.append(assistant)
-> widgets re-render from updated view models
```

### Slash Popup

```text
input starts with "/"
-> commands engine builds suggestions
-> shell sets overlay = CommandPalette
-> command popup renders suggestions
-> Enter selects suggestion
-> composer is replaced with selected command template
```

### Settings

```text
user submits "/settings"
-> shell opens Settings overlay
-> left/right or up/down changes verbosity
-> session.settings updated
-> future responses use new verbosity
```

---

## Event Routing

The shell owns focus and routes input to one target at a time.

```rust
pub enum FocusTarget {
    Composer,
    CommandPopup,
    SettingsPopup,
}
```

Rules:

- default focus is `Composer`
- when command popup opens, focus remains in composer until user uses arrow keys
- `/settings` moves focus to `SettingsPopup`
- `Esc` returns focus to `Composer`

---

## Why Datasources Per Widget

This is the key modularity rule.

Without it:

- every widget knows the entire app state
- render code and business logic get coupled again
- testing widgets is hard

With datasource/view model boundaries:

- widgets are pure renderers
- shell controls state transitions
- service controls domain logic
- command engine controls parsing and suggestions

That gives a stable split:

- `state mutation` in shell/session
- `use cases` in service
- `presentation` in widgets

---

## Migration Plan From Current `src/chat.rs`

### Step 1

Extract non-UI logic first:

- `TraceMode`
- request/response model
- `handle_request`
- transcript save/load
- slash parsing

### Step 2

Introduce structured transcript blocks instead of `body: String`.

### Step 3

Split current `App` into:

- `ChatSession`
- `OverlayState`
- `ComposerState`
- `ChatShell`

### Step 4

Move all `ratatui` rendering to `widgets/*`.

### Step 5

Reduce `src/main.rs` to:

```rust
lexflex::chat::run_tui(options)
```

---

## Recommended Initial Public API

```rust
pub mod chat {
    pub use settings::Verbosity;
    pub use session::ChatSession;
    pub use service::{ChatResponse, ChatService};
    pub use commands::{SlashCommand, SlashSuggestion};

    pub fn run_tui(options: ChatOptions) -> Result<(), ChatError>;
}
```

This is enough to support:

- current terminal chat
- future integration tests over command parsing and service responses
- future alternate frontends

---

## v1 Scope

Implement now:

- modular widgets
- widget view models
- slash popup
- `/translate en pl`
- `/translate pl en`
- `/settings`
- `verbosity` with `Compact | Normal | Detailed`

Do not implement yet:

- multi-turn reasoning
- long-term memory
- generic command registry
- learner UI
- dialogue planning
- LLM chat persona

Those belong to a later conversation engine, not to the first modular TUI architecture.
