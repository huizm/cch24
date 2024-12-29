use std::{net::{Ipv4Addr, Ipv6Addr}, str::FromStr, sync::{Arc, Mutex}, time::Duration};

use axum::{extract::{rejection::JsonRejection, Json, Path, Query, State}, http::{header, StatusCode}, response::{IntoResponse, Response}, routing::{get, post}, Router};
use lazy_static::lazy_static;

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

lazy_static! {
    static ref cow: Arc<leaky_bucket::RateLimiter> = Arc::new(
        leaky_bucket::RateLimiter::builder()
            .initial(5)
            .refill(1)
            .max(5)
            .interval(Duration::from_secs(1))
            .build()
    );
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

async fn milk(
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

#[derive(Clone, Copy, PartialEq)]
enum Tile {
    Empty,
    Cookie,
    Milk,
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
                Tile::Empty => '⬛',
                Tile::Cookie => '🍪',
                Tile::Milk => '🥛',
        })
    }
}

struct Board {
    b: [Vec<Tile>; 4], // each vec a *column* not *row*
    winner: Option<Tile>,
}

impl std::ops::Index<(usize, usize)> for Board {
    type Output = Tile;

    /// # Contract
    /// 
    /// - `index.0`, `index.1` within `0..=3`
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let col = &self.b[index.1];
        if col.len() <= 3 - index.0 {
            &Tile::Empty
        } else {
            &col[3 - index.0]
        }
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "⬜{}{}{}{}⬜\n⬜{}{}{}{}⬜\n⬜{}{}{}{}⬜\n⬜{}{}{}{}⬜\n⬜⬜⬜⬜⬜⬜\n{}",
            self[(0, 0)], self[(0, 1)], self[(0, 2)], self[(0, 3)],
            self[(1, 0)], self[(1, 1)], self[(1, 2)], self[(1, 3)],
            self[(2, 0)], self[(2, 1)], self[(2, 2)], self[(2, 3)],
            self[(3, 0)], self[(3, 1)], self[(3, 2)], self[(3, 3)],
            match self.winning_message() {
                Some(s) => s,
                None => String::new(),
            },
        )
    }
}

impl Board {
    fn new() -> Self {
        Self {
            b: Default::default(),
            winner: None,
        }
    }

    fn reset(&mut self) {
        self.b = Default::default();
        self.winner = None;
    }

    fn winning_message(&self) -> Option<String> {
        self.winner
            .and_then(|w| match w {
                    Tile::Empty => Some("No winner.\n".to_owned()),
                    other => Some(format!("{} wins!\n", other)),
        })
    }

    fn insert(&mut self, column: usize, team: Tile) -> bool {
        if self.winner.is_some() {
            // game already finished
            return false;
        };

        // insert tile
        let col = &mut self.b[column];
        let row = col.len();
        if row == 4 {
            return false;
        };
        col.push(team);
        
        // check winner eagerly
        if self.b.iter().all(|v| v.len() == 4) {
            // game finishes, no winner
            self.winner = Some(Tile::Empty);
        };

        if (0..=3).map(|i| self[(row, i)]).all(|t| t == team) || // column
            (0..=3).map(|i| self[(i, column)]).all(|t| t == team) || // row
            (0..=3).map(|i| self[(i, i)]).all(|t| t == team) ||
            (0..=3).map(|i| self[(i, 3 - i)]).all(|t| t == team) // diagonal
        {
            // game finishes, team wins
            self.winner = Some(team);
        };

        true
    }
}

lazy_static! {
    static ref singleton_board: Arc<Mutex<Board>> = Arc::new(Mutex::new(Board::new()));
}

async fn board(State(b): State<Arc<Mutex<Board>>>) -> impl IntoResponse {
    let b = b.lock().unwrap();
    b.to_string()
}

async fn reset(State(b): State<Arc<Mutex<Board>>>) -> impl IntoResponse {
    let mut b = b.lock().unwrap();
    b.reset();
    b.to_string()
}

async fn place(
    Path((team, col)): Path<(String, usize)>,
    State(b): State<Arc<Mutex<Board>>>,
) -> Response
{
    // validate data
    if !(1..=4).contains(&col) {
        return (
            StatusCode::BAD_REQUEST,
        ).into_response();
    };
    
    let team = if team == "cookie"
    {
        Tile::Cookie
    } else if team == "milk" {
        Tile::Milk
    } else {
        return (
            StatusCode::BAD_REQUEST,
        ).into_response();
    };

    // insert tile
    let mut b = b.lock().unwrap();
    if !b.insert(col - 1, team) {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            b.to_string(),
        ).into_response()
    } else {
        (
            StatusCode::OK,
            b.to_string(),
        ).into_response()
    }
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
        .route("/9/milk", post(milk)).with_state(cow.clone())
        .route("/12/board", get(board).with_state(singleton_board.clone()))
        .route("/12/reset", post(reset).with_state(singleton_board.clone()))
        .route("/12/place/:team/:column", post(place).with_state(singleton_board.clone()));
    
    Ok(router.into())
}
