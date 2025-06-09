use derive_more::{Deref, DerefMut};
use jolt_sdk::{self as jolt, Serializable};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A wrapper around JoltHyperKZGProof that implements serialization
#[derive(Deref, DerefMut)]
pub struct JoltProofWrapper {
    inner: jolt::JoltHyperKZGProof,
}

impl fmt::Debug for JoltProofWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JoltProofWrapper")
            .field("proof", &"<JoltHyperKZGProof>")
            .finish()
    }
}

impl Clone for JoltProofWrapper {
    fn clone(&self) -> Self {
        // Since we can't clone the inner proof, we'll need to serialize and deserialize it
        let bytes = self
            .inner
            .serialize_to_bytes()
            .expect("Failed to serialize proof for cloning");
        let proof = jolt::JoltHyperKZGProof::deserialize_from_bytes(&bytes)
            .expect("Failed to deserialize proof for cloning");
        Self { inner: proof }
    }
}

impl JoltProofWrapper {
    /// Create a new JoltProofWrapper from a JoltHyperKZGProof
    pub fn new(proof: jolt::JoltHyperKZGProof) -> Self {
        Self { inner: proof }
    }
}

impl From<jolt::JoltHyperKZGProof> for JoltProofWrapper {
    fn from(proof: jolt::JoltHyperKZGProof) -> Self {
        Self { inner: proof }
    }
}

impl Into<jolt::JoltHyperKZGProof> for JoltProofWrapper {
    fn into(self) -> jolt::JoltHyperKZGProof {
        self.inner
    }
}

impl Serialize for JoltProofWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self
            .inner
            .serialize_to_bytes()
            .map_err(|e| serde::ser::Error::custom(format!("Failed to serialize proof: {}", e)))?;
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for JoltProofWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        let proof = jolt::JoltHyperKZGProof::deserialize_from_bytes(&bytes)
            .map_err(|e| serde::de::Error::custom(format!("Failed to deserialize proof: {}", e)))?;
        Ok(JoltProofWrapper { inner: proof })
    }
}

/// A bundle containing the input, output, and proof of a Jolt computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoltProofBundle<T, U> {
    /// The input to the computation
    pub input: T,
    /// The output of the computation
    pub output: U,
    /// The proof of the computation
    pub proof: JoltProofWrapper,
}

impl<T, U> JoltProofBundle<T, U> {
    /// Create a new JoltProofBundle
    pub fn new(input: T, output: U, proof: jolt::JoltHyperKZGProof) -> Self {
        Self {
            input,
            output,
            proof: JoltProofWrapper::new(proof),
        }
    }
}
