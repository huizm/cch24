use std::net::{Ipv4Addr, Ipv6Addr};

use axum::{extract::Query, http::{header, StatusCode}, response::IntoResponse, routing::get, Router};

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

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/", get(hello_bird))
                                    .route("/-1/seek", get(seek))
                                    .route("/2/dest", get(dest))
                                    .route("/2/key", get(key))
                                    .route("/2/v6/dest", get(dest_v6))
                                    .route("/2/v6/key", get(key_v6));
    
    Ok(router.into())
}
