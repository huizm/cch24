use axum::{extract::Path, http::StatusCode, response::Html};

pub async fn star() -> Html<&'static str> {
    Html(r#"<html><div id="star" class="lit"></div></html>"#)
}

pub async fn color(Path(color): Path<String>) -> Result<Html<String>, StatusCode> {
    Ok(Html(format!(r#"
        <html>
            <div class="present {}" hx-get="/23/present/{}" hx-swap="outerHTML">
                <div class="ribbon"></div>
                <div class="ribbon"></div>
                <div class="ribbon"></div>
                <div class="ribbon"></div>
            </div>
        </html>
    "#, color, match color.as_str() {
        "red" => "blue",
        "blue" => "purple",
        "purple" => "red",
        _ => { return Err(StatusCode::IM_A_TEAPOT); },
    })))
}
