use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, Request}, response::IntoResponse,
};
use http::HeaderValue;
use serde::Serialize;

// mostly shamelessly stolen from:
// https://github.com/tokio-rs/axum/blob/axum-v0.7.3/examples/customize-extractor-error/src/custom_extractor.rs
// this is needed because axum actually follows standards properly,
// and doesn't accept post requests with invalid content-type headers.
// meanwhile, LBP sends random crap as the content type, such as fucking "application/x-www-form-urlencoded" :/

#[derive(Debug, Clone, Copy, Default)]
pub struct Json<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for Json<T>
where
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = JsonRejection;

    async fn from_request(mut req: Request, state: &S) -> Result<Self, Self::Rejection> {
        req.headers_mut().insert("content-type", HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()));
        axum::Json::<T>::from_request(req, state)
            .await
            .map(|v| Self(v.0))
    }
}

// We implement `IntoResponse` for our extractor so it can be used as a response
impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> axum::response::Response {
        let Self(value) = self;
        axum::Json(value).into_response()
    }
}

impl<T> std::ops::Deref for Json<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Json<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}