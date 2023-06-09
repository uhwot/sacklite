pub mod user;
pub mod comment;

pub type DbError = Box<dyn std::error::Error + Send + Sync>;