use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use jsonwebtoken as jwt;
use serde::{Deserialize, Serialize};

use crate::model::UserId;
use crate::Elimination;

#[must_use]
#[derive(Clone)]
pub struct Manager {
    inner: Arc<Inner>,
}

#[must_use]
#[derive(Clone)]
struct Inner {
    algorithm: jwt::Algorithm,
    issuer: String,
    lifetime: Duration,
    #[allow(unused)]
    raw_key: String,
    enc_key: jwt::EncodingKey,
    dec_key: jwt::DecodingKey,
    validation: jwt::Validation,
}

#[must_use]
#[derive(Debug, Clone)]
pub struct Builder<Key = (), Issuer = (), Lifetime = ()> {
    key: Key,
    issuer: Issuer,
    lifetime: Lifetime,
}

impl Builder {
    fn new() -> Self {
        Self {
            key: (),
            issuer: (),
            lifetime: (),
        }
    }
}

impl<Key, Issuer, Lifetime> Builder<Key, Issuer, Lifetime> {
    pub fn key(self, value: &str) -> Builder<String, Issuer, Lifetime> {
        let Self {
            key: _,
            issuer,
            lifetime,
        } = self;
        Builder {
            key: value.to_string(),
            issuer,
            lifetime,
        }
    }

    pub fn issuer(self, value: &str) -> Builder<Key, String, Lifetime> {
        let Self {
            key,
            issuer: _,
            lifetime,
        } = self;
        Builder {
            key,
            issuer: value.to_string(),
            lifetime,
        }
    }

    pub fn lifetime(self, value: Duration) -> Builder<Key, Issuer, Duration> {
        let Self {
            key,
            issuer,
            lifetime: _,
        } = self;
        Builder {
            key,
            issuer,
            lifetime: value,
        }
    }
}

impl Builder<String, String, Duration> {
    pub fn build(self) -> Manager {
        let Self {
            key: raw_key,
            issuer,
            lifetime,
        } = self;
        let algorithm = jwt::Algorithm::HS256;
        let enc_key = jwt::EncodingKey::from_secret(raw_key.as_bytes());
        let dec_key = jwt::DecodingKey::from_secret(raw_key.as_bytes());
        let validation = jwt::Validation::new(algorithm);
        let inner = Inner {
            algorithm,
            issuer,
            lifetime,
            raw_key,
            enc_key,
            dec_key,
            validation,
        };
        Manager {
            inner: Arc::new(inner),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct EncodeClaims<'a> {
    iat: u64,
    exp: u64,
    #[serde(borrow = "'a")]
    iss: &'a str,
    sub: UserId,
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
struct DecodeClaims {
    iat: u64,
    exp: u64,
    iss: String,
    sub: UserId,
}

impl Manager {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn encode(&self, id: UserId) -> Result<String, Elimination> {
        let iat = jwt::get_current_timestamp();
        let exp = iat + self.inner.lifetime.as_secs();
        let iss = self.inner.issuer.as_str();
        let sub = id;
        let claims = EncodeClaims { iat, exp, iss, sub };
        let header = jwt::Header::new(self.inner.algorithm);
        let key = &self.inner.enc_key;
        let encoded = jwt::encode(&header, &claims, key).context("Failed to encode JWT")?;
        Ok(encoded)
    }

    pub fn decode(&self, token: &str) -> Result<UserId, Elimination> {
        let key = &self.inner.dec_key;
        let validation = &self.inner.validation;
        let token = jwt::decode(token, key, validation).context("Failed to decode JWT")?;
        let DecodeClaims { sub, .. } = token.claims;
        Ok(sub)
    }
}
