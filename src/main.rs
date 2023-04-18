use actix_identity::IdentityMiddleware;
use actix_web::{
    web,
    App,
    HttpServer,
    middleware::{Logger, Compress, Condition},
    cookie::{Key, time::Duration}
};
use actix_web_lab::middleware::from_fn;
use actix_session::{
    SessionMiddleware,
    storage::RedisActorSessionStore,
    config::{CookieContentSecurity, PersistentSession}
};

use diesel::{r2d2, SqliteConnection};

use env_logger::Builder;
use log::{info, warn};
use base64::{Engine as _, engine::general_purpose};

mod endpoints;
mod types;
mod utils;
mod middleware;
mod responder;
mod db;

type DbPool = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let config = types::config::Config::parse_from_file("config.yml");

    Builder::new().parse_filters(&config.log_level).init();

    info!("Listening on {}:{}", config.listen_addr, config.listen_port);

    let listen_addr = config.listen_addr.clone();
    let listen_port = config.listen_port;

    let digest_key_present = !config.digest_key.is_empty();
    if !digest_key_present {
        warn!("Server digest key is empty. Digests will not be verified and LBP games will probably not accept server responses.");
    }

    let session_key = parse_session_key(&config.session_secret_key);

    let manager = r2d2::ConnectionManager::<SqliteConnection>::new("app.db");
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to SQLite DB file");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope(&config.base_path)
                    .configure(endpoints::gameserver::cfg)
                    .app_data(web::Data::new(types::pub_key_store::PubKeyStore::new().unwrap()))
                    .wrap(Condition::new(digest_key_present, from_fn(middleware::digest::verify_digest)))
                    .wrap(IdentityMiddleware::default())
                    .wrap(
                        SessionMiddleware::builder(
                            RedisActorSessionStore::new(&config.redis_conn),
                            session_key.clone(),
                        )
                        .cookie_name("MM_AUTH".to_string())
                        .cookie_content_security(CookieContentSecurity::Signed)
                        .session_lifecycle(
                            PersistentSession::default().session_ttl(Duration::hours(1))
                        )
                        .build()
                    )
                    .wrap(from_fn(middleware::session_hack::session_hack))
            )
            .route("/autodiscover", web::get().to(endpoints::autodiscover::autodiscover))
            .wrap(Compress::default())
            .wrap(Logger::default())
    })
    .bind((listen_addr, listen_port))?
    .run()
    .await
}

fn parse_session_key(key: &str) -> Key {
    let base64 = general_purpose::STANDARD;
    let key = base64.decode(key).expect("Session secret key isn't valid base64");
    match key.as_slice() {
        [] => {
            info!("Session secret key is empty, generating random one...");
            let key = Key::generate();
            info!("Key: {}", base64.encode(key.master()));
            info!("Copy this into your config.yml file!");
            key
        },
        key => Key::from(key),
    }
}