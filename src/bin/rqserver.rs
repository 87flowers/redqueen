#![forbid(unsafe_code)]

use anyhow::Result;
use askama::Template;
use axum::{
    Json, Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use redqueen::{
    common::api::PongMessage,
    server::{connect_to_repository, db::Repository},
};
use std::sync::Arc;

struct AppState {
    repo: Repository,
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = Arc::new(AppState { repo: connect_to_repository().await? });

    let app = Router::new()
        .route("/", get(handler))
        .route("/login", get(login_get_handler))
        .merge(api_routes(state.clone()))
        .fallback(|| async { AppError::NotFound });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn api_routes(state: Arc<AppState>) -> Router {
    let unauth_api = Router::new().route("/api/ping", get(handle_get_api_ping)).with_state(state.clone());

    let auth_api = Router::new()
        .route("/api/authed_ping", get(handle_get_api_authed_ping))
        .route_layer(middleware::from_fn_with_state(state.clone(), api_authentication))
        .with_state(state.clone());

    Router::new().merge(unauth_api).merge(auth_api)
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

async fn handle_get_api_ping() -> Json<PongMessage> {
    Json(PongMessage { redqueen: true })
}

async fn handle_get_api_authed_ping() -> Json<PongMessage> {
    Json(PongMessage { redqueen: true })
}

async fn api_authentication(
    State(state): State<Arc<AppState>>, req: Request, next: Next,
) -> Result<Response, StatusCode> {
    let authorization_header =
        req.headers().get("Authorization").and_then(|h| h.to_str().ok()).ok_or(StatusCode::UNAUTHORIZED)?;
    let authorization_header = authorization_header.split_whitespace();

    let Ok(["RedQueen", username, public_key, time_stamp, nonce, signature]): Result<[&str; 6], _> =
        authorization_header.collect::<Vec<_>>().try_into()
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // state.repo.worker_get_by_pubkey(username, public_key);

    Ok(next.run(req).await)
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
