use crate::{keys::WorkerPublicKey, server::domain::UserId};

#[derive(Copy, Clone, Debug)]
pub struct WorkerId(pub i64);

#[derive(Debug)]
pub struct Worker {
    pub id: WorkerId,
    pub owner: UserId,
    pub name: String,
    pub enabled: bool,
    pub key: Option<WorkerPublicKey>,
}
