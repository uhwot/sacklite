use uuid::Uuid;

use super::{GameVersion, Platform};

#[derive(Clone)]
pub struct SessionData {
    pub user_id: Uuid,
    pub online_id: String,
    pub platform: Platform,
    pub game_version: GameVersion,
}
