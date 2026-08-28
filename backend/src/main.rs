mod routes;

use std::{env, net::SocketAddr, sync::Arc};

use axum::{routing::get, Router};
use tokio::signal;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) build_sha: Arc<str>,
}

pub(crate) fn app_router(build_sha: impl Into<Arc<str>>) -> Router {
    let state = AppState {
        build_sha: build_sha.into(),
    };

    Router::new()
        .route("/health", get(routes::health::handler))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "booking_recovery_loop_api=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let port_source = if env::var("PORT").is_ok() {
        "supplied"
    } else {
        "default"
    };
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let build_sha = env!("BUILD_SHA");

    info!(
        port,
        port_source,
        build_sha,
        "configuration: PORT is supplied/default; no secret configuration is required by the foundation"
    );

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("the configured HTTP port must be bindable");
    info!(%address, "health-only API foundation is listening");

    axum::serve(listener, app_router(build_sha))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("the API server should shut down cleanly");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Ctrl+C signal handler should install");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("SIGTERM handler should install")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("shutdown signal received");
}
