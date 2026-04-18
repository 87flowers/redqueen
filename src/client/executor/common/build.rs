use crate::{client::paths::Paths, common::domain::EngineBranch};
use std::{io::Write, path::PathBuf};
use thiserror::Error;
use tokio::process::Command;

#[derive(Error, Debug)]
pub enum BuildEngineError {
    #[error("I/O Error")]
    IoError(#[from] std::io::Error),
}

pub async fn build_engine(branch: EngineBranch) -> Result<PathBuf, BuildEngineError> {
    let engine_path = Paths::new().engine_executable(&branch);
    let executable_name = engine_path.file_name().unwrap();

    let tmp_dir = tempfile::tempdir()?;

    let mut out = std::io::stdout();

    let clone_out = Command::new("git")
        .arg("clone")
        .args(["--revision", &branch.commit_hash])
        .args(["--depth", "1"])
        .arg("--")
        .arg(&branch.url)
        .arg(".")
        .current_dir(&tmp_dir)
        .output()
        .await?;

    out.write_all(&clone_out.stdout)?;
    out.flush()?;

    let make_out = Command::new("make")
        .arg("-j")
        .arg(format!("EXE={}", executable_name.to_str().unwrap()))
        .current_dir(&tmp_dir)
        .output()
        .await?;

    out.write_all(&make_out.stdout)?;
    out.flush()?;

    Ok(engine_path)
}
