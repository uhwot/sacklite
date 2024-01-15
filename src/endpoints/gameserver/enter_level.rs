use axum::{Router, routing::post, extract::Path, http::StatusCode};

use crate::AppState;
use crate::endpoints::gameserver::SlotType;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/enterLevel/:type/:id", post(enter_level))
}

async fn enter_level(Path((_, _)): Path<(SlotType, u64)>) -> StatusCode {
    StatusCode::OK
}
