use axum::{Router, routing::post};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/filter", post(filter))
}

// TODO: yet another stubbaroni
async fn filter(str: String) -> String {
    str
}
