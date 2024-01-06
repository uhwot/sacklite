use std::{net::SocketAddr, time::Duration};

use anyhow::Context;
use axum::{Router, routing::get, middleware::from_fn};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_sessions::{SessionManagerLayer, RedisStore, fred::{clients::RedisPool, types::RedisConfig, interfaces::ClientLike}, Expiry};
use tracing::{warn, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use types::Config;

mod endpoints;
mod middleware;
mod extractors;
mod responders;
mod types;
mod utils;

#[derive(Clone)]
struct AppState {
    config: Config,
    pool: Pool<Postgres>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = types::Config::parse_from_file("config.yml");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::new(&config.log_level),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let digest_key_present = !config.digest_key.is_empty();
    if !digest_key_present {
        warn!("Server digest key is empty. Digests will not be verified and LBP games will probably not accept server responses.");
    }

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.db_conn)
        .await
        .context("can't connect to database")?;

    let state = AppState {
        config: config.clone(),
        pool,
    };

    types::pub_key_store::init_keys();

    let pool = RedisPool::new(RedisConfig::default(), None, None, None, 6).unwrap();

    #[allow(clippy::let_underscore_future)]
    let _ = pool.connect();
    pool.wait_for_connect().await.context("can't connect to Redis server")?;

    let session_store = RedisStore::new(pool);
    let session_service = ServiceBuilder::new()
        .layer(from_fn(middleware::remove_set_cookie))
        .layer(
            SessionManagerLayer::new(session_store)
                .with_name("MM_AUTH")
                .with_expiry(Expiry::OnInactivity(time::Duration::minutes(30)))
        );

    let app = Router::new()
        .nest(&config.base_path, endpoints::gameserver::routes(&config).await)
        .layer(session_service)
        .route("/autodiscover", get(endpoints::autodiscover))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let addr = config.listen_addr.parse().context("can't parse listen address")?;
    let addr = SocketAddr::new(addr, config.listen_port);
    let listener = TcpListener::bind(addr).await.context("Couldn't bind address")?;

    info!("Listening on {}:{}", config.listen_addr, config.listen_port);
    axum::serve(listener, app).await.context("can't start axum server")
}
