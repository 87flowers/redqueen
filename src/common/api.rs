use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::common::domain::{Engine, EngineId, Workload};

#[derive(Serialize, Deserialize)]
pub struct PongMessage {
    pub redqueen: bool,
}

impl PongMessage {
    pub fn valid(&self) -> bool {
        self.redqueen
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkloadMessage {
    pub workload: Workload,
    pub related_engines: HashMap<EngineId, Engine>,
}
