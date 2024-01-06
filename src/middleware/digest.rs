use axum::{extract::{Request, State}, middleware::Next, response::{Response, IntoResponse}, http::{StatusCode, HeaderValue}, body::{Body, Bytes}};
use axum_extra::extract::CookieJar;
use http_body_util::BodyExt;
use sha1::{Digest, Sha1};
use tracing::debug;

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
    State((digest_key, base_path)): State<(String, String)>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, Response> {
    let full_path = base_path + req.uri().path();

    // game calculates the digest with or without the body depending on if it's a resource upload
    let include_body = !req.uri().path().starts_with("/upload/");
    let digest_header_key = match include_body {
        true => "X-Digest-A",
        false => "X-Digest-B",
    };

    let cookies = CookieJar::from_headers(req.headers());
    let mm_auth = cookies.get("MM_AUTH").map_or("", |c| c.value());

    let headers = req.headers().clone();
    let client_digest = headers
        .get(digest_header_key)
        .ok_or_else(|| StatusCode::FORBIDDEN.into_response())?
        .to_str()
        .map_err(|_| StatusCode::FORBIDDEN.into_response())?;

    let (expected_digest, req) = match include_body {
        true => {
            let (parts, body) = req.into_parts();
            let (digest, body) = buffer_body(body, &full_path, mm_auth, &digest_key).await?;
            (digest, Request::from_parts(parts, body))
        },
        false => (calc_digest(&full_path, mm_auth, Bytes::new(), &digest_key), req),
    };

    let header_val = HeaderValue::from_str(&expected_digest).unwrap();

    match expected_digest == client_digest {
        true => {
            let mut resp = next.run(req).await;
            resp.headers_mut().insert("X-Digest-B", header_val);
            Ok(resp)
        },
        false => {
            debug!("Digest is invalid, ignoring request");
            debug!("expected digest: {expected_digest}");
            debug!("client digest: {client_digest}");
            let mut resp = StatusCode::FORBIDDEN.into_response();
            resp.headers_mut().insert("X-Digest-B", header_val);
            Err(resp)
        },
    }
}

pub async fn send_digest(
    State((digest_key, base_path)): State<(String, String)>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, Response> {
    let full_path = base_path + req.uri().path();
    let cookies = CookieJar::from_headers(req.headers());
    let mm_auth = cookies.get("MM_AUTH").map_or("", |c| c.value());
    let include_body = !req.uri().path().starts_with("/upload/");

    let resp = next.run(req).await;
    let (digest, mut resp) = match include_body {
        true => {
            let (parts, body) = resp.into_parts();
            let (digest, body) = buffer_body(body, &full_path, mm_auth, &digest_key).await?;
            (digest, Response::from_parts(parts, body))
        },
        false => (calc_digest(&full_path, mm_auth, Bytes::new(), &digest_key), resp),
    };

    resp.headers_mut().insert("X-Digest-A", HeaderValue::from_str(&digest).unwrap());
    Ok(resp)
}

// https://github.com/tokio-rs/axum/blob/axum-v0.7.3/examples/consume-body-in-extractor-or-middleware/src/main.rs
// the trick is to take the request apart, buffer the body, do what you need to do, then put
// the request back together
async fn buffer_body(body: Body, full_path: &str, mm_auth: &str, digest_key: &str) -> Result<(String, Body), Response> {
    // this wont work if the body is an long running stream
    let bytes = body
        .collect()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?
        .to_bytes();

    let digest = calc_digest(full_path, mm_auth, bytes.clone(), digest_key);

    Ok((digest, Body::from(bytes)))
}
