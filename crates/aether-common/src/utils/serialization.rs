use serde::{de::DeserializeOwned, Serialize};

use crate::errors::{AetherError, AetherResult};

pub fn to_json<T: Serialize>(v: &T) -> AetherResult<String> {
    serde_json::to_string(v).map_err(|e| AetherError::InvalidJobSpec(e.to_string()))
}

pub fn from_json<T: DeserializeOwned>(s: &str) -> AetherResult<T> {
    serde_json::from_str(s).map_err(|e| AetherError::InvalidJobSpec(e.to_string()))
}
