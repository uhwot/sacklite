use actix_web::{web, App, HttpServer, middleware::{Logger, Compress, Condition}};
use actix_web_lab::middleware::from_fn;
use env_logger::Builder;
use log::{info, warn};

mod endpoints;
mod types;
mod digest;
mod utils;
mod responder;

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

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .service(
                web::scope(&config.base_path)
                    .configure(endpoints::gameserver::cfg)
                    .wrap(Condition::new(digest_key_present, from_fn(digest::verify_digest)))
            )
            .route("/autodiscover", web::get().to(endpoints::autodiscover::autodiscover))
            .wrap(Compress::default())
            .wrap(Logger::default())
    })
    .bind((listen_addr, listen_port))?
    .run()
    .await
}