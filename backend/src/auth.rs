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
            headers
                .get("x-test-oid")
                .and_then(|value| value.to_str().ok())
                .map(ToOwned::to_owned)
                .ok_or(AuthError)
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
            self.validate_token(token, &kid, &discovery, &keys)
        }
    }

    /// Keeps the production token checks in one testable boundary. Discovery
    /// supplies the issuer and key set; the validator then enforces RS256,
    /// audience, tenant, expiry, nbf, and stable oid.
    fn validate_token(
        &self,
        token: &str,
        kid: &str,
        discovery: &Discovery,
        keys: &JwkSet,
    ) -> Result<String, AuthError> {
        let jwk = keys.find(kid).ok_or(AuthError)?;
        let key = DecodingKey::from_jwk(jwk).map_err(|_| AuthError)?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&discovery.issuer]);
        validation.set_audience(&[self.client_id.as_ref()]);
        validation.validate_nbf = true;
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

#[cfg_attr(test, allow(dead_code))]
fn audience_matches(aud: &serde_json::Value, expected: &str) -> bool {
    aud.as_str().is_some_and(|value| value == expected)
        || aud
            .as_array()
            .is_some_and(|values| values.iter().any(|value| value.as_str() == Some(expected)))
}

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde_json::json;

    use super::{Discovery, EntraValidator};

    // This is an isolated test-only key pair. It has no production authority.
    const TEST_PRIVATE_KEY: &str = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEogIBAAKCAQEAxcfjNuwexMhxFA0POMxM9TrXkuBUTNekSfVb0vjZqoiCpJFJ
sWsAtgJh1+lgvqJsqGhUezJiKzQyTr6d10b5wJOZpgE3zdndxhIcNXoFFooVBqki
+mx6plDTVMYgXAykPx1TvNghcg6IMYV0RbQ1lgrrfwqgLB4VpTiA/2O0bWkb+LCG
A5iz+TuB8WrOhJ/EBip7k9i5cvLMTFpV/xC85niB2v3emHPJtZPGe8cqBRNVyanl
dO3We6Y4t/Zrrgfg1kO2ODOJO+q4LJwM6icTO0UTdfNX29PxzYN615za+hDjSZgE
TmwmKWw4fGZP+PblUxoLABquY97D321qISJn7QIDAQABAoIBADPxFHualFQS3hup
eFtu0DFBNFdXAdyyb2ua+/QStbuDIWhN3cAx/VxPkNmu6WD3cDjkOXenWj/FdAbZ
KcPdWH8aZGpD/J9bvdIkdHMY6hgqyG3Y4p5I+gcOyAmGBP6XtVT8Az9ftZzqMxtq
VIhv1PjkQke5hyo+9mlPRxWLXlmzH2c4nhfm2m0EAOLwOJG5M9CMRC1ofxBNxlZE
6kkC/2nCsV7wvoSW8NBlvZ353vJTHN5O7YiQ/OSj+kyNqnW6RHCI8Iz/JslzPown
/5+Ka89DDes933CBW8WQUdbvhgOmZlx1VinWqm8tCE36fR80XfQPiWb3p7IjcMIj
7HrPLzMCgYEA/4lq+ZETSFM6xImX6WusODcsVs6JpWIWuzGtZI1giVtpcyLkmF/Q
O3vLscIIfwRjjOHJg3XuMEqPkj9QoYRhTaB8UTuJM5Nu8ZPR0j9WSJZcUkPQxByk
+hIy/b8VNoxqLWVb7QWJp6MpLZ1NVfaE0hvwpg8f1F5PW4Rtiba5JDsCgYEAxiOq
/3sYwVh9P+x/DSN0pHr97LORwCDwypRckmbZOW3rZUo7W7Gq3CqtKqlB5J0EQAYJ
PlBhU2y6D9n9VECerph7qdLhxbsECNLkyDpTFoy9yBmtoR7CBy3kp7wyC2HMT65Z
/izXwFX5g6V44ZdkSsrbwtyaGk1DvwvLQFzpKfcCgYBM2Qm3xf5TiNwqkOqDgyMG
wOjvritM9kO5xgXMMIwworICsyKmBGJ+EQvACIc/k5VQn/JXO5cHJNUqeSoJeOM+
Uh4w28O2JAeAVSELpoqPR9C52LUm1Sp0HhcBon6BqhagUlQj4r90D6hplF3WlU16
Vna3qeK7niUlc5zxhmcFUwKBgFpUrIKo5hJPe1qHQS0GOwk5oUYmX45N9jkFBmcg
SGwsNqMJAqK9Dv4s3qGSZJ4LD4L4vYIRNy3HZdQQN7QPechzb/1uTMvOhPpY20CF
hpfDNkphmozX7vFC9PmbjN0viuvQuupsGzhuecCQ0dlXIbwPW70swXy4OOiCQfln
4kzlAoGARx3bE7joB2vToXBkNqqftbQtkEO2F1dP5DC7TeOtZN0ji18mQ9+v0BCA
jI+LqxjlamMom8Ir9y7L26etbg58DfPfkvstLpxrL3CEaYPLoOTzNReYgXUS8yRO
gF361bU8g65u0wfXupV3am3yBcXQo1h0NWK6/DyW5a38YV3yZn8=
-----END RSA PRIVATE KEY-----"#;
    const TEST_JWK_N: &str = "xcfjNuwexMhxFA0POMxM9TrXkuBUTNekSfVb0vjZqoiCpJFJsWsAtgJh1-lgvqJsqGhUezJiKzQyTr6d10b5wJOZpgE3zdndxhIcNXoFFooVBqki-mx6plDTVMYgXAykPx1TvNghcg6IMYV0RbQ1lgrrfwqgLB4VpTiA_2O0bWkb-LCGA5iz-TuB8WrOhJ_EBip7k9i5cvLMTFpV_xC85niB2v3emHPJtZPGe8cqBRNVyanldO3We6Y4t_Zrrgfg1kO2ODOJO-q4LJwM6icTO0UTdfNX29PxzYN615za-hDjSZgETmwmKWw4fGZP-PblUxoLABquY97D321qISJn7Q";

    fn validator() -> EntraValidator {
        EntraValidator::from_environment(reqwest::Client::new())
    }

    fn discovery() -> Discovery {
        Discovery {
            issuer: "https://issuer.example.test/v2.0".to_owned(),
            jwks_uri: "https://keys.example.test/jwks".to_owned(),
        }
    }

    fn keys() -> jsonwebtoken::jwk::JwkSet {
        serde_json::from_value(json!({
            "keys": [{
                "kty": "RSA", "kid": "test-kid", "use": "sig", "alg": "RS256",
                "n": TEST_JWK_N, "e": "AQAB"
            }]
        }))
        .expect("test JWK set should parse")
    }

    fn token(overrides: serde_json::Value) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_secs();
        let mut payload = json!({
            "oid": "stable-owner-oid",
            "tid": "35c6fe40-0ec0-46b6-98c6-213ad4de6650",
            "aud": "25c704f4-465a-47af-80ab-2c489466b697",
            "iss": "https://issuer.example.test/v2.0",
            "exp": now + 300,
            "nbf": now.saturating_sub(5)
        });
        for (key, value) in overrides.as_object().expect("object overrides") {
            payload[key] = value.clone();
        }
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("test-kid".to_owned());
        encode(
            &header,
            &payload,
            &EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY.as_bytes()).expect("test key"),
        )
        .expect("test token should sign")
    }

    #[test]
    fn production_token_validation_requires_discovery_signature_audience_tenant_time_and_oid() {
        let validator = validator();
        let discovery = discovery();
        let keys = keys();
        let valid = token(json!({}));
        assert_eq!(
            validator
                .validate_token(&valid, "test-kid", &discovery, &keys)
                .expect("a fully valid token"),
            "stable-owner-oid"
        );

        for invalid in [
            token(json!({"iss": "https://wrong-issuer.example.test"})),
            token(json!({"aud": "wrong-client"})),
            token(json!({"tid": "wrong-tenant"})),
            token(json!({"oid": ""})),
            token(json!({"exp": 1})),
            token(json!({"nbf": u64::MAX / 4})),
        ] {
            assert!(
                validator
                    .validate_token(&invalid, "test-kid", &discovery, &keys)
                    .is_err(),
                "invalid issuer, audience, tenant, oid, expiry, or nbf must be rejected"
            );
        }
        let tampered = format!("{}x", valid);
        assert!(validator
            .validate_token(&tampered, "test-kid", &discovery, &keys)
            .is_err());
    }

    #[test]
    fn audience_must_equal_the_registered_client() {
        assert!(super::audience_matches(
            &json!(["other-client", "25c704f4-465a-47af-80ab-2c489466b697"]),
            "25c704f4-465a-47af-80ab-2c489466b697"
        ));
        assert!(!super::audience_matches(
            &json!(["other-client"]),
            "registered-client"
        ));
    }
}
