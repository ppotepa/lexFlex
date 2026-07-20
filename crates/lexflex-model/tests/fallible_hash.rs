use lexflex_model::{canonical_hash, CanonicalHashError};
use serde::ser::{Error as _, Serializer};
use serde::Serialize;

struct Failing;

impl Serialize for Failing {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(S::Error::custom("forced"))
    }
}

#[test]
fn canonical_hash_returns_error_instead_of_panicking() {
    let result = canonical_hash(&Failing);
    assert!(matches!(
        result,
        Err(CanonicalHashError::Serialization { .. })
    ));
}
