use anyhow::Result;
use clap::{Parser, Subcommand};
use futures::StreamExt;
use redqueen::{
    db::Repository,
    domain::{Password, UserId, generate_worker_key_pair},
};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};

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
    /// Miscellaneous utility commands
    #[command(subcommand)]
    Util(UtilCommand),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let db_opts = SqliteConnectOptions::new()
        .filename("rqdatabase.db")
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);
    let db = SqlitePool::connect_with(db_opts).await?;
    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    let repo = Repository::new(db);

    let args = Args::parse();
    match args.cmd {
        Command::User(cmd) => do_user_cmd(&repo, cmd).await,
        Command::Util(cmd) => do_util_cmd(&repo, cmd).await,
    }
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

async fn do_user_cmd(repo: &Repository, cmd: UserCommand) -> Result<()> {
    let bool_to_str = |value| if value { "enabled" } else { "disabled" };
    match cmd {
        UserCommand::List => {
            println!("id\tname\tenabled");
            let mut users = repo.user_get_all();
            let mut count = 0;
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
        UserCommand::Get { username } => match repo.user_get(&username).await {
            Ok(Some(user)) => println!("{:?}", user),
            Ok(None) => println!("Username {username} not found"),
            Err(err) => println!("Failed: {err}"),
        },
        UserCommand::Add { username } => match repo.user_new(&username).await {
            Ok(UserId(id)) => {
                println!("User successfully added with user id {id}");
                println!("To enable user account, use command: rqcli user set-enable {username} true");
                println!("To set user password, use command: rqcli user set-password {username}");
                println!("A user needs to be *both* enabled and have a set password before being able to log in");
                println!("A user starts out with no permissions. To see a list of options, see: rqcli user help");
            }
            Err(err) => println!("Failed: {err}"),
        },
        UserCommand::SetPassword { username } => match repo.user_get(&username).await {
            Ok(Some(user)) => {
                println!("Setting password for username {} (id: {})", user.username, user.id.0);
                let Ok(password1) = rpassword::prompt_password("New password: ") else {
                    println!("Password update aborted");
                    return Ok(());
                };
                let Ok(password2) = rpassword::prompt_password("New password (again): ") else {
                    println!("Password update aborted");
                    return Ok(());
                };
                if password1 != password2 {
                    println!("Passwords do not match");
                    return Ok(());
                }
                let Some(hash) = Password::from_raw_password(&password1) else {
                    println!("Failed to hash password");
                    return Ok(());
                };
                match repo.user_set_password(&username, hash).await {
                    Ok(true) => println!("Password updated for {username}"),
                    Ok(false) => println!("Could not find username {username} when setting password"),
                    Err(err) => println!("Failed: {err}"),
                }
            }
            Ok(None) => println!("Username {username} not found"),
            Err(err) => println!("Failure while retrieving user infomation: {err}"),
        },
        UserCommand::SetEnabled { username, value } => match repo.user_set_enabled(&username, value).await {
            Ok(true) => println!("Successfully {} {username}", bool_to_str(value)),
            Ok(false) => println!("Username {username} not found"),
            Err(err) => println!("Failed: {err}"),
        },
        UserCommand::SetAutoApprove { username, value } => match repo.user_set_auto_approve(&username, value).await {
            Ok(true) => println!("Automatic approval of tests by {username} has been {}", bool_to_str(value)),
            Ok(false) => println!("Username {username} not found"),
            Err(err) => println!("Failed: {err}"),
        },
        UserCommand::SetApprover { username, value } => match repo.user_set_approver(&username, value).await {
            Ok(true) => println!("{username} is {} an approver", if value { "now" } else { "now not" }),
            Ok(false) => println!("Username {username} not found"),
            Err(err) => println!("Failed: {err}"),
        },
        UserCommand::SetAdmin { username, value } => match repo.user_set_admin(&username, value).await {
            Ok(true) => println!("{username} is {} an admin", if value { "now" } else { "now not" }),
            Ok(false) => println!("Username {username} not found"),
            Err(err) => println!("Failed: {err}"),
        },
    }
    Ok(())
}

#[derive(Subcommand)]
enum UtilCommand {
    /// Generate ed25519 key pair
    GenerateKeyPair,
}

async fn do_util_cmd(_repo: &Repository, cmd: UtilCommand) -> Result<()> {
    match cmd {
        UtilCommand::GenerateKeyPair => {
            let (public, private) = generate_worker_key_pair();
            println!("Public Key: {}", public.to_string());
            println!("Private Key: {}", private.to_string());
        }
    }
    Ok(())
}
