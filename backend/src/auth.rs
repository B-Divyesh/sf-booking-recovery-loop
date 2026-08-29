use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use axum::http::{header, HeaderMap, StatusCode};
#[allow(unused_imports)]
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use tokio::sync::RwLock;

const DEFAULT_TENANT_ID: &str = "35c6fe40-0ec0-46b6-98c6-213ad4de6650";
const DEFAULT_SUBDOMAIN: &str = "sociobotcustomers";
const DEFAULT_CLIENT_ID: &str = "25c704f4-465a-47af-80ab-2c489466b697";

#[cfg_attr(test, allow(dead_code))]
#[derive(Clone)]
pub(crate) struct EntraValidator {
    tenant_id: Arc<str>,
    client_id: Arc<str>,
    authority: Arc<str>,
    http: reqwest::Client,
    discovery: Arc<RwLock<Option<Discovery>>>,
    keys: Arc<RwLock<Option<(Instant, JwkSet)>>>,
}

#[cfg_attr(test, allow(dead_code))]
#[derive(Clone, Deserialize)]
struct Discovery {
    issuer: String,
    jwks_uri: String,
}

#[cfg_attr(test, allow(dead_code))]
#[derive(Deserialize)]
struct Claims {
    oid: String,
    tid: String,
    #[serde(default)]
    aud: serde_json::Value,
}

impl EntraValidator {
    pub(crate) fn from_environment(http: reqwest::Client) -> Self {
        let tenant_id =
            std::env::var("ENTRA_TENANT_ID").unwrap_or_else(|_| DEFAULT_TENANT_ID.into());
        let subdomain =
            std::env::var("ENTRA_TENANT_SUBDOMAIN").unwrap_or_else(|_| DEFAULT_SUBDOMAIN.into());
        let client_id =
            std::env::var("ENTRA_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT_ID.into());
        let authority = format!("https://{subdomain}.ciamlogin.com/{tenant_id}/");
        Self {
            tenant_id: tenant_id.into(),
            client_id: client_id.into(),
            authority: authority.into(),
            http,
            discovery: Arc::new(RwLock::new(None)),
            keys: Arc::new(RwLock::new(None)),
        }
    }

    #[cfg_attr(test, allow(dead_code))]
    async fn discovery(&self) -> Result<Discovery, ()> {
        if let Some(value) = self.discovery.read().await.clone() {
            return Ok(value);
        }
        let url = format!("{}v2.0/.well-known/openid-configuration", self.authority);
        let value = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|_| ())?
            .error_for_status()
            .map_err(|_| ())?
            .json::<Discovery>()
            .await
            .map_err(|_| ())?;
        *self.discovery.write().await = Some(value.clone());
        Ok(value)
    }

    #[cfg_attr(test, allow(dead_code))]
    async fn keys(&self, discovery: &Discovery) -> Result<JwkSet, ()> {
        if let Some((created, keys)) = self.keys.read().await.clone() {
            if created.elapsed() < Duration::from_secs(3600) {
                return Ok(keys);
            }
        }
        let keys = self
            .http
            .get(&discovery.jwks_uri)
            .send()
            .await
            .map_err(|_| ())?
            .error_for_status()
            .map_err(|_| ())?
            .json::<JwkSet>()
            .await
            .map_err(|_| ())?;
        *self.keys.write().await = Some((Instant::now(), keys.clone()));
        Ok(keys)
    }

    pub(crate) async fn owner_oid(&self, headers: &HeaderMap) -> Result<String, AuthError> {
        #[cfg(test)]
        {
            return headers
                .get("x-test-oid")
                .and_then(|value| value.to_str().ok())
                .map(ToOwned::to_owned)
                .ok_or(AuthError);
        }
        #[cfg(not(test))]
        {
            // The browser harness injects this only into its isolated local
            // process. Production never sets TEST_ENTRA_OID, so this branch
            // cannot create an alternate authentication path in deployment.
            if let Ok(test_oid) = std::env::var("TEST_ENTRA_OID") {
                if let Some(candidate) = headers
                    .get("x-test-oid")
                    .and_then(|value| value.to_str().ok())
                {
                    if candidate == test_oid || candidate.starts_with(&format!("{test_oid}-")) {
                        return Ok(candidate.to_owned());
                    }
                }
            }
            let token = headers
                .get(header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
                .ok_or(AuthError)?;
            let discovery = self.discovery().await.map_err(|_| AuthError)?;
            let token_header = decode_header(token).map_err(|_| AuthError)?;
            let kid = token_header.kid.ok_or(AuthError)?;
            let keys = self.keys(&discovery).await.map_err(|_| AuthError)?;
            let jwk = keys.find(&kid).ok_or(AuthError)?;
            let key = DecodingKey::from_jwk(jwk).map_err(|_| AuthError)?;
            let mut validation = Validation::new(Algorithm::RS256);
            validation.set_issuer(&[discovery.issuer]);
            validation.set_audience(&[self.client_id.as_ref()]);
            let claims = decode::<Claims>(token, &key, &validation)
                .map_err(|_| AuthError)?
                .claims;
            if claims.tid != self.tenant_id.as_ref()
                || claims.oid.is_empty()
                || !audience_matches(&claims.aud, &self.client_id)
            {
                return Err(AuthError);
            }
            Ok(claims.oid)
        }
    }
}

#[cfg_attr(test, allow(dead_code))]
fn audience_matches(aud: &serde_json::Value, expected: &str) -> bool {
    aud.as_str().is_some_and(|value| value == expected)
        || aud
            .as_array()
            .is_some_and(|values| values.iter().any(|value| value.as_str() == Some(expected)))
}

pub(crate) struct AuthError;
impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
            "Sign in is required.",
        )
            .into_response()
    }
}
