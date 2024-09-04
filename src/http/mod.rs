pub mod email;
pub mod error;
pub mod types;

use anyhow::{Context, Ok};
use axum::{
    error_handling::HandleErrorLayer,
    extract::DefaultBodyLimit,
    http::{Method, StatusCode, Uri},
    response::Redirect,
    BoxError, Router,
};

use axum_session::{SessionConfig, SessionLayer, SessionStore};
use axum_session_sqlx::SessionPgPool;
use sqlx::postgres::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing;

use crate::config::WebConfig;

pub type Result<T, E = error::AppError> = std::result::Result<T, E>;

#[derive(Clone)]
pub struct WebContext {
    pub config: Arc<WebConfig>,
    pub db: PgPool,
}

#[derive(Clone)]
struct AppState {}

pub async fn serve(config: WebConfig, db: PgPool, port: u16) -> anyhow::Result<()> {
    let session_config = SessionConfig::default()
        .with_table_name("sessions")
        .with_max_age(None)
        .with_session_name("huxifun");

    let session_store = SessionStore::<SessionPgPool>::new(Some(db.clone().into()), session_config)
        .await
        .unwrap();

    let ctx = WebContext {
        config: Arc::new(config),
        db,
    };

    let app = Router::<WebContext>::new()
        .merge(crate::fun::router())
        .merge(crate::fun::hu::router())
        .merge(crate::fun::book::router())
        .merge(crate::fun::xi::router())
        .merge(crate::fun::user::router())
        .merge(crate::fun::message::router())
        .merge(crate::fun::image::router())
        .merge(crate::fun::comment::router())
        .nest_service("/css", ServeDir::new("htdocs/css"))
        .nest_service("/js", ServeDir::new("htdocs/js"))
        .nest_service("/img", ServeDir::new("htdocs/img"))
        .with_state(ctx)
        .layer(SessionLayer::new(session_store))
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(1024 * 1000))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(30)),
        );

    let app = app.fallback(handler_404);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::debug!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn handle_timeout_error(
    method: Method,
    uri: Uri,
    // the last argument must be the error itself
    err: BoxError,
) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("`{} {}` failed with {}", method, uri, err),
    )
}

async fn handler_404() -> Redirect {
    Redirect::temporary("/error/404")
}
