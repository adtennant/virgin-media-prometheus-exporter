use actix_web::{
    dev::Body,
    http::header::{ContentType, Header},
    web::Data,
    HttpResponse,
};
use prometheus::{Encoder, Registry, TextEncoder};

pub async fn metrics(registry: Data<Registry>) -> Result<HttpResponse, HttpResponse> {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();

    let metrics = registry.gather();
    encoder.encode(&metrics, &mut buffer).map_err(|e| {
        log::error!("failed to encode metrics: {:?}", e);
        HttpResponse::InternalServerError()
    })?;

    Ok(HttpResponse::Ok()
        .header(ContentType::name(), encoder.format_type())
        .body(Body::from(buffer)))
}
