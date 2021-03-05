use actix_web::{
    dev::Body,
    http::header::{ContentType, Header},
    web::{self, Data},
    HttpResponse,
};
use prometheus::{Encoder, Registry, TextEncoder};

#[tracing::instrument(skip(registry))]
pub async fn metrics(registry: Data<Registry>) -> HttpResponse {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();

    let metrics = web::block(move || registry.gather()).await.unwrap();
    encoder.encode(&metrics, &mut buffer).unwrap();

    HttpResponse::Ok()
        .append_header((ContentType::name(), encoder.format_type()))
        .body(Body::from(buffer))
}
