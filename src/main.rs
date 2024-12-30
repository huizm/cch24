use std::sync::Arc;

use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post, delete, put},
    Router
};
use sqlx::PgPool;

mod handlers;
mod models;

async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

async fn seek() -> impl IntoResponse {
    (
        StatusCode::FOUND,
        [(header::LOCATION, "https://www.youtube.com/watch?v=9Gc4QTqslN4")],
    )
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    let pool = Arc::new(pool);
    
    let router = Router::new()
        .route("/", get(hello_bird))
        .route("/-1/seek", get(seek))
        .route("/2/dest", get(handlers::dest))
        .route("/2/key", get(handlers::key))
        .route("/2/v6/dest", get(handlers::dest_v6))
        .route("/2/v6/key", get(handlers::key_v6))
        .route("/5/manifest", post(handlers::manifest))
        .route("/9/milk", post(handlers::milk)).with_state(handlers::cow.clone())
        .route("/12/board", get(handlers::board).with_state(handlers::singleton_board.clone()))
        .route("/12/reset", post(handlers::reset).with_state(handlers::singleton_board.clone()))
        .route("/12/place/:team/:column", post(handlers::place).with_state(handlers::singleton_board.clone()))
        .route("/16/wrap", post(handlers::wrap))
        .route("/16/unwrap", get(handlers::unwrap))
        .route("/19/reset", post(handlers::clear_quotes)).with_state(pool.clone())
        .route("/19/cite/:id", get(handlers::cite)).with_state(pool.clone())
        .route("/19/remove/:id", delete(handlers::remove)).with_state(pool.clone())
        .route("/19/undo/:id", put(handlers::undo)).with_state(pool.clone())
        .route("/19/draft", post(handlers::draft)).with_state(pool.clone());

    Ok(router.into())
}
