use crate::client::VirginHubClient;
use crate::collector::Collector;
use crate::routes::{health_check, metrics};
use crate::settings::Settings;

use actix_web::dev::Server;
use actix_web::{
    middleware,
    web::{self, Data},
    App, HttpServer,
};
use anyhow::Result;
use prometheus::Registry;
use std::net::TcpListener;

const REGISTRY_PREFIX: &str = "virgin_media";

pub struct Application {
    server: Server,
}

impl Application {
    pub fn build(settings: Settings) -> Result<Self> {
        let client = VirginHubClient::new(settings.hub_ip);

        let collector = Collector::new(client)?;
        let registry = Registry::new_custom(Some(String::from(REGISTRY_PREFIX)), None)?;
        registry.register(Box::new(collector))?;

        let registry = Data::new(registry);

        let address = format!("0.0.0.0:{}", settings.port);
        let listener = TcpListener::bind(address)?;

        let server = HttpServer::new(move || {
            App::new()
                .wrap(middleware::Compress::default())
                .route("/health", web::get().to(health_check))
                .route("/metrics", web::get().to(metrics))
                .app_data(registry.clone())
        })
        .listen(listener)?
        .run();

        Ok(Application { server })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
