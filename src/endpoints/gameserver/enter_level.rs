use axum::{Router, routing::post, extract::Path, http::StatusCode};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/enterLevel/:type/:id", post(enter_level))
}

async fn enter_level(Path((slot_type, _)): Path<(String, u64)>) -> StatusCode {
    if !["developer", "user"].contains(&slot_type.as_str()) {
        return StatusCode::NOT_FOUND;
    }
    StatusCode::OK
}
