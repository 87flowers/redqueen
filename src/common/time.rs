use std::time::{SystemTime, UNIX_EPOCH};

pub fn unix_time() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time travel is impossible").as_secs() as i64
}
