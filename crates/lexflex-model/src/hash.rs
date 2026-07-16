use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt::Write;

pub fn canonical_hash<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("canonical hashing requires serializable value");
    let mut digest = Sha256::new();
    digest.update(bytes);
    let hash = digest.finalize();
    let mut output = String::with_capacity(hash.len() * 2);
    for byte in hash {
        write!(&mut output, "{byte:02x}").expect("write to string");
    }
    output
}
