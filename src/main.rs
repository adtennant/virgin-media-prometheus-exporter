use virgin_media_prometheus_exporter::{app::Application, settings};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = settings::load_settings().expect("failed to load settings");

    let app = Application::build(settings).expect("failed to start application");
    app.run().await
}
