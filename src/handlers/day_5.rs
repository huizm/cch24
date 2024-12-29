use std::str::FromStr;
use axum::http::StatusCode;

pub async fn manifest(body: String) -> Result<(StatusCode, String), (StatusCode, &'static str)> {
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
