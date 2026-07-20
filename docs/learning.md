# Learning Workflow

The learning crate represents observations and proposals separately from the
production model. `LearningOverlay` accepts a proposal only with a matching
observation, keeps it pending by default, and exposes only explicitly approved
proposals. Rejection and duplicate proposal errors are typed.

Approval does not mutate the verified base model. Promotion remains an explicit
operation: `promote` returns the approved proposal as an artifact, and a future
model loader must validate it before any overlay is applied to production.
Serialized overlays are verified on load, including duplicate proposal IDs,
required identifiers, and non-empty expected behavior.
