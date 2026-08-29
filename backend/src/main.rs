mod migrations;
mod routes;

use std::{env, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use axum::{
    extract::State,
    http::{header, HeaderName, HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    SqlitePool,
};
use tokio::{signal, sync::Mutex};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorError,
    GovernorLayer,
};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const GENERAL_BURST: u32 = 40;
const WRITE_BURST: u32 = 12;
const WRITE_REPLENISH_SECONDS: u64 = 60;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) build_sha: Arc<str>,
    pub(crate) pool: SqlitePool,
    pub(crate) demo_lock: Arc<Mutex<()>>,
    pub(crate) encryption_key: Arc<[u8; 32]>,
    pub(crate) http: reqwest::Client,
    static_dir: Arc<PathBuf>,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn app_router(
    pool: SqlitePool,
    build_sha: impl Into<Arc<str>>,
    static_dir: impl Into<PathBuf>,
) -> Router {
    app_router_with_key(pool, build_sha, static_dir, [7_u8; 32])
}

pub(crate) fn app_router_with_key(
    pool: SqlitePool,
    build_sha: impl Into<Arc<str>>,
    static_dir: impl Into<PathBuf>,
    encryption_key: [u8; 32],
) -> Router {
    let static_dir = static_dir.into();
    let state = AppState {
        build_sha: build_sha.into(),
        pool,
        demo_lock: Arc::new(Mutex::new(())),
        encryption_key: Arc::new(encryption_key),
        http: reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("HTTP client configuration is valid"),
        static_dir: Arc::new(static_dir.clone()),
    };

    let mut general_builder = GovernorConfigBuilder::default().key_extractor(SmartIpKeyExtractor);
    general_builder
        .per_millisecond(50)
        .burst_size(GENERAL_BURST)
        .methods(vec![Method::GET]);
    let general_limit = Arc::new(
        general_builder
            .use_headers()
            .finish()
            .expect("general rate limit must be valid"),
    );

    let mut write_builder = GovernorConfigBuilder::default().key_extractor(SmartIpKeyExtractor);
    write_builder
        .per_second(WRITE_REPLENISH_SECONDS)
        .burst_size(WRITE_BURST)
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
        .layer(GovernorLayer::new(write_limit.clone()).error_handler(write_limit_error));

    let read_routes = Router::new()
        .route("/workspace", get(routes::demo::show))
        .layer(GovernorLayer::new(general_limit.clone()));
    let demo_api = read_routes.merge(write_routes);

    let practice_write_routes = Router::new()
        .route("/practices", post(routes::practice::create))
        .route(
            "/public/{slug}/attempts",
            post(routes::practice::create_attempt),
        )
        .route(
            "/practice/attempts/{attempt_id}/recover",
            post(routes::practice::recover),
        )
        .route(
            "/provider/{practice_id}/receipts",
            post(routes::practice::receipt),
        )
        .route(
            "/provider/{practice_id}/payments",
            post(routes::practice::payment),
        )
        .route("/practice", axum::routing::delete(routes::practice::delete))
        .layer(GovernorLayer::new(write_limit.clone()).error_handler(write_limit_error));
    let practice_read_routes = Router::new()
        .route("/practice", get(routes::practice::show))
        .route("/practice/export", get(routes::practice::export))
        .route("/public/{slug}", get(routes::practice::public_show))
        .layer(GovernorLayer::new(general_limit.clone()));

    let immutable_assets = Router::new()
        .nest_service("/assets", ServeDir::new(static_dir.join("assets")))
        .nest_service("/fonts", ServeDir::new(static_dir.join("fonts")))
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        ));

    Router::new()
        .route("/health", get(routes::health::handler))
        .nest("/api/v1/demo", demo_api)
        .nest("/api/v1", practice_read_routes.merge(practice_write_routes))
        .merge(immutable_assets)
        .route_service("/robots.txt", ServeFile::new(static_dir.join("robots.txt")))
        .route_service("/sitemap.xml", ServeFile::new(static_dir.join("sitemap.xml")))
        .route_service("/favicon.svg", ServeFile::new(static_dir.join("favicon.svg")))
        .route_service(
            "/apple-touch-icon.png",
            ServeFile::new(static_dir.join("apple-touch-icon.png")),
        )
        .route_service(
            "/social-card.png",
            ServeFile::new(static_dir.join("social-card.png")),
        )
        .route("/", get(spa_index))
        .route("/demo", get(spa_index))
        .route("/privacy", get(spa_index))
        .route("/terms", get(spa_index))
        .route("/start", get(spa_index))
        .route("/app", get(spa_index))
        .route("/app/settings/data", get(spa_index))
        .route("/b/{slug}", get(spa_index))
        .route("/b/{slug}/complete", get(spa_index))
        .route("/404", get(spa_index))
        .fallback(not_found)
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
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
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

fn write_limit_error(error: GovernorError) -> Response {
    match error {
        GovernorError::TooManyRequests { wait_time, .. } => {
            // tower-governor reports whole elapsed seconds by rounding down. HTTP
            // Retry-After must not invite the client to retry before a token exists.
            let retry_after = wait_time.saturating_add(1).max(1);
            (
                StatusCode::TOO_MANY_REQUESTS,
                [
                    (header::RETRY_AFTER, retry_after.to_string()),
                    (
                        HeaderName::from_static("x-ratelimit-after"),
                        retry_after.to_string(),
                    ),
                    (
                        HeaderName::from_static("x-ratelimit-limit"),
                        WRITE_BURST.to_string(),
                    ),
                    (
                        HeaderName::from_static("x-ratelimit-remaining"),
                        "0".to_owned(),
                    ),
                ],
                "Too many sample writes. Try again after the stated delay.",
            )
                .into_response()
        }
        other => Response::from(other),
    }
}

async fn spa_index(State(state): State<AppState>) -> Response {
    file_response(&state.static_dir.join("index.html"), StatusCode::OK).await
}

async fn not_found(State(state): State<AppState>) -> Response {
    file_response(&state.static_dir.join("index.html"), StatusCode::NOT_FOUND).await
}

async fn file_response(path: &std::path::Path, status: StatusCode) -> Response {
    match tokio::fs::read_to_string(path).await {
        Ok(body) => (status, [(header::CACHE_CONTROL, "no-cache")], Html(body)).into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "The web application is not available.",
        )
            .into_response(),
    }
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
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePool::connect_with(options)
        .await
        .expect("the demo database must open");
    migrations::up(&pool)
        .await
        .expect("the demo database migration must apply");

    let key_path = env::var("CONTACT_KEY_FILE").unwrap_or_else(|_| "/data/contact.key".to_owned());
    let (encryption_key, key_source) = load_or_create_key(std::path::Path::new(&key_path)).await;
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let build_sha = env!("BUILD_SHA");
    info!(
        port,
        port_source,
        database_source,
        key_source,
        build_sha,
        "configuration loaded; contact encryption key is persisted without logging its value"
    );

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("the configured HTTP port must be bindable");
    info!(%address, "Booking Recovery Loop API is listening");

    let app = app_router_with_key(pool.clone(), build_sha, static_dir, encryption_key);
    let scheduler_state = AppState {
        build_sha: Arc::from(env!("BUILD_SHA")),
        pool,
        demo_lock: Arc::new(Mutex::new(())),
        encryption_key: Arc::new(encryption_key),
        http: reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("HTTP client configuration is valid"),
        static_dir: Arc::new(PathBuf::new()),
    };
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if routes::practice::run_due_jobs(&scheduler_state, chrono::Utc::now().timestamp())
                .await
                .is_err()
            {
                tracing::warn!("scheduled recovery loop could not process due jobs");
            }
        }
    });
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("the API server should shut down cleanly");
}

async fn load_or_create_key(path: &std::path::Path) -> ([u8; 32], &'static str) {
    if let Ok(bytes) = tokio::fs::read(path).await {
        if let Ok(key) = <[u8; 32]>::try_from(bytes.as_slice()) {
            return (key, "persisted");
        }
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .expect("contact key directory must be writable");
    }
    let mut key = [0_u8; 32];
    getrandom::fill(&mut key).expect("operating system random source must be available");
    tokio::fs::write(path, key)
        .await
        .expect("contact encryption key must be persisted");
    (key, "generated")
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
