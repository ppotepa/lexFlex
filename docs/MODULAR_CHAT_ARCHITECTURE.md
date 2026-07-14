# Chat client architecture

The terminal UI is a thin frontend for `ConversationEngine`.

```text
keyboard input
  → slash command parser
  → EngineRequest
  → ChatWorker / ConversationEngine
  → EngineResponse
  → transcript and widgets
```

## Responsibilities

- `src/chat/app.rs` composes the service, session and terminal.
- `src/chat/shell.rs` owns the event loop and input routing.
- `src/chat/commands.rs` parses slash commands.
- `src/chat/service.rs` sends typed requests to the engine worker.
- `src/chat/session.rs` owns transcript, mode and UI lifecycle state.
- `src/chat/widgets/` and `src/chat/tui/` render view models.

The chat client does not parse facts, inspect `DocumentGraph`, resolve entities, query Wikipedia or call an LLM. Those operations are engine responsibilities.

## Current commands

The supported command set is documented in [CLI.md](CLI.md). Regular messages are always submitted as `EngineRequest::UserTurn`.
