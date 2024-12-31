use axum::{http::header, response::IntoResponse};

pub async fn star() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html")],
        r#"<html><div id="star" class="lit"></div></html>"#,
    )
}
