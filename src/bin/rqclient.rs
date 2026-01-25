use std::{error::Error, path::Path, process::Command};

fn lookup_branch_name(repo_url: &str, branch_name: &str) -> Result<Option<git2::Oid>, git2::Error> {
    let expected_head = format!("refs/heads/{branch_name}");

    let mut remote = git2::Remote::create_detached(repo_url)?;
    remote.connect(git2::Direction::Fetch)?;
    let heads = remote.list()?;

    let oid = heads
        .iter()
        .find(|head| head.name() == expected_head)
        .map(|head| head.oid());

    Ok(oid)
}

fn build_engine(
    repo_url: &str,
    oid: &str,
    exe_name: &str,
    dest_dir: &Path,
) -> Result<(), std::io::Error> {
    let tmp_dir = tempfile::tempdir()?;

    let clone_out = Command::new("git")
        .arg("clone")
        .args(["--revision", oid])
        .args(["--depth", "1"])
        .arg("--")
        .arg(repo_url)
        .arg(".")
        .current_dir(&tmp_dir)
        .output()?;

    println!("{:?}", clone_out);

    let make_out = Command::new("make")
        .arg("-j")
        .arg(format!("EXE={exe_name}"))
        .current_dir(&tmp_dir)
        .output()?;

    println!("{:?}", make_out);

    std::fs::copy(tmp_dir.path().join(exe_name), dest_dir.join(exe_name))?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let oid = lookup_branch_name("https://github.com/Ciekce/Stormphrax", "main")?.unwrap();

    println!("{}", oid);

    println!(
        "{:?}",
        build_engine(
            "https://github.com/Ciekce/Stormphrax",
            oid.to_string().as_str(),
            &format!("stormphrax-{oid}"),
            &Path::new("/home/lily"),
        )
    );

    Ok(())
}
