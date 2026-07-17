use crate::CanonicalHashError;
use serde::Serialize;

pub fn canonical_bytes<T>(value: &T) -> Result<Vec<u8>, CanonicalHashError>
where
    T: Serialize + ?Sized,
{
    serde_json::to_vec(value).map_err(|error| CanonicalHashError::Serialization {
        message: error.to_string(),
    })
}
