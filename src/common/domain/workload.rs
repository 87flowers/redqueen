use std::collections::HashSet;

use crate::common::domain::{EngineBranch, EngineId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkloadId(pub i64);

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Workload {
    Bench { id: WorkloadId, config: BenchWorkloadConfig, state: Option<BenchWorkloadState> },
}

pub trait WorkloadConfig {
    fn related_engines(&self) -> HashSet<EngineId>;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BenchWorkloadConfig {
    pub engine: EngineBranch,
}

impl WorkloadConfig for BenchWorkloadConfig {
    fn related_engines(&self) -> HashSet<EngineId> {
        let mut result = HashSet::new();
        result.insert(self.engine.engine_id.clone());
        result
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BenchWorkloadState {
    pub bench: u64,
    pub nps: u64,
}
