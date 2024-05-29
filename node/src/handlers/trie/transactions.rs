use axum::{
    extract::{Json, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::models::transactions;
use crate::AppState;

pub async fn transaction(
    State(AppState): State<AppState>,
    Json(transaction): Json<transactions::Transaction>,
) -> impl IntoResponse {
    let state = State(AppState);
    transactions::insert(state.pool.clone(), transaction);
    (StatusCode::OK, "Transaction submitted successfully!")
}
