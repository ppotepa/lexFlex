use crate::api::response::{EngineDiagnostic, EngineResponse};

pub(crate) fn with_diagnostics(
    response: EngineResponse,
    mut incoming: Vec<EngineDiagnostic>,
) -> EngineResponse {
    match response {
        EngineResponse::Error {
            code,
            message,
            diagnostics,
        } => {
            for diagnostic in diagnostics {
                if !incoming.iter().any(|existing| existing == &diagnostic) {
                    incoming.push(diagnostic);
                }
            }
            EngineResponse::Error {
                code,
                message,
                diagnostics: incoming,
            }
        }
        EngineResponse::TextAnalyzed {
            analysis,
            diagnostics,
        } => EngineResponse::TextAnalyzed {
            analysis,
            diagnostics: merge(incoming, diagnostics),
        },
        EngineResponse::TextIngested {
            analysis,
            assertion,
            outcome,
            snapshot_hash,
            diagnostics,
        } => EngineResponse::TextIngested {
            analysis,
            assertion,
            outcome,
            snapshot_hash,
            diagnostics: merge(incoming, diagnostics),
        },
        EngineResponse::TextAnswer {
            analysis,
            goal,
            solutions,
            snapshot_hash,
            diagnostics,
        } => EngineResponse::TextAnswer {
            analysis,
            goal,
            solutions,
            snapshot_hash,
            diagnostics: merge(incoming, diagnostics),
        },
        EngineResponse::TextAmbiguous {
            alternatives,
            diagnostics,
        } => EngineResponse::TextAmbiguous {
            alternatives,
            diagnostics: merge(incoming, diagnostics),
        },
        other => other,
    }
}

pub(crate) fn prepend_diagnostics(
    response: EngineResponse,
    incoming: Vec<EngineDiagnostic>,
) -> EngineResponse {
    with_diagnostics(response, incoming)
}

fn merge(
    mut incoming: Vec<EngineDiagnostic>,
    diagnostics: Vec<EngineDiagnostic>,
) -> Vec<EngineDiagnostic> {
    for diagnostic in diagnostics {
        if !incoming.iter().any(|existing| existing == &diagnostic) {
            incoming.push(diagnostic);
        }
    }
    incoming
}
