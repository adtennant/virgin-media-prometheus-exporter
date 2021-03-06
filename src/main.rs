mod app;
mod client;
mod collector;
mod routes;
mod settings;
mod snmp;

use app::Application;
use env_logger::Env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let settings = settings::load_settings().expect("failed to load settings");

    let app = Application::build(settings).expect("failed to start application");
    app.run().await
}
