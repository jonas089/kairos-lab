use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};

use crate::AppState;
//use crate::handlers::delta_tree;

pub fn trie_routes() -> Router<AppState> {
    Router::new()
    //.route("/transfer", post(delta_tree::transfer::transfer))
    //.route("/submit_batch", get(delta_tree::submit_batch::submit_batch))
}
