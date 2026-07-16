use serde::de::DeserializeOwned;
use std::path::Path;

pub fn read_ron<T>(path: &Path) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let source =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    ron::from_str(&source).map_err(|error| format!("{}: {error}", path.display()))
}
