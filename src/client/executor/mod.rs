use crate::client::domain::Remote;
use crate::common::api::WorkloadMessage;
use crate::common::domain::Workload;

mod bench;
mod common;

pub fn execute_workload(remote_name: &str, remote: Remote, msg: WorkloadMessage) {
    match msg.workload {
        Workload::Bench { id, config, state } => {
            bench::execute(remote_name, remote, id, config, state, &msg.related_engines)
        }
    }
}
