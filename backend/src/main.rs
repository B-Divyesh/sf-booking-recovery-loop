mod auth;
mod migrations;
mod routes;

use std::{collections::HashMap, env, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use axum::{
    extract::{Request, State},
    http::{header, HeaderName, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use sqlx::{any::AnyPoolOptions, AnyPool};
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
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const GENERAL_BURST: u32 = 40;
const WRITE_BURST: u32 = 12;
const WRITE_REPLENISH_SECONDS: u64 = 60;
const SHARED_API_REQUESTS_PER_SECOND: i32 = 40;
const SHARED_WRITE_REQUESTS_PER_MINUTE: i32 = 12;
const READ_RATE_RESERVATION: i32 = 4;

#[derive(Clone, Copy)]
pub(crate) struct LocalRateWindow {
    window_start: i64,
    remaining: i32,
    exhausted: bool,
}

fn database_pool_max_connections(database_url: &str) -> u32 {
    // The factory's shared PgBouncer session pool has a 15-client ceiling
    // shared with release work and operational connections. Three serving
    // replicas therefore get two connections each, leaving enough headroom
    // that a valid global limiter burst cannot turn into EMAXCONNSESSION 503s.
    if database_url.starts_with("sqlite:") {
        1
    } else {
        2
    }
}

fn postgres_schema_setup_statements() -> [&'static str; 2] {
    [
        "CREATE SCHEMA IF NOT EXISTS booking_recovery_loop",
        "SET search_path TO booking_recovery_loop, public",
    ]
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) build_sha: Arc<str>,
    pub(crate) pool: AnyPool,
    pub(crate) demo_lock: Arc<Mutex<()>>,
    pub(crate) rate_windows: Arc<Mutex<HashMap<String, LocalRateWindow>>>,
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
    pool: AnyPool,
    build_sha: impl Into<Arc<str>>,
    static_dir: impl Into<PathBuf>,
) -> Router {
    app_router_with_key(pool, build_sha, static_dir, [7_u8; 32])
}

pub(crate) fn app_router_with_key(
    pool: AnyPool,
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
    pool: AnyPool,
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
        rate_windows: Arc::new(Mutex::new(HashMap::new())),
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
        .layer(middleware::from_fn_with_state(
            state.clone(),
            shared_api_rate_limit,
        ))
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

/// The governor above protects a single process from bursts. This database
/// counter is the authoritative allowance across replicas; it is deliberately
/// before every versioned API route and leaves only /health outside the policy.
async fn shared_api_rate_limit(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let client = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown");
    let now = chrono::Utc::now().timestamp();
    let is_write = request.method() == Method::POST;
    // Writes deliberately use a minute bucket. The value is authoritative in
    // PostgreSQL, so changing HTTP connections or replicas cannot multiply a
    // 12-write allowance. Reads retain the short 40-request burst window.
    let (window_start, key, limit, retry_after) = if is_write {
        (
            now - now.rem_euclid(WRITE_REPLENISH_SECONDS as i64),
            format!("write:{client}"),
            SHARED_WRITE_REQUESTS_PER_MINUTE,
            WRITE_REPLENISH_SECONDS.to_string(),
        )
    } else {
        (
            now,
            format!("read:{client}"),
            SHARED_API_REQUESTS_PER_SECOND,
            "1".to_owned(),
        )
    };
    let local_key = format!("{key}:{window_start}");
    let reservation = if is_write { 1 } else { READ_RATE_RESERVATION };
    // A small local reservation turns a large burst into at most ten shared
    // database updates rather than one update per request. The database still
    // grants every block atomically, so independent replicas cannot exceed
    // the global 40-read/12-write allowance. Holding this short per-process
    // lock covers only a quota reservation, never application work.
    let allowed = {
        let mut local_windows = state.rate_windows.lock().await;
        local_windows.retain(|_, value| value.window_start >= now - 2 * 60);
        if let Some(window) = local_windows.get_mut(&local_key) {
            if window.remaining > 0 {
                window.remaining -= 1;
                Ok(true)
            } else if window.exhausted {
                Ok(false)
            } else {
                reserve_rate_block(&state.pool, &key, window_start, reservation, limit)
                    .await
                    .map(|granted| {
                        *window = LocalRateWindow {
                            window_start,
                            remaining: granted.saturating_sub(1),
                            exhausted: granted == 0,
                        };
                        granted > 0
                    })
            }
        } else {
            reserve_rate_block(&state.pool, &key, window_start, reservation, limit)
                .await
                .map(|granted| {
                    local_windows.insert(
                        local_key,
                        LocalRateWindow {
                            window_start,
                            remaining: granted.saturating_sub(1),
                            exhausted: granted == 0,
                        },
                    );
                    granted > 0
                })
        }
    };
    match allowed {
        Ok(true) => next.run(request).await,
        Ok(false) => (
            StatusCode::TOO_MANY_REQUESTS,
            [
                (header::RETRY_AFTER, retry_after.as_str()),
                (
                    HeaderName::from_static("x-ratelimit-after"),
                    retry_after.as_str(),
                ),
                (
                    HeaderName::from_static("x-ratelimit-limit"),
                    if is_write { "12" } else { "40" },
                ),
                (HeaderName::from_static("x-ratelimit-remaining"), "0"),
            ],
            "Too many requests. Try again after the stated delay.",
        )
            .into_response(),
        // Failing open would permit unlimited deletes when shared storage is
        // unhealthy. A 503 is safer and tells callers to retry.
        Err(error) => {
            tracing::error!(%error, client, "shared API rate limit database update failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                [(header::RETRY_AFTER, "1")],
                "Request protection is temporarily unavailable. Try again shortly.",
            )
                .into_response()
        }
    }
}

async fn reserve_rate_block(
    pool: &AnyPool,
    key: &str,
    window_start: i64,
    reservation: i32,
    limit: i32,
) -> Result<i32, sqlx::Error> {
    // PostgreSQL stores this column as INTEGER (i32), while SQLite's dynamic
    // integer type had masked the wrong i64 decoder in local-only testing.
    // `$1` parameters work with both PostgreSQL and SQLite. `AnyPool` selects
    // a driver but does not rewrite `?` placeholders for PostgreSQL.
    sqlx::query_scalar::<_, i32>(
        "INSERT INTO api_rate_windows (client_key, window_start, hits) VALUES ($1, $2, $3) \
         ON CONFLICT (client_key, window_start) DO UPDATE SET hits = api_rate_windows.hits + $3 \
         WHERE api_rate_windows.hits <= $4 \
         RETURNING hits",
    )
    .bind(key)
    .bind(window_start)
    .bind(reservation)
    .bind(limit - reservation)
    .fetch_optional(pool)
    .await
    .map(|hits| hits.map(|_| reservation).unwrap_or_default())
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
    sqlx::any::install_default_drivers();
    let database_source = if env::var("DATABASE_URL").is_ok() {
        "supplied"
    } else {
        "generated default"
    };
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:///data/booking-recovery-loop.db?mode=rwc".to_owned());
    if env::var("REQUIRE_SHARED_DATABASE").ok().as_deref() == Some("1")
        && !database_url.starts_with("postgres")
    {
        panic!("REQUIRE_SHARED_DATABASE=1 requires a PostgreSQL DATABASE_URL");
    }
    let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| {
        if PathBuf::from("/app/dist/index.html").exists() {
            "/app/dist".to_owned()
        } else {
            "dist".to_owned()
        }
    });
    let uses_postgres = database_url.starts_with("postgres");
    let pool = AnyPoolOptions::new()
        .max_connections(database_pool_max_connections(&database_url))
        .after_connect(move |connection, _| {
            Box::pin(async move {
                // PgBouncer may hand a different physical connection to each
                // request. Set the tenant schema on every acquired PostgreSQL
                // connection instead of relying on a startup URL option. The
                // shared factory database has a populated public schema for
                // other products, so create this product's isolated schema
                // before selecting it; PostgreSQL otherwise falls back to
                // public and SQLx sees another product's migration history.
                if uses_postgres {
                    for statement in postgres_schema_setup_statements() {
                        sqlx::query(statement).execute(&mut *connection).await?;
                    }
                }
                Ok(())
            })
        })
        .connect(&database_url)
        .await
        .expect("the configured shared database must open");
    // sqlx's PostgreSQL migrator holds a session advisory lock. The factory's
    // managed URL is PgBouncer transaction pooling, where a subsequent
    // unlock can be assigned a different physical connection and leave that
    // lock behind. Migrations are therefore a one-shot release operation for
    // the shared database (run with RUN_MIGRATIONS=1), never a replica startup
    // operation. SQLite remains self-starting for the no-config local path.
    let run_migrations = !uses_postgres || env::var("RUN_MIGRATIONS").ok().as_deref() == Some("1");
    if run_migrations {
        migrations::up(&pool)
            .await
            .expect("the demo database migration must apply");
    } else {
        info!("shared PostgreSQL migrations are managed by the release job");
    }

    let key_path = env::var("CONTACT_KEY_FILE").unwrap_or_else(|_| "/data/contact.key".to_owned());
    let (encryption_key, key_source) =
        load_key_from_environment_or_file(std::path::Path::new(&key_path)).await;
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
        database_source,
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
        rate_windows: Arc::new(Mutex::new(HashMap::new())),
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
    fn postgres_connections_create_and_select_the_product_schema_before_migrating() {
        assert_eq!(
            super::postgres_schema_setup_statements(),
            [
                "CREATE SCHEMA IF NOT EXISTS booking_recovery_loop",
                "SET search_path TO booking_recovery_loop, public",
            ]
        );
    }

    #[test]
    fn postgres_replica_pools_leave_operational_headroom_in_the_shared_pooler() {
        assert_eq!(super::database_pool_max_connections("sqlite::memory:"), 1);
        assert_eq!(
            super::database_pool_max_connections("postgresql://example.test/postgres"),
            2
        );
        assert!(
            super::database_pool_max_connections("postgresql://example.test/postgres") * 3 + 3 < 15,
            "three replicas must leave room for release and operational connections"
        );
    }
}
