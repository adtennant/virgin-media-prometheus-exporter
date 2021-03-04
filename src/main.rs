mod app;
mod client;
mod collector;
mod routes;
mod settings;
mod snmp;

use app::Application;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = settings::load_settings().expect("failed to load settings");

    let app = Application::build(settings).expect("failed to start application");
    app.run().await
}
