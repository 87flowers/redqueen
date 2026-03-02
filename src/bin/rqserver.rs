#![forbid(unsafe_code)]

use anyhow::Result;
use askama::Template;
use axum::{
    Json, Router,
    extract::{Request, State},
    http::{StatusCode, header::CONTENT_LENGTH},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use dashmap::{DashMap, Entry};
use redqueen::{
    common::{
        api::PongMessage,
        domain::{Signature, WorkerPublicKey},
        headers,
        time::unix_time,
    },
    server::{connect_to_repository, db::Repository},
};
use std::{sync::Arc, time::Duration};

struct NonceCache {
    store: DashMap<u64, i64>,
}

impl NonceCache {
    pub fn new() -> Self {
        Self { store: DashMap::new() }
    }

    pub fn hit(&self, nonce: u64) -> bool {
        match self.store.entry(nonce) {
            Entry::Vacant(entry) => {
                entry.insert(unix_time());
                false
            }
            Entry::Occupied(_) => true,
        }
    }

    pub fn cleanup(&self, ttl: i64) {
        let cutoff = unix_time() - ttl;
        self.store.retain(|_, v| *v >= cutoff);
    }
}

struct AppState {
    repo: Repository,
    nonce_cache: NonceCache,
}

const REQUEST_ACCEPTED_CLOCK_SKEW: i64 = 60; // seconds
const NONCE_MIN_TTL: i64 = REQUEST_ACCEPTED_CLOCK_SKEW * 4; // seconds

fn spawn_nonce_cache_cleanup_thread(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(REQUEST_ACCEPTED_CLOCK_SKEW as u64));
        loop {
            ticker.tick().await;
            state.nonce_cache.cleanup(NONCE_MIN_TTL);
        }
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = Arc::new(AppState {
        repo: connect_to_repository().await?,
        nonce_cache: NonceCache::new(),
    });

    spawn_nonce_cache_cleanup_thread(state.clone());

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
        .route("/api/auth_ping", get(handle_get_api_authed_ping))
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
    let current_timestamp = unix_time();

    fn get_header<'req>(req: &'req Request, name: &str) -> Result<&'req str, StatusCode> {
        req.headers().get(name).and_then(|h| h.to_str().ok()).ok_or(StatusCode::UNAUTHORIZED)
    }

    let timestamp = get_header(&req, headers::RQ_TIMESTAMP)?;
    let nonce = get_header(&req, headers::RQ_NONCE)?;
    let body_hash = get_header(&req, headers::RQ_BODY_HASH)?;
    let signature = get_header(&req, headers::RQ_SIGNATURE)?;
    let username = get_header(&req, headers::RQ_USERNAME)?;
    let public_key = get_header(&req, headers::RQ_PUBLIC_KEY)?;
    let content_length = get_header(&req, &CONTENT_LENGTH.to_string())?;

    let timestamp = timestamp.parse::<i64>().map_err(|_| StatusCode::UNAUTHORIZED)?;
    let nonce = nonce.parse::<u64>().map_err(|_| StatusCode::UNAUTHORIZED)?;
    let body_hash: [u8; 64] = const_hex::decode_to_array(body_hash).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let signature = Signature::from_str(signature).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let username = const_hex::decode(username).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let username = str::from_utf8(username.as_slice()).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let public_key = WorkerPublicKey::from_str(public_key).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let content_length = content_length.parse::<usize>().map_err(|_| StatusCode::UNAUTHORIZED)?;

    if (timestamp - current_timestamp).abs() >= REQUEST_ACCEPTED_CLOCK_SKEW {
        // TODO: Tell the client its clock is skewed, and to alert the user.
        return Err(StatusCode::UNAUTHORIZED);
    }

    if state.nonce_cache.hit(nonce) {
        // TODO: If accidental nonce collision, client should retry. Deal with this if this becomes a problem.
        return Err(StatusCode::UNAUTHORIZED);
    }

    let worker = {
        let mut tx = state.repo.begin_read().await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        let Some(owner) = tx.user_get(username).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)? else {
            // TODO: Inform worker its owner does not exist.
            return Err(StatusCode::UNAUTHORIZED);
        };
        let Some(worker) =
            tx.worker_get_by_pubkey(owner.id, public_key).await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        else {
            // TODO: Inform worker it does not exist.
            return Err(StatusCode::UNAUTHORIZED);
        };
        worker
    };

    let Some(public_key) = worker.key else {
        // This should never happen.
        return Err(StatusCode::UNAUTHORIZED);
    };

    let method = req.method();
    let path = req.uri().path();
    let signature_payload =
        format!("RedQueen {method} {path} {content_length} {} {timestamp} {nonce}", const_hex::encode(body_hash));
    if !public_key.verify(signature_payload.as_bytes(), signature) {
        return Err(StatusCode::UNAUTHORIZED);
    }

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
