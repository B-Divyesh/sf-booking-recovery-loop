mod migrations;
mod routes;

use std::{env, net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    http::{header, HeaderName, HeaderValue, Method},
    routing::{get, post},
    Router,
};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tokio::signal;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) build_sha: Arc<str>,
    pub(crate) pool: SqlitePool,
}

pub(crate) fn app_router(
    pool: SqlitePool,
    build_sha: impl Into<Arc<str>>,
    static_dir: impl Into<PathBuf>,
) -> Router {
    let state = AppState {
        build_sha: build_sha.into(),
        pool,
    };
    let static_dir = static_dir.into();
    let index = static_dir.join("index.html");

    let mut general_builder = GovernorConfigBuilder::default().key_extractor(SmartIpKeyExtractor);
    general_builder.per_millisecond(50).burst_size(40);
    let general_limit = Arc::new(
        general_builder
            .use_headers()
            .finish()
            .expect("general rate limit must be valid"),
    );

    let mut write_builder = GovernorConfigBuilder::default().key_extractor(SmartIpKeyExtractor);
    write_builder
        .per_millisecond(200)
        .burst_size(12)
        .methods(vec![Method::POST]);
    let write_limit = Arc::new(
        write_builder
            .use_headers()
            .finish()
            .expect("write rate limit must be valid"),
    );

    let write_routes = Router::new()
        .route("/workspaces", post(routes::demo::create))
        .route("/reset", post(routes::demo::reset))
        .route(
            "/attempts/{attempt_id}/recover",
            post(routes::demo::recover),
        )
        .layer(GovernorLayer::new(write_limit));

    let demo_api = Router::new()
        .route("/workspace", get(routes::demo::show))
        .merge(write_routes)
        .layer(GovernorLayer::new(general_limit));

    Router::new()
        .route("/health", get(routes::health::handler))
        .nest("/api/v1/demo", demo_api)
        .fallback_service(ServeDir::new(&static_dir).fallback(ServeFile::new(index)))
        .with_state(state)
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; base-uri 'self'; connect-src 'self'; font-src 'self'; form-action 'self'; frame-ancestors 'none'; img-src 'self' data:; object-src 'none'; script-src 'self'; style-src 'self'",
            ),
        ))
        .layer(PropagateRequestIdLayer::new(HeaderName::from_static(
            "x-request-id",
        )))
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::new(
            HeaderName::from_static("x-request-id"),
            MakeRequestUuid,
        ))
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
    let database_source = if env::var("DATABASE_URL").is_ok() {
        "supplied"
    } else {
        "generated default"
    };
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://booking-recovery-loop.db".to_owned());
    let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| {
        if PathBuf::from("/app/dist/index.html").exists() {
            "/app/dist".to_owned()
        } else {
            "dist".to_owned()
        }
    });
    let options = database_url
        .parse::<SqliteConnectOptions>()
        .expect("DATABASE_URL must be a valid SQLite URL")
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePool::connect_with(options)
        .await
        .expect("the demo database must open");
    migrations::up(&pool)
        .await
        .expect("the demo database migration must apply");

    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let build_sha = env!("BUILD_SHA");
    info!(
        port,
        port_source,
        database_source,
        build_sha,
        "configuration loaded; no secret configuration is required for M1"
    );

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("the configured HTTP port must be bindable");
    info!(%address, "Booking Recovery Loop API is listening");

    axum::serve(
        listener,
        app_router(pool, build_sha, static_dir).into_make_service_with_connect_info::<SocketAddr>(),
    )
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
