use std::{collections::HashSet, fs};

use axum::{http::{header, HeaderMap, StatusCode}, response::IntoResponse, extract::Json};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde_json::Value;

static SECRET: &str = "lingang guli guli guli grata lingangu lingangu";

pub async fn wrap(Json(payload): Json<Value>) -> impl IntoResponse {
    let mut gift = String::from("gift=");
    let token = jsonwebtoken::encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret(SECRET.as_ref())
    ).unwrap();
    gift.push_str(token.as_str());

    (
        StatusCode::OK,
        [(header::SET_COOKIE, gift)],
    )
}

pub async fn unwrap(headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    let validation = &mut Validation::default();
    validation.required_spec_claims = HashSet::new();
    
    if let Some(gift) = headers.get("Cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("gift="))
        .and_then(|t| jsonwebtoken::decode::<Value>(t,
            &DecodingKey::from_secret(SECRET.as_ref()),
            validation).ok())
        .and_then(|d| Some(d.claims))
    {
        Ok(Json(gift))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn decode(token: String) -> Result<Json<Value>, StatusCode> {
    let key = DecodingKey::from_rsa_pem(
        fs::read_to_string("keys/day16_santa_public_key.pem")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .as_bytes()
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let validation = &mut Validation::default();
    validation.required_spec_claims = HashSet::new();
    validation.algorithms = vec![jsonwebtoken::Algorithm::RS256, jsonwebtoken::Algorithm::RS512];

    jsonwebtoken::decode::<Value>(
        &token,
        &key,
        validation,
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::InvalidSignature => StatusCode::UNAUTHORIZED,
        jsonwebtoken::errors::ErrorKind::InvalidEcdsaKey => StatusCode::UNAUTHORIZED,
        jsonwebtoken::errors::ErrorKind::InvalidRsaKey(_) => StatusCode::UNAUTHORIZED,
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => StatusCode::UNAUTHORIZED,
        _ => StatusCode::BAD_REQUEST,
    })
    .map(|d| Json(d.claims))
}
