use lazy_static::lazy_static;
use std::{sync::Arc, time::Duration};
use axum::{extract::{State, Json, rejection::JsonRejection}, response::{Response, IntoResponse}, http::StatusCode};

lazy_static! {
    pub static ref cow: Arc<leaky_bucket::RateLimiter> = Arc::new(
        leaky_bucket::RateLimiter::builder()
            .initial(5)
            .refill(1)
            .max(5)
            .interval(Duration::from_secs(1))
            .build()
    );
}

#[derive(serde::Deserialize)]
pub struct Payload {
    liters: Option<f32>,
    gallons: Option<f32>,

    litres: Option<f32>,
    pints: Option<f32>,
}

const LITERS_PER_GALLON: f32 = 3.785411784;
const LITRES_PER_PINT: f32 = 0.56826125;

pub async fn milk(
    State(milkee): State<Arc<leaky_bucket::RateLimiter>>,
    json: Result<Json<Payload>, JsonRejection>,
) -> Response
{
    let milked = milkee.try_acquire(1);

    match json {
        Ok(Json(payload)) => {
            if let Some(liters) = payload.liters {
                if payload.gallons.is_none() && payload.litres.is_none() && payload.pints.is_none() {
                    return (
                        StatusCode::OK,
                        Json(serde_json::json!({"gallons": liters / LITERS_PER_GALLON})),
                    ).into_response();
                };
            };

            if let Some(gallons) = payload.gallons {
                if payload.liters.is_none() && payload.litres.is_none() && payload.pints.is_none() {
                    return (
                        StatusCode::OK,
                        Json(serde_json::json!({"liters": gallons * LITERS_PER_GALLON})),
                    ).into_response();
                };
            };

            if let Some(litres) = payload.litres {
                if payload.liters.is_none() && payload.gallons.is_none() && payload.pints.is_none() {
                    return (
                        StatusCode::OK,
                        Json(serde_json::json!({"pints": litres / LITRES_PER_PINT})),
                    ).into_response();
                };
            };

            if let Some(pints) = payload.pints {
                if payload.liters.is_none() && payload.gallons.is_none() && payload.litres.is_none() {
                    return (
                        StatusCode::OK,
                        Json(serde_json::json!({"litres": pints * LITRES_PER_PINT})),
                    ).into_response();
                };
            };

            (
                StatusCode::BAD_REQUEST,
            ).into_response()
        },
        Err(e) => {
            match e {
                JsonRejection::MissingJsonContentType(_) => {
                    if milked {
                        (
                            StatusCode::OK,
                            "Milk withdrawn\n",
                        ).into_response()
                    } else {
                        (
                            StatusCode::TOO_MANY_REQUESTS,
                            "No milk available\n",
                        ).into_response()
                    }
                },
                _ => {
                    (
                        StatusCode::BAD_REQUEST
                    ).into_response()
                },
            }
        },
    }
}
