use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PongMessage {
    pub redqueen: bool,
}

impl PongMessage {
    pub fn valid(&self) -> bool {
        self.redqueen
    }
}
