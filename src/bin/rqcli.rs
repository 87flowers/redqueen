#![forbid(unsafe_code)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use futures::{Stream, StreamExt};
use redqueen::keys::{WorkerPublicKey, generate_worker_key_pair};
use redqueen::server::{
    connect_to_repository,
    db::Repository,
    domain::{Password, UserId, Worker, WorkerId},
};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Perform user management
    #[command(subcommand)]
    User(UserCommand),
    /// Perform worker management
    #[command(subcommand)]
    Worker(WorkerCommand),
    /// Miscellaneous utility commands
    #[command(subcommand)]
    Util(UtilCommand),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let repo = connect_to_repository().await?;

    let args = Args::parse();
    match args.cmd {
        Command::User(cmd) => do_user_cmd(&repo, cmd).await,
        Command::Worker(cmd) => do_worker_cmd(&repo, cmd).await,
        Command::Util(cmd) => do_util_cmd(&repo, cmd).await,
    }

    Ok(())
}

#[derive(Subcommand)]
enum UserCommand {
    /// List all users in database
    List,
    /// Get information about a user
    Get { username: String },
    /// Add a new user (User will start disabled, with no password and with no permissions)
    Add { username: String },
    /// Enable/Disable user account (A user needs to be *both* enabled and have a set password in order to login)
    SetEnabled {
        username: String,
        #[arg(action = clap::ArgAction::Set)]
        value: bool,
    },
    /// Set password for account
    SetPassword { username: String },
    /// Enable/Disable automatic approval of tests this user submits
    SetAutoApprove {
        username: String,
        #[arg(action = clap::ArgAction::Set)]
        value: bool,
    },
    /// Allow/Disallow user from approving submitted tests
    SetApprover {
        username: String,
        #[arg(action = clap::ArgAction::Set)]
        value: bool,
    },
    /// Specify that a user account is an admin user
    SetAdmin {
        username: String,
        #[arg(action = clap::ArgAction::Set)]
        value: bool,
    },
}

async fn do_user_cmd(repo: &Repository, cmd: UserCommand) {
    let bool_to_str = |value| if value { "enabled" } else { "disabled" };
    match cmd {
        UserCommand::List => {
            let Ok(mut tx) = repo.begin_read().await else {
                return println!("Failed to start read transaction on database");
            };

            let mut users = tx.user_get_all();
            let mut count = 0;

            println!("id\tname\tenabled");
            while let Some(user) = users.next().await {
                match user {
                    Ok(user) => {
                        println!("{}\t{}\t{}", user.id.0, user.username, user.enabled);
                        count += 1;
                    }
                    Err(err) => println!("Error: {err}"),
                }
            }
            println!("End of user list ({} user(s) found)", count);
        }
        UserCommand::Get { username } => {
            let Ok(mut tx) = repo.begin_read().await else {
                return println!("Failed to start read transaction on database");
            };
            match tx.user_get(&username).await {
                Ok(Some(user)) => println!("{:?}", user),
                Ok(None) => println!("Username {username} not found"),
                Err(err) => println!("Failed: {err}"),
            }
        }
        UserCommand::Add { username } => {
            let Ok(mut tx) = repo.begin_write().await else {
                return println!("Failed to start write transaction on database");
            };
            match tx.user_new(&username).await {
                Ok(UserId(id)) => {
                    println!("User successfully added with user id {id}");
                    println!("To enable user account, use command: rqcli user set-enable {username} true");
                    println!("To set user password, use command: rqcli user set-password {username}");
                    println!("A user needs to be *both* enabled and have a set password before being able to log in");
                    println!("A user starts out with no permissions. To see a list of options, see: rqcli user help");
                }
                Err(err) => println!("Failed: {err}"),
            }
        }
        UserCommand::SetPassword { username } => {
            let Ok(mut tx) = repo.begin_write().await else {
                return println!("Failed to start write transaction on database");
            };
            match tx.user_get(&username).await {
                Ok(Some(user)) => {
                    println!("Setting password for username {} (id: {})", user.username, user.id.0);
                    let Ok(password1) = rpassword::prompt_password("New password: ") else {
                        return println!("Password update aborted");
                    };
                    let Ok(password2) = rpassword::prompt_password("New password (again): ") else {
                        return println!("Password update aborted");
                    };
                    if password1 != password2 {
                        return println!("Passwords do not match");
                    }
                    let Some(hash) = Password::from_raw_password(&password1) else {
                        return println!("Failed to hash password");
                    };
                    match tx.user_set_password(&username, hash).await {
                        Ok(true) => println!("Password updated for {username}"),
                        Ok(false) => println!("Could not find username {username} when setting password"),
                        Err(err) => println!("Failed: {err}"),
                    }
                }
                Ok(None) => println!("Username {username} not found"),
                Err(err) => println!("Failure while retrieving user infomation: {err}"),
            }
        }
        UserCommand::SetEnabled { username, value } => {
            let Ok(mut tx) = repo.begin_write().await else {
                return println!("Failed to start write transaction on database");
            };
            match tx.user_set_enabled(&username, value).await {
                Ok(true) => println!("Successfully {} {username}", bool_to_str(value)),
                Ok(false) => println!("Username {username} not found"),
                Err(err) => println!("Failed: {err}"),
            }
        }
        UserCommand::SetAutoApprove { username, value } => {
            let Ok(mut tx) = repo.begin_write().await else {
                return println!("Failed to start write transaction on database");
            };
            match tx.user_set_auto_approve(&username, value).await {
                Ok(true) => println!("Automatic approval of tests by {username} has been {}", bool_to_str(value)),
                Ok(false) => println!("Username {username} not found"),
                Err(err) => println!("Failed: {err}"),
            }
        }
        UserCommand::SetApprover { username, value } => {
            let Ok(mut tx) = repo.begin_write().await else {
                return println!("Failed to start write transaction on database");
            };
            match tx.user_set_approver(&username, value).await {
                Ok(true) => println!("{username} is {} an approver", if value { "now" } else { "now not" }),
                Ok(false) => println!("Username {username} not found"),
                Err(err) => println!("Failed: {err}"),
            }
        }
        UserCommand::SetAdmin { username, value } => {
            let Ok(mut tx) = repo.begin_write().await else {
                return println!("Failed to start write transaction on database");
            };
            match tx.user_set_admin(&username, value).await {
                Ok(true) => println!("{username} is {} an admin", if value { "now" } else { "now not" }),
                Ok(false) => println!("Username {username} not found"),
                Err(err) => println!("Failed: {err}"),
            }
        }
    }
}

#[derive(Subcommand)]
enum WorkerCommand {
    /// List workers in database
    List { owner_username: Option<String> },
    /// Get information about a worker
    Get { id: i64 },
    /// Add a new worker
    Add { owner_username: String, worker_name: String, worker_public_key: String },
    /// Enable/Disable worker
    SetEnabled {
        id: i64,
        #[arg(action = clap::ArgAction::Set)]
        value: bool,
    },
}

async fn print_worker_list(mut workers: impl Stream<Item = Result<Worker, sqlx::Error>> + Unpin) {
    let mut count = 0;
    println!("id\towner\tname\tenabled");
    while let Some(worker) = workers.next().await {
        match worker {
            Ok(worker) => {
                println!("{}\t{}\t{}\t{}", worker.id.0, worker.owner.0, worker.name, worker.enabled);
                count += 1;
            }
            Err(err) => println!("Error: {err}"),
        }
    }
    println!("End of worker list ({} workers(s) found)", count);
}

async fn do_worker_cmd(repo: &Repository, cmd: WorkerCommand) {
    let bool_to_str = |value| if value { "enabled" } else { "disabled" };
    match cmd {
        WorkerCommand::List { owner_username } => {
            let Ok(mut tx) = repo.begin_read().await else {
                return println!("Failed to start read transaction on database");
            };
            match owner_username {
                Some(owner_username) => {
                    let owner = match tx.user_get(&owner_username).await {
                        Ok(Some(user)) => user.id,
                        Ok(None) => return println!("Username {owner_username} not found"),
                        Err(err) => return println!("Failed to resolve owner username: {err}"),
                    };
                    print_worker_list(tx.worker_owned_by(&owner)).await;
                }
                None => print_worker_list(tx.worker_get_all()).await,
            }
        }
        WorkerCommand::Get { id } => {
            let Ok(mut tx) = repo.begin_read().await else {
                return println!("Failed to start read transaction on database");
            };
            match tx.worker_get(WorkerId(id)).await {
                Ok(Some(worker)) => println!("{:?}", worker),
                Ok(None) => println!("Worker id {id} not found"),
                Err(err) => println!("Failed: {err}"),
            }
        }
        WorkerCommand::Add { owner_username, worker_name, worker_public_key } => {
            let Ok(mut tx) = repo.begin_write().await else {
                return println!("Failed to start write transaction on database");
            };
            let owner = match tx.user_get(&owner_username).await {
                Ok(Some(user)) => user.id,
                Ok(None) => return println!("Username {owner_username} not found"),
                Err(err) => return println!("Failed to resolve owner username : {err}"),
            };
            let key = match WorkerPublicKey::from_str(&worker_public_key) {
                Ok(key) => key,
                Err(err) => return println!("Invalid public key: {err}"),
            };
            match tx.worker_new(owner, &worker_name, key).await {
                Ok(WorkerId(id)) => println!("Worker successfully added with worker id {id}"),
                Err(err) => println!("Failed: {err}"),
            }
        }
        WorkerCommand::SetEnabled { id, value } => {
            let Ok(mut tx) = repo.begin_write().await else {
                return println!("Failed to start write transaction on database");
            };
            match tx.worker_set_enabled(WorkerId(id), value).await {
                Ok(true) => println!("Successfully {} worker id {id}", bool_to_str(value)),
                Ok(false) => println!("Worker id {id} not found"),
                Err(err) => println!("Failed: {err}"),
            }
        }
    }
}

#[derive(Subcommand)]
enum UtilCommand {
    /// Generate ed25519 key pair
    GenerateKeyPair,
}

async fn do_util_cmd(_repo: &Repository, cmd: UtilCommand) {
    match cmd {
        UtilCommand::GenerateKeyPair => {
            let (public, private) = generate_worker_key_pair();
            println!("Public Key: {}", public.to_string());
            println!("Private Key: {}", private.to_string());
        }
    }
}
