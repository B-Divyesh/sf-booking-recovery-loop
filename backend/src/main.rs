mod auth;
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
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use tokio::{signal, sync::Mutex};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorError,
    GovernorLayer,
};
use tower_http::{
    compression::CompressionLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};
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
    pub(crate) entra: auth::EntraValidator,
    pub(crate) integrations: Arc<IntegrationConfig>,
    /// This exists only for isolated integration tests which run a loopback
    /// delivery fixture. Production never honours owner supplied URLs.
    pub(crate) allow_test_delivery_urls: bool,
    static_dir: Arc<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct IntegrationConfig {
    pub(crate) delivery_url: Option<String>,
    pub(crate) delivery_bearer_token: Option<String>,
    pub(crate) delivery_callback_secret: Option<String>,
    pub(crate) billing_base_url: String,
    pub(crate) billing_product_slug: String,
    pub(crate) public_base_url: String,
}

impl IntegrationConfig {
    fn from_environment() -> Self {
        Self {
            delivery_url: env::var("DELIVERY_PROVIDER_URL")
                .ok()
                .filter(|v| !v.is_empty()),
            delivery_bearer_token: env::var("DELIVERY_PROVIDER_TOKEN")
                .ok()
                .filter(|v| !v.is_empty()),
            delivery_callback_secret: env::var("DELIVERY_CALLBACK_SECRET")
                .ok()
                .filter(|v| !v.is_empty()),
            billing_base_url: env::var("SOCIOBOT_BILLING_BASE_URL")
                .unwrap_or_else(|_| "https://api.sociobot.in/api/v1".to_owned()),
            billing_product_slug: env::var("SOCIOBOT_BOOKING_PRODUCT_SLUG")
                .unwrap_or_else(|_| "booking-recovery-loop-deposit".to_owned()),
            public_base_url: env::var("PUBLIC_BASE_URL")
                .unwrap_or_else(|_| "https://booking-recovery-loop.sociobot.in".to_owned()),
        }
    }

    fn delivery_ready(&self) -> bool {
        self.delivery_url.is_some()
            && self.delivery_bearer_token.is_some()
            && self.delivery_callback_secret.is_some()
    }
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
    app_router_with_integrations(
        pool,
        build_sha,
        static_dir,
        encryption_key,
        IntegrationConfig::from_environment(),
    )
}

pub(crate) fn app_router_with_integrations(
    pool: SqlitePool,
    build_sha: impl Into<Arc<str>>,
    static_dir: impl Into<PathBuf>,
    encryption_key: [u8; 32],
    integrations: IntegrationConfig,
) -> Router {
    let static_dir = static_dir.into();
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("HTTP client configuration is valid");
    let state = AppState {
        build_sha: build_sha.into(),
        pool,
        demo_lock: Arc::new(Mutex::new(())),
        encryption_key: Arc::new(encryption_key),
        entra: auth::EntraValidator::from_environment(http.clone()),
        integrations: Arc::new(integrations),
        http,
        allow_test_delivery_urls: cfg!(test)
            || env::var("ALLOW_UNSAFE_TEST_DELIVERY_URLS").ok().as_deref() == Some("1"),
        static_dir: Arc::new(static_dir.clone()),
    };
    app_router_from_state(state)
}

fn app_router_from_state(state: AppState) -> Router {
    let static_dir = state.static_dir.as_ref().clone();

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
            "/practice/delivery/test",
            post(routes::practice::test_delivery_connection),
        )
        .route(
            "/provider/{practice_id}/receipts",
            post(routes::practice::receipt),
        )
        .route(
            "/public/attempts/{attempt_id}/payments/complete",
            post(routes::practice::complete_payment),
        )
        .route("/practice", axum::routing::delete(routes::practice::delete))
        .layer(GovernorLayer::new(write_limit.clone()).error_handler(write_limit_error));
    let practice_read_routes = Router::new()
        .route(
            "/integrations/status",
            get(routes::practice::integration_status),
        )
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

    // This is intentionally outside the read/write route groups: DELETE,
    // callbacks, and every future API method inherit a limiter. Health is the
    // one operational endpoint that stays exempt.
    let api_limit = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .per_millisecond(50)
            .burst_size(GENERAL_BURST)
            .finish()
            .expect("API rate limit must be valid"),
    );
    let api = practice_read_routes
        .merge(practice_write_routes)
        .nest("/demo", demo_api)
        .layer(GovernorLayer::new(api_limit.clone()).error_handler(api_limit_error));

    Router::new()
        .route("/health", get(routes::health::handler))
        .nest("/api/v1", api)
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
        .route("/auth/callback", get(spa_index))
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
                "default-src 'self'; base-uri 'self'; connect-src 'self' https://sociobotcustomers.ciamlogin.com https://api.sociobot.in; font-src 'self'; form-action 'self'; frame-ancestors 'none'; img-src 'self' data:; object-src 'none'; script-src 'self'; style-src 'self'",
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
        .layer(CompressionLayer::new())
}

fn api_limit_error(error: GovernorError) -> Response {
    match error {
        GovernorError::TooManyRequests { wait_time, .. } => {
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
                        GENERAL_BURST.to_string(),
                    ),
                    (
                        HeaderName::from_static("x-ratelimit-remaining"),
                        "0".to_owned(),
                    ),
                ],
                "Too many requests. Try again after the stated delay.",
            )
                .into_response()
        }
        other => Response::from(other),
    }
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

fn runtime_storage_paths(data_dir: &std::path::Path) -> (PathBuf, PathBuf) {
    (
        data_dir.join("booking-recovery-loop.sqlite3"),
        data_dir.join("contact.key"),
    )
}

fn is_transient_sqlite_lock(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("database is locked")
        || message.contains("database table is locked")
        || message.contains("database is busy")
        || message.contains("code: 5")
}

async fn open_runtime_database_with_policy(
    sqlite_path: &std::path::Path,
    attempts: u32,
    busy_timeout: Duration,
    retry_delay: Duration,
) -> Result<SqlitePool, String> {
    let mut last_error = "SQLite initialization was not attempted".to_owned();

    for attempt in 1..=attempts {
        // Azure Files is a network filesystem, so SQLite's default rollback
        // journal is retained. One connection and one application replica
        // serialize writes without a WAL or journal-mode transition.
        let sqlite_options = SqliteConnectOptions::new()
            .filename(sqlite_path)
            .create_if_missing(true)
            .foreign_keys(true)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(busy_timeout);
        let pool = match SqlitePoolOptions::new()
            // A single connection matches the one-replica deployment and
            // avoids competing file locks inside the mounted filesystem.
            .max_connections(1)
            .connect_with(sqlite_options)
            .await
        {
            Ok(pool) => pool,
            Err(error) => {
                last_error = error.to_string();
                if attempt < attempts && is_transient_sqlite_lock(&last_error) {
                    warn!(attempt, "SQLite file is busy during startup; retrying");
                    tokio::time::sleep(retry_delay).await;
                    continue;
                }
                return Err(last_error);
            }
        };

        match migrations::up(&pool).await {
            Ok(()) => return Ok(pool),
            Err(error) => {
                last_error = error.to_string();
                pool.close().await;
                if attempt < attempts && is_transient_sqlite_lock(&last_error) {
                    warn!(
                        attempt,
                        error = %last_error,
                        "SQLite migrations are busy during startup; retrying"
                    );
                    tokio::time::sleep(retry_delay).await;
                    continue;
                }
                return Err(last_error);
            }
        }
    }

    Err(last_error)
}

async fn open_runtime_database(sqlite_path: &std::path::Path) -> Result<SqlitePool, String> {
    open_runtime_database_with_policy(
        sqlite_path,
        30,
        Duration::from_secs(2),
        Duration::from_secs(1),
    )
    .await
}

async fn prepare_data_dir() -> (PathBuf, &'static str) {
    let supplied = env::var_os("BOOKING_DATA_DIR").map(PathBuf::from);
    let requested = supplied
        .clone()
        .unwrap_or_else(|| PathBuf::from("/data/state"));
    if tokio::fs::create_dir_all(&requested).await.is_ok() {
        return (
            requested,
            if supplied.is_some() {
                "supplied"
            } else {
                "default"
            },
        );
    }

    // The production image always contains writable /data and the deployment
    // mounts its durable share there. This fallback keeps a bare local binary
    // usable on hosts where creating /data is not permitted.
    let fallback = env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".booking-recovery-loop-data");
    tokio::fs::create_dir_all(&fallback)
        .await
        .expect("the local data directory must be writable");
    (fallback, "local fallback")
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
    let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| {
        if PathBuf::from("/app/dist/index.html").exists() {
            "/app/dist".to_owned()
        } else {
            "dist".to_owned()
        }
    });
    let (data_dir, data_source) = prepare_data_dir().await;
    let (sqlite_path, key_path) = runtime_storage_paths(&data_dir);
    let pool = open_runtime_database(&sqlite_path)
        .await
        .expect("the durable SQLite file must open and migrate");

    let (encryption_key, key_source) = load_key_from_environment_or_file(&key_path).await;
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let build_sha = env!("BUILD_SHA");
    let integrations = IntegrationConfig::from_environment();
    let delivery_source = if integrations.delivery_ready() {
        "supplied credentialed provider"
    } else {
        "not configured"
    };
    info!(
        port,
        port_source,
        data_source,
        sqlite_path = %sqlite_path.display(),
        key_source,
        delivery_source,
        build_sha,
        "configuration loaded; contact encryption key is persisted without logging its value"
    );

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("the configured HTTP port must be bindable");
    info!(%address, "Booking Recovery Loop API is listening");

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("HTTP client configuration is valid");
    let state = AppState {
        build_sha: Arc::from(build_sha),
        pool,
        demo_lock: Arc::new(Mutex::new(())),
        encryption_key: Arc::new(encryption_key),
        entra: auth::EntraValidator::from_environment(http.clone()),
        http,
        integrations: Arc::new(integrations),
        allow_test_delivery_urls: false,
        static_dir: Arc::new(PathBuf::from(static_dir)),
    };
    let scheduler_state = state.clone();
    let app = app_router_from_state(state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if routes::demo::purge_expired(&scheduler_state.pool)
                .await
                .is_err()
            {
                tracing::warn!("expired demo cleanup could not run");
            }
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

async fn load_key_from_environment_or_file(path: &std::path::Path) -> ([u8; 32], &'static str) {
    if let Some(key) = env::var("CONTACT_ENCRYPTION_KEY")
        .ok()
        .and_then(|value| parse_contact_key(&value))
    {
        return (key, "supplied shared secret");
    }
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

fn parse_contact_key(value: &str) -> Option<[u8; 32]> {
    hex::decode(value)
        .ok()
        .or_else(|| URL_SAFE_NO_PAD.decode(value).ok())
        .and_then(|bytes| <[u8; 32]>::try_from(bytes.as_slice()).ok())
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

#[cfg(test)]
mod tests {
    #[test]
    fn production_state_paths_are_both_under_data() {
        let (sqlite_path, key_path) =
            super::runtime_storage_paths(std::path::Path::new("/data/state"));
        assert_eq!(
            sqlite_path,
            std::path::Path::new("/data/state/booking-recovery-loop.sqlite3")
        );
        assert_eq!(key_path, std::path::Path::new("/data/state/contact.key"));
    }

    #[tokio::test]
    async fn sqlite_state_survives_restart_in_data_directory() {
        let root = std::env::temp_dir().join(format!(
            "booking-recovery-loop-restart-{}",
            uuid::Uuid::now_v7()
        ));
        let data_dir = root.join("data");
        tokio::fs::create_dir_all(&data_dir).await.unwrap();
        let (sqlite_path, _) = super::runtime_storage_paths(&data_dir);
        assert_eq!(sqlite_path.parent(), Some(data_dir.as_path()));

        let options = super::SqliteConnectOptions::new()
            .filename(&sqlite_path)
            .create_if_missing(true);
        let first = super::SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(options.clone())
            .await
            .unwrap();
        crate::migrations::up(&first).await.unwrap();
        sqlx::query(
            "INSERT INTO api_rate_windows (client_key, window_start, hits) VALUES ($1, $2, $3)",
        )
        .bind("restart-proof")
        .bind(123_i64)
        .bind(7_i32)
        .execute(&first)
        .await
        .unwrap();
        first.close().await;

        let second = super::SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(options)
            .await
            .unwrap();
        let hits: i32 = sqlx::query_scalar(
            "SELECT hits FROM api_rate_windows WHERE client_key = $1 AND window_start = $2",
        )
        .bind("restart-proof")
        .bind(123_i64)
        .fetch_one(&second)
        .await
        .unwrap();
        assert_eq!(hits, 7, "state in the data directory must survive restart");
        second.close().await;
        tokio::fs::remove_dir_all(root).await.unwrap();
    }

    #[tokio::test]
    async fn mounted_sqlite_startup_waits_for_a_transient_file_lock() {
        let root = std::env::temp_dir().join(format!(
            "booking-recovery-loop-locked-start-{}",
            uuid::Uuid::now_v7()
        ));
        let data_dir = root.join("data");
        tokio::fs::create_dir_all(&data_dir).await.unwrap();
        let (sqlite_path, _) = super::runtime_storage_paths(&data_dir);

        let locking_pool = super::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                super::SqliteConnectOptions::new()
                    .filename(&sqlite_path)
                    .create_if_missing(true),
            )
            .await
            .unwrap();
        crate::migrations::up(&locking_pool).await.unwrap();
        let mut lock = locking_pool.acquire().await.unwrap();
        sqlx::query("BEGIN EXCLUSIVE")
            .execute(&mut *lock)
            .await
            .unwrap();

        let release_lock = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(120)).await;
            sqlx::query("ROLLBACK").execute(&mut *lock).await.unwrap();
        });
        let reopened = super::open_runtime_database_with_policy(
            &sqlite_path,
            20,
            std::time::Duration::from_millis(20),
            std::time::Duration::from_millis(10),
        )
        .await
        .expect("startup must recover after the previous file lock is released");
        release_lock.await.unwrap();

        let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(&reopened)
            .await
            .unwrap();
        assert_eq!(journal_mode, "delete");
        reopened.close().await;
        locking_pool.close().await;
        tokio::fs::remove_dir_all(root).await.unwrap();
    }
}
