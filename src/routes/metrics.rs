use actix_web::{
    dev::Body,
    http::header::{ContentType, Header},
    HttpResponse,
};
use prometheus::{Encoder, TextEncoder};

pub async fn metrics() -> HttpResponse {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();

    let metrics = prometheus::gather();
    encoder.encode(&metrics, &mut buffer).unwrap();

    HttpResponse::Ok()
        .header(ContentType::name(), encoder.format_type())
        .body(Body::from(buffer))
}
