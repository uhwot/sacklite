use std::sync::Arc;

use actix_session::{
    config::{CookieContentSecurity, PersistentSession},
    storage::RedisActorSessionStore,
    SessionMiddleware,
};
use actix_web::{
    cookie::{time::Duration, Key},
    middleware::{Compress, Condition, Logger},
    web::{self, Data, PayloadConfig, JsonConfig}, App, HttpServer,
};
use actix_web_lab::middleware::from_fn;

use actix_xml::XmlConfig;

use sqlx::postgres::PgPoolOptions;

use base64::{engine::general_purpose, Engine as _};
use env_logger::Builder;
use log::{info, warn};

mod endpoints;
mod middleware;
mod responder;
mod types;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = types::Config::parse_from_file("config.yml");

    Builder::new().parse_filters(&config.log_level).init();

    info!("Listening on {}:{}", config.listen_addr, config.listen_port);

    let listen_addr = config.listen_addr.clone();
    let listen_port = config.listen_port;

    let digest_key_present = !config.digest_key.is_empty();
    if !digest_key_present {
        warn!("Server digest key is empty. Digests will not be verified and LBP games will probably not accept server responses.");
    }

    let session_key = parse_session_key(&config.session_secret_key);

    let pool = PgPoolOptions::new()
        .connect(&config.db_conn)
        .await
        .expect("Couldn't init database connection pool");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(Arc::new(pool.clone())))
            .service(
                web::scope(&config.base_path)
                    .configure(endpoints::gameserver::cfg)
                    .app_data(Data::new(types::PubKeyStore::new().unwrap()))
                    .wrap(Condition::new(
                        digest_key_present,
                        from_fn(middleware::verify_digest),
                    ))
                    .wrap(
                        SessionMiddleware::builder(
                            RedisActorSessionStore::new(&config.redis_conn),
                            session_key.clone(),
                        )
                        .cookie_name("MM_AUTH".to_string())
                        .cookie_content_security(CookieContentSecurity::Signed)
                        .session_lifecycle(
                            PersistentSession::default().session_ttl(Duration::minutes(30)),
                        )
                        .build(),
                    )
                    .wrap(from_fn(middleware::session_hack))
                    // needed since content types are inconsistent because of fucking course they are (ノಠ益ಠ)ノ彡┻━┻
                    .app_data(
                        XmlConfig::default().content_type(|_| { true })
                    )
                    // oh my fucking god i hate this
                    .app_data(
                        JsonConfig::default().content_type(|_| { true })
                    )
                    .app_data(PayloadConfig::new(config.payload_limit as usize))
            )
            .route(
                "/autodiscover",
                web::get().to(endpoints::autodiscover::autodiscover),
            )
            .wrap(Compress::default())
            .wrap(Logger::default())
    })
    .bind((listen_addr, listen_port))?
    .run()
    .await
}

fn parse_session_key(key: &str) -> Key {
    let base64 = general_purpose::STANDARD;
    let key = base64
        .decode(key)
        .expect("Session secret key isn't valid base64");
    match key.as_slice() {
        [] => {
            info!("Session secret key is empty, generating random one...");
            let key = Key::generate();
            info!("Key: {}", base64.encode(key.master()));
            info!("Copy this into your config.yml file!");
            key
        }
        key => Key::from(key),
    }
}
