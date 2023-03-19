use actix_web::{
    body::{EitherBody, MessageBody, BodySize},
    dev::{self, ServiceResponse},
    web::{Bytes, Data},
    Error, HttpResponse,
};
use actix_web_lab::middleware::Next;
use actix_http::{StatusCode, h1, header::{HeaderValue, HeaderName}};

use log::{info, debug};
use sha1::{Sha1, Digest};

use crate::types::config::Config;

const EXEMPT_PATHS: [&str; 8] = [
    "/login",
    "/eula",
    "/announce",
    "/status",
    "/farc_hashes",
    "/t_conf",
    "/network_settings.nws",
    "/ChallengeConfig.xml",
];

fn calc_digest(path: &str, mm_auth: &str, body: &Bytes, digest_key: &str) -> String {
    let mut sha1 = Sha1::new();

    sha1.update(body);
    sha1.update(mm_auth);
    sha1.update(path);
    sha1.update(digest_key);

    format!("{:x}", sha1.finalize())
}

pub async fn verify_digest(mut req: dev::ServiceRequest, next: Next<impl MessageBody + 'static>) -> Result<ServiceResponse<EitherBody<impl MessageBody>>, Error> {
    let config = req.app_data::<Data<Config>>().unwrap().clone();

    let path = req.path().to_owned();

    let mut stripped_path = path.as_str();
    if stripped_path.starts_with(&config.base_path) {
        stripped_path = &stripped_path[config.base_path.len()..]
    }

    // game calculates the digest with or without the body depending on if it's a resource upload
    let exclude_body = stripped_path.starts_with("/upload/");
    let digest_header_key = match exclude_body {
        false => "X-Digest-A",
        true => "X-Digest-B",
    };

    let body = match exclude_body {
        true => Bytes::new(),
        false => req.extract::<Bytes>().await.unwrap(),
    };

    let mm_auth = match req.cookie("MM_AUTH") {
        Some(cookie) => cookie.value().to_owned(),
        None => String::new(),
    };

    let req_digest = calc_digest(&path, &mm_auth, &body, &config.digest_key);

    // the game doesn't start sending digests until after the announcement,
    // so if the request is before that we accept it anyways
    let mut digest_pass = !config.verify_client_digest || EXEMPT_PATHS.contains(&stripped_path);

    if !digest_pass {
        let client_digest = req.headers().get(digest_header_key);
        match client_digest {
            Some(cl_digest) => {
                digest_pass = req_digest == cl_digest.to_str().unwrap();
                if !digest_pass {
                    info!("Invalid digest, ignoring request");
                    debug!("digest: {req_digest}");
                    debug!("client digest: {cl_digest:?}");
                }
            },
            None => info!("Missing digest, ignoring request")
        }
    }

    if !exclude_body {
        req.set_payload(bytes_to_payload(body));
    }

    if !digest_pass {
        let (req, _pl) = req.into_parts();
        let res = HttpResponse::new(StatusCode::FORBIDDEN).map_into_right_body();
        return Ok(ServiceResponse::new(req, res));
    }

    let mut res = next.call(req).await?;

    res.headers_mut().insert(HeaderName::from_static("x-digest-b"), HeaderValue::from_str(&req_digest).unwrap());

    // code stolen from here:
    // https://github.com/chriswk/actix-middleware-etag/blob/fe10145fa730d9c45deb7e05c594ad5760b9761a/src/lib.rs#L103
    let mut payload = Bytes::new();
    let mut res = res.map_body(|_h, body| match body.size() {
        BodySize::Sized(_size) => {
            let bytes = body.try_into_bytes().unwrap_or_else(|_| Bytes::new());
            payload = bytes.clone();
            bytes.clone().boxed()
        }
        _ => body.boxed(),
    });

    let resp_digest = calc_digest(&path, &mm_auth, &payload, &config.digest_key);
    res.headers_mut().insert(HeaderName::from_static("x-digest-a"), HeaderValue::from_str(&resp_digest).unwrap());

    Ok(res.map_into_left_body())
}

fn bytes_to_payload(buf: Bytes) -> dev::Payload {
    let (_, mut pl) = h1::Payload::create(true);
    pl.unread_data(buf);
    dev::Payload::from(pl)
}