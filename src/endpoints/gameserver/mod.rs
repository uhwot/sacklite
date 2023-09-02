use actix_web::web;
use actix_web_lab::middleware::from_fn;
use serde::Deserialize;

use crate::middleware;

mod auth;
mod client_config;
mod comment;
mod enter_level;
mod message;
mod publish;
mod resource;
mod tags;
mod user;

// i would have split this shit up, but actix-web doesn't let me ¯\_(ツ)_/¯
pub fn cfg(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .wrap(from_fn(middleware::parse_session))
            // login
            .route("/login", web::post().to(auth::login))
            .route("/goodbye", web::post().to(auth::goodbye))
            // message
            .route("/eula", web::get().to(message::eula))
            .route("/announce", web::get().to(message::announce))
            .route("/notification", web::get().to(message::notification))
            // enter level
            .route(
                "/enterLevel/{slot_type}/{slot_id}",
                web::post().to(enter_level::enter_level),
            )
            // tags
            .route("/tags", web::get().to(tags::tags))
            // user
            .route("/user/{online_id}", web::get().to(user::user))
            .route("/users", web::get().to(user::users))
            .route("/updateUser", web::post().to(user::update_user))
            .route("/get_my_pins", web::get().to(user::get_my_pins))
            .route("/update_my_pins", web::post().to(user::update_my_pins))
            // resource
            .route("/r/{hash}", web::get().to(resource::download))
            .route("/upload/{hash}", web::post().to(resource::upload))
            .route(
                "/filterResources",
                web::post().to(resource::filter_resources),
            )
            .route(
                "/showNotUploaded",
                web::post().to(resource::filter_resources),
            )
            // comment
            .route(
                "/userComments/{online_id}",
                web::get().to(comment::user_comments),
            )
            .route(
                "/postUserComment/{online_id}",
                web::post().to(comment::post_user_comment),
            )
            .route(
                "/deleteUserComment/{online_id}",
                web::post().to(comment::delete_user_comment),
            )
            // publish
            .route("/startPublish", web::post().to(publish::start_publish))
            .route("/publish", web::post().to(publish::publish))
            // client config
            .route(
                "/network_settings.nws",
                web::get().to(client_config::network_settings),
            ),
    );
}

#[derive(Deserialize)]
struct Location {
    x: u16,
    y: u16,
}
