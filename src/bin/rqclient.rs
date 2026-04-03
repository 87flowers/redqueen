#![forbid(unsafe_code)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use redqueen::{
    client::{
        config::Configuration,
        domain::Remote,
        paths::Paths,
        request::{BodyMeta, build_request},
    },
    common::{
        api::PongMessage,
        domain::{Workload, generate_worker_key_pair},
    },
};
use reqwest::{Method, StatusCode, Url};
use std::{fs, time::Duration};
use tokio::time::sleep;
use toml_edit::{Table, value};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Perform remotes management
    #[command(subcommand)]
    Remote(RemoteCommand),
    /// Run client as configured
    Run,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.cmd {
        Command::Remote(cmd) => do_remote_cmd(cmd).await,
        Command::Run => do_run().await,
    }
    Ok(())
}

#[derive(Subcommand)]
enum RemoteCommand {
    /// Add a remote
    Add { remote_name: String, remote_url: String, username: String },
    /// Ping a remote
    Ping { remote_name: String },
}

async fn do_remote_cmd(cmd: RemoteCommand) {
    match cmd {
        RemoteCommand::Add { remote_name, remote_url, username } => {
            let url = match Url::parse(&remote_url) {
                Ok(url) => url,
                Err(err) => return println!("Failed to parse remote url: {err}"),
            };
            match url.scheme() {
                "https" => {}
                "http" => {
                    println!("WARNING: Using a remote with unencrypted HTTP.");
                    println!("WARNING: We strongly recommend using HTTPS/TLS.");
                }
                _ => return println!("Unrecognised url schema"),
            }
            if url.cannot_be_a_base()
                || url.path() != "/"
                || url.fragment().is_some()
                || url.query().is_some()
                || !url.username().is_empty()
                || url.password().is_some()
            {
                return println!("Remote url failed validation");
            }
            match reqwest::get(url.join("/api/ping").unwrap()).await {
                Err(err) => return println!("Failed to ping remote: {err}"),
                Ok(response) => {
                    if response.status() != StatusCode::OK {
                        return println!("Remote ping resulted in unexpected status code: {}", response.status());
                    }
                    let pong = match response.json::<PongMessage>().await {
                        Err(err) => return println!("Failed to parse remote ping reponse: {err}"),
                        Ok(pong) => pong,
                    };
                    if !pong.valid() {
                        return println!("Invalid pong response");
                    }
                    println!("Successfully pinged remote");
                }
            }

            let config_file_path = Paths::new().config_dir().join("rqclient.conf");
            let mut config = match fs::read_to_string(&config_file_path) {
                Ok(config) => config.parse::<toml_edit::DocumentMut>().expect("Invalid configuration file"),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => toml_edit::DocumentMut::new(),
                Err(err) => return println!("Error reading configuration: {err}"),
            };

            println!("Generating keypair for remote {remote_name} ({remote_url})...");
            let (public_key, private_key) = generate_worker_key_pair();
            println!("Public key: {}", public_key.to_string());
            println!("Execute the following command on the server to register worker:");
            println!("  rqcli worker add <owner username> <worker name> {}", public_key.to_string());
            println!("TODO: Print worker registration URL here when implemented");

            let mut new_remote = toml_edit::Table::new();
            new_remote.insert("url", value(remote_url));
            new_remote.insert("priority", value(0));
            new_remote.insert("username", value(username));
            new_remote.insert("public_key", value(public_key.to_string()));
            new_remote.insert("private_key", value(private_key.to_string()));

            if config.get("remotes").is_none() {
                let mut t = Table::new();
                t.set_implicit(true);
                config["remotes"] = toml_edit::Item::Table(t);
            }
            config["remotes"][remote_name] = toml_edit::Item::Table(new_remote);

            use std::io::Write;

            let config = config.to_string();
            fs::create_dir_all(Paths::new().config_dir()).expect("Failed to create configuration directory");
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(config_file_path)
                .expect("Error opening config file for writing");
            f.write_all(config.as_bytes()).expect("Error writing configuation to file");
            f.flush().expect("Error flushing configuration to file");
        }
        RemoteCommand::Ping { remote_name } => {
            let config_file_path = Paths::new().config_file_path();
            let config: Configuration = match fs::read_to_string(&config_file_path) {
                Ok(config) => toml_edit::de::from_str(&config).expect("Invalid configuration file"),
                Err(err) => return println!("Error reading configuration: {err}"),
            };

            let Some(remote) = config.remotes.get(&remote_name) else {
                return println!("Remote {remote_name} not found in configuration");
            };

            let client = reqwest::Client::new();
            let body = "{}";
            let response = build_request(&client, remote, Method::GET, "/api/auth_ping", BodyMeta::from_str(body))
                .body(body)
                .send()
                .await
                .unwrap();
            println!("{:#?}", response);
        }
    }
}

async fn get_workload() -> Option<(String, Remote, Workload)> {
    let config_file_path = Paths::new().config_file_path();
    let config: Configuration = match fs::read_to_string(&config_file_path) {
        Ok(config) => toml_edit::de::from_str(&config).expect("Invalid configuration file"),
        Err(err) => {
            println!("Error reading configuration: {err}");
            return None;
        }
    };

    let mut remotes: Vec<_> = config.remotes.iter().collect();
    remotes.sort_by_key(|remote| remote.1.priority);

    for (remote_name, remote) in remotes {
        let client = reqwest::Client::new();
        let body = "";
        let response =
            match build_request(&client, remote, Method::GET, "/api/workload/current", BodyMeta::from_str(body))
                .body(body)
                .send()
                .await
            {
                Err(err) => {
                    println!("Error when retrieving current workload from remote {remote_name}: {err}");
                    continue;
                }
                Ok(response) => response,
            };
        if response.status() != StatusCode::OK {
            println!("Unexpected status code from remote {remote_name}: {}", response.status());
            continue;
        }
        let workload = match response.json::<Workload>().await {
            Err(err) => {
                println!("Failed to parse workload from remote {remote_name}: {err}");
                continue;
            }
            Ok(workload) => workload,
        };

        return Some((remote_name.to_string(), remote.clone(), workload));
    }

    None
}

async fn do_run() {
    loop {
        let Some((remote_name, remote, workload)) = get_workload().await else {
            sleep(Duration::from_mins(2)).await;
            continue;
        };

        eprintln!("workload from remote {remote_name} at {}", remote.url);
        eprintln!("{:?}", workload);
        sleep(Duration::from_secs(10)).await;
    }
}
