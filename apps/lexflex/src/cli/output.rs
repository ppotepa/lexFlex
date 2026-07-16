use lexflex_engine::api::response::EngineResponse;
use serde::Serialize;

pub fn print_response(response: &EngineResponse) -> Result<(), String> {
    print_json(response)
}

pub fn print_json<T: Serialize>(value: &T) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(|error| error.to_string())?
    );
    Ok(())
}
