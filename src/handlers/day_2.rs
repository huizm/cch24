use std::net::{Ipv4Addr, Ipv6Addr};
use axum::{extract::Query, response::IntoResponse};

#[derive(serde::Deserialize)]
pub struct DestReq {
    from: Ipv4Addr,
    key: Ipv4Addr,
}

pub async fn dest(req: Query<DestReq>) -> impl IntoResponse {
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
pub struct KeyReq {
    from: Ipv4Addr,
    to: Ipv4Addr,
}

pub async fn key(req: Query<KeyReq>) -> impl IntoResponse {
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
pub struct DestV6Req {
    from: Ipv6Addr,
    key: Ipv6Addr,
}

pub async fn dest_v6(req: Query<DestV6Req>) -> impl IntoResponse {
    let source_addr = req.0;
    let from = source_addr.from.to_bits();
    let key = source_addr.key.to_bits();

    let to = Ipv6Addr::from_bits(from ^ key);
    to.to_string()
}

#[derive(serde::Deserialize)]
pub struct KeyV6Req {
    from: Ipv6Addr,
    to: Ipv6Addr,
}

pub async fn key_v6(req: Query<KeyV6Req>) -> impl IntoResponse {
    let source_addr = req.0;
    let from = source_addr.from.to_bits();
    let to = source_addr.to.to_bits();

    let key = Ipv6Addr::from_bits(to ^ from);
    key.to_string()
}
