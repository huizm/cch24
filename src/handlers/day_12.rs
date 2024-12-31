use lazy_static::lazy_static;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::{Arc, Mutex};
use axum::{extract::{State, Path}, response::{Response, IntoResponse}, http::StatusCode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tile {
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

#[derive(Debug)]
pub struct Board {
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

    fn new_random(rng: &mut StdRng) -> Self {
        let mut b = Self::new();
        
        // generate tiles
        let mut generated_tiles: Vec<Tile> = Vec::new();
        for _ in 0..16 {
            generated_tiles.push(if rng.gen::<bool>() { Tile::Cookie } else { Tile::Milk });
        };

        // insert tiles reversely
        for i in (0..16).rev() {
            b.b[i % 4].push(generated_tiles[i]);
        };

        // check for winner lazily
        b.check_winner();

        b
    }

    /// Check for winner lazily
    fn check_winner(&mut self) {
        // diagonal
        if (0..=3).map(|i| self[(i, i)]).filter(|t| *t != Tile::Empty).all(|t| t == self[(0, 0)]) {
            self.winner = Some(self[(0, 0)]);
            return;
        } else if (0..=3).map(|i| self[(i, 3 - i)]).filter(|t| *t != Tile::Empty).all(|t| t == self[(0, 3)]) {
            self.winner = Some(self[(0, 3)]);
            return;
        };

        // row and column
        for i in 0..=3 {
            if (0..=3).map(|j| self[(i, j)]).filter(|t| *t != Tile::Empty).all(|t| t == self[(i, 0)]) {
                self.winner = Some(self[(i, 0)]);
                return;
            } else if (0..=3).map(|j| self[(i, j)]).filter(|t| *t != Tile::Empty).all(|t| t == self[(0, i)]) {
                self.winner = Some(self[(0, i)]);
                return;
            }
        }

        // game finished
        if self.b.iter().all(|v| v.len() == 4) {
            self.winner = Some(Tile::Empty);
        } else {
            self.winner = None;
        };
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
    pub static ref singleton_board: Arc<Mutex<Board>> = Arc::new(Mutex::new(Board::new()));
}

pub async fn board(State(b): State<Arc<Mutex<Board>>>) -> impl IntoResponse {
    let b = b.lock().unwrap();
    b.to_string()
}

pub async fn reset(State((b, rng)): State<(Arc<Mutex<Board>>, Arc<Mutex<StdRng>>)>) -> impl IntoResponse {
    let mut b = b.lock().unwrap();
    b.reset();

    let mut rng = rng.lock().unwrap();
    *rng = rand::rngs::StdRng::seed_from_u64(2024);

    b.to_string()
}

pub async fn place(
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

pub async fn random_board(State(rng): State<Arc<Mutex<StdRng>>>) -> String {
    let mut rng = rng.lock().unwrap();
    Board::new_random(&mut *rng).to_string()
}
