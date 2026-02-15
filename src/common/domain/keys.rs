use anyhow::{Result, anyhow};
use argon2::password_hash::rand_core::OsRng;
use ed25519_dalek::{SigningKey, VerifyingKey};

#[derive(Debug)]
pub struct WorkerPublicKey(VerifyingKey);

pub struct WorkerPrivateKey(SigningKey);

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
}

pub fn generate_worker_key_pair() -> (WorkerPublicKey, WorkerPrivateKey) {
    let private_key = SigningKey::generate(&mut OsRng);
    let public_key = private_key.verifying_key();
    (WorkerPublicKey::from_ed25519(public_key), WorkerPrivateKey::from_ed25519(private_key))
}
