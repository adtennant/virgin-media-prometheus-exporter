mod app;
mod client;
mod collector;
mod routes;
mod settings;
mod snmp;

use app::Application;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    LogTracer::init().expect("failed to set logger");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer =
        BunyanFormattingLayer::new("virgin-media-prometheus-exporter".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    tracing::subscriber::set_global_default(subscriber).expect("failed to set subscriber");

    let settings = settings::load_settings().expect("failed to load settings");

    let app = Application::build(settings).expect("failed to start application");
    app.run().await
}
