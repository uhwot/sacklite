use actix_web::Responder;
use maud::html as xml;

use crate::responder::Xml;

pub async fn news() -> impl Responder {
    // TODO: placeholder, implement this crap once the format is released
    Xml(xml!(
        news {
            subcategory {
                item {
                    content {
                        frame width="512" {
                            title { "fuck" }
                            item width="512" {
                                slot type="developer" {
                                    id { "42883" }
                                }
                                npHandle icon="" { "uh_wot" }
                                content { "ass" }
                            }
                        }
                    }
                }
            }
        }
    )
    .into_string())
}
