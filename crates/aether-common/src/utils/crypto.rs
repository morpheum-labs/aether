use sha2::{Digest, Sha256};

pub fn sha256_32(bytes: &[u8]) -> [u8; 32] {
    let out = Sha256::digest(bytes);
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}
