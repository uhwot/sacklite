use axum::{response::IntoResponse, http::{Response, header, HeaderValue}, body::Body};
use maud::Markup;

/// An XML response.
///
/// Will automatically get `Content-Type: text/xml`.
#[derive(Clone, Debug)]
#[must_use]
pub struct Xml(pub Markup);

impl IntoResponse for Xml
{
    fn into_response(self) -> Response<Body> {
        (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime::TEXT_XML.as_ref()),
            )],
            self.0.into_string(),
        )
            .into_response()
    }
}

impl From<Markup> for Xml {
    fn from(inner: Markup) -> Self {
        Self(inner)
    }
}