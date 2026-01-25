use askama::Template;
use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};

type Db = sqlx::SqlitePool;
type Result<T, E = crate::AppError> = std::result::Result<T, E>;

#[derive(Clone)]
struct AppState {
    db: Db,
}

#[tokio::main]
async fn main() -> Result<()> {
    let db_opts = SqliteConnectOptions::new()
        .filename("rqdatabase.db")
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);
    let db = Db::connect_with(db_opts).await?;
    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    let state = AppState { db };

    let app = Router::new()
        .route("/", get(handler))
        .route("/login", get(login_get_handler))
        .fallback(|| async { AppError::NotFound })
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn handler() -> Response {
    Html("<h1>Hello, World!</h1>").into_response()
}

async fn login_get_handler() -> Result<Response, AppError> {
    #[derive(Template)]
    #[template(path = "login.html")]
    struct Tmpl {}
    let template = Tmpl {};
    Ok(Html(template.render()?).into_response())
}

#[derive(Debug, displaydoc::Display, thiserror::Error)]
enum AppError {
    /// not found
    NotFound,
    /// could not render template
    Render(#[from] askama::Error),
    /// database error
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        #[derive(Template)]
        #[template(path = "error.html")]
        struct Tmpl {
            err: AppError,
        }
        let tmpl = Tmpl { err: self };

        if let Ok(body) = tmpl.render() {
            (status, Html(body)).into_response()
        } else {
            (status, "Something went wrong").into_response()
        }
    }
}
