use std::time::Duration;

use anyhow::Context;
use jsonwebtoken as jwt;
use serde::{Deserialize, Serialize};

use crate::entity::{Credential, UserId};
use crate::Failure;

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

pub trait JwtConfig: Send + Sync {
    fn algorithm(&self) -> jwt::Algorithm;
    fn issuer(&self) -> &str;
    fn lifetime(&self) -> Duration;
    fn encodign_key(&self) -> &jwt::EncodingKey;
    fn decoding_key(&self) -> &jwt::DecodingKey;
    fn validation(&self) -> &jwt::Validation;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Jwt;

impl Jwt {
    pub fn config_builder() -> Builder {
        Builder::new()
    }
}

impl<Context> crate::entity::CredentialManager<Context> for Jwt
where
    Context: JwtConfig,
{
    async fn make_credential(
        &self,
        ctx: Context,
        params: crate::entity::MakeCredentialParams,
    ) -> Result<Credential, Failure> {
        let iat = jwt::get_current_timestamp();
        let exp = iat + ctx.lifetime().as_secs();
        let iss = ctx.issuer();
        let sub = params.user_id;
        let claims = EncodeClaims { iat, exp, iss, sub };
        let header = jwt::Header::new(ctx.algorithm());
        let key = ctx.encodign_key();
        let encoded = jwt::encode(&header, &claims, key).context("Failed to encode JWT")?;
        Ok(Credential(encoded))
    }

    async fn revoke_credential(
        &self,
        _ctx: Context,
        _credential: Credential,
    ) -> Result<(), Failure> {
        todo!()
    }

    async fn check_credential(
        &self,
        ctx: Context,
        credential: Credential,
    ) -> Result<UserId, Failure> {
        let Credential(token) = credential;
        let key = ctx.decoding_key();
        let validation = ctx.validation();
        let token = jwt::decode(&token, key, validation).context("Failed to decode JWT")?;
        let DecodeClaims { sub, .. } = token.claims;
        Ok(sub)
    }
}

#[must_use]
#[derive(Clone)]
struct JwtConfigImpl {
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
    pub fn build(self) -> JwtConfigImpl {
        let Self {
            key: raw_key,
            issuer,
            lifetime,
        } = self;
        let algorithm = jwt::Algorithm::HS256;
        let enc_key = jwt::EncodingKey::from_secret(raw_key.as_bytes());
        let dec_key = jwt::DecodingKey::from_secret(raw_key.as_bytes());
        let validation = jwt::Validation::new(algorithm);
        JwtConfigImpl {
            algorithm,
            issuer,
            lifetime,
            raw_key,
            enc_key,
            dec_key,
            validation,
        }
    }
}

impl JwtConfig for JwtConfigImpl {
    fn algorithm(&self) -> jsonwebtoken::Algorithm {
        self.algorithm
    }

    fn issuer(&self) -> &str {
        &self.issuer
    }

    fn lifetime(&self) -> Duration {
        self.lifetime
    }

    fn encodign_key(&self) -> &jsonwebtoken::EncodingKey {
        &self.enc_key
    }

    fn decoding_key(&self) -> &jsonwebtoken::DecodingKey {
        &self.dec_key
    }

    fn validation(&self) -> &jsonwebtoken::Validation {
        &self.validation
    }
}
