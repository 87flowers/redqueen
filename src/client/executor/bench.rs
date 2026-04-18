use crate::{
    client::domain::Remote,
    common::domain::{BenchWorkloadConfig, BenchWorkloadState, Engine, EngineBranch, EngineId, WorkloadId},
};
use std::collections::HashMap;

pub fn execute(
    remote_name: &str, remote: Remote, id: WorkloadId, config: BenchWorkloadConfig, _: Option<BenchWorkloadState>,
    _: &HashMap<EngineId, Engine>,
) {
}
