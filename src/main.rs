use axum::{http::{header, StatusCode}, response::IntoResponse, routing::{get, post}, Router};

mod handlers;

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
async fn main() -> shuttle_axum::ShuttleAxum {
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
        .route("/16/unwrap", get(handlers::unwrap));
    
    Ok(router.into())
}
