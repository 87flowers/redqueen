use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct EngineId(pub i64);

#[derive(Debug, Deserialize, Serialize)]
pub struct Engine {
    pub id: EngineId,
    pub name: String,
    pub default_url: String,
    pub default_branch: String,
    pub base_nps: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EngineBranch {
    pub engine_id: EngineId,
    pub url: String,
    pub branch_name: String,
    pub commit_hash: String,
}
