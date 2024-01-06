use axum::{Router, extract::State, routing::get};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/eula", get(eula))
        .route("/announce", get(announce))
}

async fn eula(State(state): State<AppState>) -> String {
    state.config.eula.clone()
}

async fn announce(State(state): State<AppState>) -> String {
    state.config.announcement.clone()
}
