use super::Repository;
use crate::domain::{Worker, WorkerId};

impl Repository {
    pub async fn worker_get(&self, id: WorkerId) -> Result<Option<Worker>, sqlx::Error> {
        //let res = sqlx
        todo!()
    }
}
