# Conversation Context

`lexflex-conversation` stores semantic turns and entity mentions. Turn IDs and
mention provenance are validated before they enter the state. Reference
resolution ranks candidates deterministically by salience and returns an
explicit `Ambiguous` result on equal scores; it never silently selects one.

Conversation state is serializable, and `verify` rebuilds it through the same
insertion boundaries to reject corrupted references.
`ConversationState::save_to_file` and `load_from_file` provide an atomic,
verified JSON round-trip for session reloads.
