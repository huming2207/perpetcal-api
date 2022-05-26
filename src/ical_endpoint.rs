use actix_web::{web, Responder, HttpResponse, http::header::ContentType};
use serde::Deserialize;

use crate::ical_json::CalendarItem;

#[derive(Deserialize)]
pub struct IcalRequest {
    feed: String,
    tzid: String,
    dtfmt: String,
    sort: bool,
    limit: usize,
}

pub async fn ical_handler(form: web::Json<IcalRequest>) -> impl Responder {
    match CalendarItem::from_ical_url(&form.feed, form.tzid.clone(), &form.dtfmt, form.sort, form.limit).await {
        Ok(ical) => {
            match serde_json::to_string(&ical) {
                Ok(str) => {
                    return HttpResponse::Ok()
                        .content_type(ContentType::json())
                        .body(str)
                }
                Err(err) => {
                    return HttpResponse::InternalServerError()
                        .content_type(ContentType::plaintext())
                        .body(format!("Failed to encode JSON, reason: {}", err.to_string()))
                }
            };
        }
        Err(err) => {
            return HttpResponse::BadRequest()
                .content_type(ContentType::plaintext())
                .body(format!("Failed to convert, reason: {}", err.to_string()))
        }
    }
} 