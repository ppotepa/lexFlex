mod digest;
mod error;
mod serialize;

pub use digest::{CanonicalDigest, CanonicalDigestError};
pub use error::CanonicalHashError;
pub use serialize::canonical_bytes;

use serde::Serialize;
use sha2::{Digest, Sha256};

pub fn canonical_hash<T>(value: &T) -> Result<CanonicalDigest, CanonicalHashError>
where
    T: Serialize + ?Sized,
{
    let bytes = canonical_bytes(value)?;
    let mut digest = Sha256::new();
    digest.update(bytes);
    let value = format!("{:x}", digest.finalize());
    CanonicalDigest::new(value).map_err(|error| CanonicalHashError::Serialization {
        message: format!("internal sha256 digest invariant: {error}"),
    })
}
