use actix_web::web;

mod message;
mod auth;
mod enter_level;
mod tags;
mod news;
mod client_config;

// i would have split this shit up, but actix-web doesn't let me ¯\_(ツ)_/¯
pub fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            // login
            .route("/login", web::post().to(auth::login))
            .route("/goodbye", web::post().to(auth::goodbye))
            // message
            .route("/eula", web::get().to(message::eula))
            .route("/announce", web::get().to(message::announce))
            .route("/notification", web::get().to(message::notification))
            // enter level
            .route("/enterLevel/{slot_type}/{slot_id}", web::post().to(enter_level::enter_level))
            // tags
            .route("/tags", web::get().to(tags::tags))
            // news
            .route("/news", web::get().to(news::news))
            // client config
            .route("/network_settings.nws", web::get().to(client_config::network_settings))
    );
}