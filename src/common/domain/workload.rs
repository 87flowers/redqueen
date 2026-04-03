use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct EngineBranch {
    pub url: String,
    pub branch: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Workload {
    Bench { config: BenchWorkloadConfig, state: Option<BenchWorkloadState> },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BenchWorkloadConfig {
    pub engine: EngineBranch,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BenchWorkloadState {
    pub bench: u64,
    pub nps: u64,
}
