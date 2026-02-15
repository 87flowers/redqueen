use crate::common::domain::WorkerPublicKey;

pub struct Authorization {
    pub username: String,
    pub public_key: WorkerPublicKey,
    pub timestamp: i64,
    pub nonce: u64,
}
