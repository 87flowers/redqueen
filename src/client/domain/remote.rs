use reqwest::Url;
use serde::Deserialize;

use crate::common::domain::{WorkerPrivateKey, WorkerPublicKey};

#[derive(Debug, Deserialize)]
pub struct Remote {
    pub url: Url,
    pub priority: i64,
    pub username: String,
    pub public_key: WorkerPublicKey,
    pub private_key: WorkerPrivateKey,
}
