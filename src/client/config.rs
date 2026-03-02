use crate::client::domain::Remote;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub remotes: HashMap<String, Remote>,
}
