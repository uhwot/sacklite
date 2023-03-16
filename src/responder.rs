use actix_web::{
    http::{
        header::{self, ContentType, TryIntoHeaderValue},
        StatusCode,
    },
    HttpRequest, HttpResponse, Responder,
};

/// An XML responder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Xml(pub String);

impl Xml {
    /// Constructs a new `Xml` responder.
    pub fn new(xml: impl Into<String>) -> Self {
        Self(xml.into())
    }
}

impl Responder for Xml {
    type Body = String;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let mut res = HttpResponse::with_body(StatusCode::OK, self.0);
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            ContentType::xml().try_into_value().unwrap(),
        );
        res
    }
}
