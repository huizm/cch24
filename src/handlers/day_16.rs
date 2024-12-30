use std::collections::HashSet;

use axum::{http::{header, HeaderMap, StatusCode}, response::IntoResponse, extract::Json};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde_json::Value;

static SECRET: &str = "lingang guli guli guli grata lingangu lingangu";

pub async fn wrap(Json(payload): Json<Value>) -> impl IntoResponse {
    let mut gift = String::from("gift=");
    let token = encode(
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
        .and_then(|t| decode::<Value>(t,
            &DecodingKey::from_secret(SECRET.as_ref()),
            validation).ok())
        .and_then(|d| Some(d.claims))
    {
        Ok(Json(gift))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}
