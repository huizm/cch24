use std::sync::Arc;

use axum::{extract::{State, Path, Json}, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::{types::{chrono::Local,  Uuid}, PgPool};

use crate::models::Quote;

/// Clear the `quotes` table
pub async fn clear_quotes(
    State(pool): State<Arc<PgPool>>,
) -> Result<StatusCode, StatusCode>
{   
    match sqlx::query("DELETE FROM quotes")
        .execute(&*pool)
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Get quote by ID
pub async fn cite(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Quote>, StatusCode>
{
    match sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = ?")
        .bind(id)
        .fetch_one(&*pool)
        .await
    {
        Ok(q) => Ok(Json(q)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

// Delete quote with givin ID, respond with content
pub async fn remove(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Quote>, StatusCode>
{
    // use transaction for atomicity
    let mut tx = pool.begin().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let q = sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = ?")
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    match sqlx::query("DELETE FROM quotes WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
    {
        Ok(_) => {
            tx.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(q))
        },
        Err(_) => {
            // not commit transaction if errors are met
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update a record with givin ID
pub async fn undo(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<Uuid>,
    Json(req): Json<QuoteReq>,
) -> Result<Json<Quote>, StatusCode>
{
    match sqlx::query_as::<_, Quote>("UPDATE quotes SET author = ?, quote = ?, version = version + 1 WHERE id = ? RETURNING *")
        .bind(req.author)
        .bind(req.quote)
        .bind(id)
        .fetch_one(&*pool)
        .await
    {
        Ok(q) => Ok(Json(q)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteReq {
    pub author: String,
    pub quote: String,
}

/// Create new record
pub async fn draft(
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<QuoteReq>,
) -> Result<Json<Quote>, StatusCode>
{
    match sqlx::query_as::<_, Quote>("INSERT TO quotes (id, author, quote, created_at, version) VALUES (?, ?, ?) RETURNING *")
        .bind(Uuid::new_v4())
        .bind(req.author)
        .bind(req.quote)
        // .bind(Local::now())
        // .bind(1)
        .fetch_one(&*pool)
        .await
    {
        Ok(q) => Ok(Json(q)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
