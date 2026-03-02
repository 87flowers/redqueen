use anyhow::{Result, anyhow};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use getrandom::{SysRng, rand_core::UnwrapErr};
use serde::Deserialize;

#[derive(Debug)]
pub struct WorkerPublicKey(VerifyingKey);

pub struct WorkerPrivateKey(SigningKey);

pub struct Signature([u8; 64]);

impl WorkerPublicKey {
    pub fn from_ed25519(pk: VerifyingKey) -> Self {
        Self(pk)
    }

    pub fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        let bytes = const_hex::decode_to_array(s)?;
        Ok(Self::from_ed25519(VerifyingKey::from_bytes(&bytes)?))
    }

    pub fn to_string(&self) -> String {
        const_hex::encode(self.0.to_bytes())
    }

    #[must_use]
    pub fn verify(&self, message: &[u8], signature: Signature) -> bool {
        self.0.verify(message, &ed25519_dalek::Signature::from_bytes(&signature.0)).is_ok()
    }
}

impl WorkerPrivateKey {
    const PREFIX: &'static str = "SECRET$";

    pub fn from_ed25519(pk: SigningKey) -> Self {
        Self(pk)
    }

    pub fn from_str(s: &str) -> Result<Self> {
        let s = s.trim().strip_prefix(Self::PREFIX).ok_or(anyhow!("private key must start with {}", Self::PREFIX))?;
        let bytes = const_hex::decode_to_array(s)?;
        Ok(Self::from_ed25519(SigningKey::from_bytes(&bytes)))
    }

    pub fn to_string(&self) -> String {
        let hex = const_hex::encode(self.0.to_bytes());
        format!("{}{hex}", Self::PREFIX)
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        Signature(self.0.sign(message).to_bytes())
    }
}

impl Signature {
    pub fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        let bytes = const_hex::decode_to_array(s)?;
        Ok(Signature(bytes))
    }

    pub fn to_string(&self) -> String {
        const_hex::encode(self.0)
    }
}

pub fn generate_worker_key_pair() -> (WorkerPublicKey, WorkerPrivateKey) {
    let mut csprng = UnwrapErr(SysRng);
    let private_key = SigningKey::generate(&mut csprng);
    let public_key = private_key.verifying_key();
    (WorkerPublicKey::from_ed25519(public_key), WorkerPrivateKey::from_ed25519(private_key))
}

impl std::fmt::Debug for WorkerPrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[...]")
    }
}

impl<'de> Deserialize<'de> for WorkerPublicKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(WorkerPublicKeyVisitor)
    }
}

struct WorkerPublicKeyVisitor;

impl<'de> serde::de::Visitor<'de> for WorkerPublicKeyVisitor {
    type Value = WorkerPublicKey;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("expecting a valid public key")
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match WorkerPublicKey::from_str(v) {
            Ok(v) => Ok(v),
            Err(e) => Err(E::custom(format!("{e}"))),
        }
    }
}

impl<'de> Deserialize<'de> for WorkerPrivateKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(WorkerPrivateKeyVisitor)
    }
}

struct WorkerPrivateKeyVisitor;

impl<'de> serde::de::Visitor<'de> for WorkerPrivateKeyVisitor {
    type Value = WorkerPrivateKey;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("expecting a valid private key")
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match WorkerPrivateKey::from_str(v) {
            Ok(v) => Ok(v),
            Err(e) => Err(E::custom(format!("{e}"))),
        }
    }
}
