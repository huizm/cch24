use std::{net::{Ipv4Addr, Ipv6Addr}, str::FromStr};

use axum::{extract::{rejection::JsonRejection, Json, Query}, http::{header, HeaderMap, Response, StatusCode}, response::IntoResponse, routing::{get, post}, Router};

async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

async fn seek() -> impl IntoResponse {
    (
        StatusCode::FOUND,
        [(header::LOCATION, "https://www.youtube.com/watch?v=9Gc4QTqslN4")],
    )
}

#[derive(serde::Deserialize)]
struct DestReq {
    from: Ipv4Addr,
    key: Ipv4Addr,
}

async fn dest(req: Query<DestReq>) -> impl IntoResponse {
    let source_addr = req.0;
    let from = source_addr.from.octets();
    let key = source_addr.key.octets();
    
    let to = Ipv4Addr::new(
        from[0].overflowing_add(key[0]).0,
        from[1].overflowing_add(key[1]).0,
        from[2].overflowing_add(key[2]).0,
        from[3].overflowing_add(key[3]).0,
    );
    to.to_string()
}

#[derive(serde::Deserialize)]
struct KeyReq {
    from: Ipv4Addr,
    to: Ipv4Addr,
}

async fn key(req: Query<KeyReq>) -> impl IntoResponse {
    let source_addr = req.0;
    let from = source_addr.from.octets();
    let to = source_addr.to.octets();

    let key = Ipv4Addr::new(
        to[0].overflowing_sub(from[0]).0,
        to[1].overflowing_sub(from[1]).0,
        to[2].overflowing_sub(from[2]).0,
        to[3].overflowing_sub(from[3]).0,
    );
    key.to_string()
}

#[derive(serde::Deserialize)]
struct DestV6Req {
    from: Ipv6Addr,
    key: Ipv6Addr,
}

async fn dest_v6(req: Query<DestV6Req>) -> impl IntoResponse {
    let source_addr = req.0;
    let from = source_addr.from.to_bits();
    let key = source_addr.key.to_bits();

    let to = Ipv6Addr::from_bits(from ^ key);
    to.to_string()
}

#[derive(serde::Deserialize)]
struct KeyV6Req {
    from: Ipv6Addr,
    to: Ipv6Addr,
}

async fn key_v6(req: Query<KeyV6Req>) -> impl IntoResponse {
    let source_addr = req.0;
    let from = source_addr.from.to_bits();
    let to = source_addr.to.to_bits();

    let key = Ipv6Addr::from_bits(to ^ from);
    key.to_string()
}

async fn manifest(body: String) -> Result<(StatusCode, String), (StatusCode, &'static str)> {
    // let mut orders: Vec<order::Order> = Vec::new();

    // parse cargo manifest, return error if failed
    let manifest = cargo_manifest::Manifest::from_str(body.as_str()).map_err(|_| (
            StatusCode::BAD_REQUEST,
            "Invalid manifest",
        ))?;
    
    let package = manifest.package.ok_or((
            StatusCode::BAD_REQUEST,
            "Invalid manifest",
        ))?;

    if !package.keywords
        .and_then(|x| x.as_local())
        .ok_or((
            StatusCode::BAD_REQUEST,
            "Magic keyword not provided",
        ))?
        .contains(&String::from("Christmas 2024")) {
            return Err((
                StatusCode::BAD_REQUEST,
                "Magic keyword not provided",
            ));
    };
    
    let orders: Vec<(&str, u32)> = package.metadata
        .as_ref()
        .and_then(|x| x.get("orders"))
        .and_then(|x| x.as_array())
        .ok_or((
            StatusCode::NO_CONTENT,
            "",
        ))?
        .iter()
        .filter_map(|x| x.as_table())
        .filter_map(|x| {
            let item = x.get("item")?.as_str()?;
            let quantity: u32 = x.get("quantity")?.as_integer()?.try_into().ok()?;
            Some((item, quantity))
        })
        .collect();

    if orders.is_empty() {
        return Err((
            StatusCode::NO_CONTENT,
            "",
        ));
    };

    let mut resp = String::from(format!("{}: {}", orders[0].0, orders[0].1));
    for (i, q) in orders.iter().skip(1) {
        resp.push_str(format!("\n{}: {}", i, q).as_str());
    };
    
    Ok((
        StatusCode::OK,
        resp,
    ))
}

#[derive(serde::Deserialize)]
struct Payload {
    liters: Option<f32>,
    gallons: Option<f32>,

    litres: Option<f32>,
    pints: Option<f32>,
}

const LITERS_PER_GALLON: f32 = 3.785411784;
const LITRES_PER_PINT: f32 = 0.56826125;

async fn milk(_headers: HeaderMap, json: Result<Json<Payload>, JsonRejection>) -> axum::response::Response {
    // Tasks 2 & 3
    if let Some(payload) = json.ok()
        .and_then(|j| Some(j.0)) {
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

            return (
                StatusCode::BAD_REQUEST,
            ).into_response();
    };

    // Task 1


    ().into_response()
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(hello_bird))
        .route("/-1/seek", get(seek))
        .route("/2/dest", get(dest))
        .route("/2/key", get(key))
        .route("/2/v6/dest", get(dest_v6))
        .route("/2/v6/key", get(key_v6))
        .route("/5/manifest", post(manifest))
        .route("/9/milk", post(milk));
    
    Ok(router.into())
}
