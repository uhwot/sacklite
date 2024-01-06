use axum::{Router, routing::get};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, IntoStaticStr};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/tags", get(tags))
}

// tags for levels
// LBP1 only
#[allow(non_camel_case_types)]
#[derive(EnumIter, IntoStaticStr)]
enum Tags {
    Brilliant,
    Beautiful,
    Funky,
    Points_Fest,
    Weird,
    Tricky,
    Short,
    Vehicles,
    Easy,
    Cute,
    Quick,
    Fun,
    Relaxing,
    Great,
    Speedy,
    Race,
    Multi_Path,
    Machines,
    Complex,
    Pretty,
    Rubbish,
    Toys,
    Repetitive,
    Machinery,
    Satisfying,
    Braaains,
    Fast,
    Simple,
    Long,
    Slow,
    Mad,
    Hectic,
    Creepy,
    Perilous,
    Empty,
    Ingenious,
    Lousy,
    Frustrating,
    Timing,
    Boss,
    Springy,
    Funny,
    Musical,
    Good,
    Hilarious,
    Electric,
    Puzzler,
    Platformer,
    Difficult,
    Mechanical,
    Horizontal,
    Splendid,
    Fiery,
    Swingy,
    Single_Path,
    Annoying,
    Co_op,
    Boring,
    Moody,
    Bubbly,
    Nerve_wracking,
    Hoists,
    Ugly,
    Daft,
    Ramps,
    Secrets,
    Floaty,
    Artistic,
    Competitive,
    Gas,
    Varied,
    Stickers,
    Spikes,
    Collectables,
    Vertical,
    Balancing,
}

async fn tags() -> String {
    let mut tags = vec![];
    for tag in Tags::iter() {
        tags.push("TAG_".to_string() + tag.into());
    }
    tags.join(",")
}
