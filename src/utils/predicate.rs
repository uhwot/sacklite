use http::header;
use http_body::Body;
use tower_http::compression::Predicate;

// modified from
// https://docs.rs/tower-http/0.5.0/src/tower_http/compression/predicate.rs.html#188-191

/// Predicate that only allows responses with a specific `content-type` to be compressed.
#[derive(Clone, Debug)]
pub struct ContentType {
    content_type: &'static str,
}

impl ContentType {
    /// Create a new `ContentType` from a static string.
    pub const fn const_new(content_type: &'static str) -> Self {
        Self {
            content_type,
        }
    }
}

impl Predicate for ContentType {
    fn should_compress<B>(&self, response: &http::Response<B>) -> bool
    where
        B: Body,
    {
        content_type(response).starts_with(self.content_type)
    }
}

fn content_type<B>(response: &http::Response<B>) -> &str {
    response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default()
}
