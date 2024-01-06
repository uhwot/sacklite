use axum::{extract::{Request, State}, middleware::Next, response::{Response, IntoResponse}, http::{StatusCode, HeaderValue}, body::{Body, Bytes}};
use axum_extra::extract::CookieJar;
use http_body_util::BodyExt;
use sha1::{Digest, Sha1};
use tracing::debug;

use crate::types::Config;

fn calc_digest(path: &str, mm_auth: &str, body: Bytes, digest_key: &str) -> String {
    let mut sha1 = Sha1::new();

    sha1.update(body);
    sha1.update(mm_auth);
    sha1.update(path);
    sha1.update(digest_key);

    format!("{:x}", sha1.finalize())
}

/*
    exempt paths:
    /login
    /eula
    /announce
    /status
    /farc_hashes
    /t_conf
    /network_settings.nws
    /ChallengeConfig.xml
*/
pub async fn verify_digest(
    State(config): State<Config>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, Response> {
    let full_path = config.base_path + req.uri().path();

    // game calculates the digest with or without the body depending on if it's a resource upload
    let exclude_body = req.uri().path().starts_with("/upload/");
    let digest_header_key = match exclude_body {
        false => "X-Digest-A",
        true => "X-Digest-B",
    };

    let cookies = CookieJar::from_headers(req.headers());
    let mm_auth = cookies.get("MM_AUTH").map_or("", |c| c.value());

    let headers = req.headers().clone();
    let client_digest = headers
        .get(digest_header_key)
        .ok_or_else(|| StatusCode::FORBIDDEN.into_response())?
        .to_str()
        .map_err(|_| StatusCode::FORBIDDEN.into_response())?;

    let (req_digest, req) = match exclude_body {
        true => (calc_digest(&full_path, mm_auth, Bytes::new(), &config.digest_key), req),
        false => buffer_request_body(req, &full_path, mm_auth, &config.digest_key).await?,
    };

    match req_digest == client_digest {
        true => Ok(next.run(req).await),
        false => {
            debug!("Digest is invalid, ignoring request");
            debug!("digest: {req_digest}");
            debug!("client digest: {client_digest}");
            Err(StatusCode::FORBIDDEN.into_response())
        },
    }
}

// https://github.com/tokio-rs/axum/blob/axum-v0.7.3/examples/consume-body-in-extractor-or-middleware/src/main.rs
// the trick is to take the request apart, buffer the body, do what you need to do, then put
// the request back together
async fn buffer_request_body(request: Request, full_path: &str, mm_auth: &str, digest_key: &str) -> Result<(String, Request), Response> {
    let (parts, body) = request.into_parts();

    // this wont work if the body is an long running stream
    let bytes = body
        .collect()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?
        .to_bytes();

    let digest = calc_digest(full_path, mm_auth, bytes.clone(), digest_key);

    Ok((digest, Request::from_parts(parts, Body::from(bytes))))
}

pub async fn send_digest(
    State(digest_key): State<String>,
    req: Request,
    next: Next,
) -> Response {
    let uri = req.uri().clone();
    let cookies = CookieJar::from_headers(req.headers());
    let mm_auth = cookies.get("MM_AUTH").map_or("", |c| c.value());

    let mut resp = next.run(req).await;

    let digest = calc_digest(uri.path(), mm_auth, Bytes::new(), &digest_key);
    resp.headers_mut().insert("X-Digest-B", HeaderValue::from_str(&digest).unwrap());
    resp
}
