use axum::{
    async_trait,
    extract::{FromRequest, Request},
};
use axum_serde::Rejection;
use http::HeaderValue;
use quick_xml::DeError;

// mostly shamelessly stolen from:
// https://github.com/tokio-rs/axum/blob/axum-v0.7.3/examples/customize-extractor-error/src/custom_extractor.rs
// this is needed because axum actually follows standards properly,
// and doesn't accept post requests with invalid content-type headers.
// meanwhile, LBP sends random crap as the content type, such as fucking "application/x-www-form-urlencoded" :/

#[derive(Debug, Clone, Copy, Default)]
pub struct Xml<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for Xml<T>
where
    axum_serde::Xml<T>: FromRequest<S, Rejection = Rejection<DeError>>,
    S: Send + Sync,
{
    type Rejection = Rejection<DeError>;

    async fn from_request(mut req: Request, state: &S) -> Result<Self, Self::Rejection> {
        req.headers_mut().insert("content-type", HeaderValue::from_static("application/xml"));
        axum_serde::Xml::<T>::from_request(req, state)
            .await
            .map(|v| Self(v.0))
    }
}

impl<T> std::ops::Deref for Xml<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Xml<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}