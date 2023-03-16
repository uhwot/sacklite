use actix_web::web;

mod message;
mod tags;
mod login;
mod client_config;

// i would have split this shit up, but actix-web doesn't let me ¯\_(ツ)_/¯
pub fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            // login
            .route("/login", web::post().to(login::login))
            // message
            .route("/eula", web::get().to(message::eula))
            .route("/announce", web::get().to(message::announce))
            .route("/notification", web::get().to(message::notification))
            // tags
            .route("/tags", web::get().to(tags::tags))
            // client config
            .route("/network_settings.nws", web::get().to(client_config::network_settings))
    );
}