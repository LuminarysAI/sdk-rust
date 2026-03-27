//! Versioned skill state — safe serialisation across schema migrations.

use crate::types::SkillError;
use serde::{Deserialize, Serialize};

/// Wraps any skill state with a schema version for safe migration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct StateEnvelope {
    #[serde(rename = "schema_version")]
    pub schema_version: u32,
    #[serde(rename = "data", with = "crate::bytes_or_null")]
    pub data: Vec<u8>,
}

/// Encode `state` into a versioned envelope.
pub fn marshal_state<S: Serialize>(schema_version: u32, state: &S) -> Result<Vec<u8>, SkillError> {
    let data = rmp_serde::to_vec_named(state)?;
    let envelope = StateEnvelope { schema_version, data };
    Ok(rmp_serde::to_vec_named(&envelope)?)
}

/// Decode the versioned envelope and populate `dst`.
/// Returns the `schema_version` so callers can run migrations when needed.
pub fn unmarshal_state<D: serde::de::DeserializeOwned>(
    raw: &[u8],
) -> Result<(u32, D), SkillError> {
    if raw.is_empty() {
        return Err(SkillError("empty state".into()));
    }
    let envelope: StateEnvelope = rmp_serde::from_slice(raw)?;
    let value: D = rmp_serde::from_slice(&envelope.data)?;
    Ok((envelope.schema_version, value))
}
