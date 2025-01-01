use std::str::FromStr;

use axum::{extract::{Path, Multipart}, http::StatusCode, response::Html};

pub async fn star() -> Html<&'static str> {
    Html(r#"<html><div id="star" class="lit"></div></html>"#)
}

pub async fn color(Path(color): Path<String>) -> Result<Html<String>, StatusCode> {
    let color = html_escape::encode_safe(&color);

    Ok(Html(format!(r#"
        <html>
            <div class="present {}" hx-get="/23/present/{}" hx-swap="outerHTML">
                <div class="ribbon"></div>
                <div class="ribbon"></div>
                <div class="ribbon"></div>
                <div class="ribbon"></div>
            </div>
        </html>
    "#, color, match color.as_ref() {
        "red" => "blue",
        "blue" => "purple",
        "purple" => "red",
        _ => { return Err(StatusCode::IM_A_TEAPOT); },
    })))
}

pub async fn ornament(
    Path((state, n)): Path<(String, String)>,
) -> Result<Html<String>, StatusCode>
{
    let state = html_escape::encode_safe(&state);
    let n = html_escape::encode_safe(&n);

    let (curr_state, next_state) = match state.as_ref() {
        "on" => (" on", "off"),
        "off" => ("", "on"),
        _ => { return Err(StatusCode::IM_A_TEAPOT); },
    };

    Ok(Html(format!(r#"
        <html>
            <div class="ornament{}"
                id="ornament{}"
                hx-trigger="load delay:2s once"
                hx-get="/23/ornament/{}/{}"
                hx-swap="outerHTML">
            </div>
        </html>
        "#, curr_state, n, next_state, n)
    ))
}

pub async fn lockfile(mut multipart: Multipart) -> Result<Html<String>, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        if field.name() == Some("lockfile") {
            // parse lockfile
            let data = field.bytes().await.unwrap();
            let lockfile = String::from_utf8(data.to_vec()).ok()
                .and_then(|s| cargo_lock::Lockfile::from_str(s.as_str()).ok())
                .ok_or(StatusCode::BAD_REQUEST)?;
            
            // handle packages
            let mut resp = Vec::new();

            for p in lockfile.packages {
                if let Some(checksum) = p.checksum {
                    let checksum = checksum.as_sha256()
                        .ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
        
                    let color = format!("#{:x}{:x}{:x}", checksum[0], checksum[1], checksum[2]);
                    let top = checksum[3];
                    let left = checksum[4];
        
                    resp.push(format!(
                        r#"<div style="background-color:{color};top:{top}px;left:{left}px;"></div>"#,
                    ));
                };
            };

            println!("{:?}", resp);
            return Ok(Html(resp.join("\n")));
        };
    };

    Err(StatusCode::BAD_REQUEST)
}
