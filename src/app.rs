use crate::client::VirginHubClient;
use crate::collector::Collector;
use crate::routes::{health_check, metrics};
use crate::settings::Settings;

use actix_web::dev::Server;
use actix_web::{middleware, web, App, HttpServer};
use anyhow::Result;
use std::net::TcpListener;

pub struct Application {
    server: Server,
}

impl Application {
    pub fn build(settings: Settings) -> Result<Self> {
        let client = VirginHubClient::new(settings.hub_ip);

        let collector = Collector::new(client)?;
        prometheus::register(Box::new(collector))?;

        let address = format!("0.0.0.0:{}", settings.port);
        let listener = TcpListener::bind(address)?;

        let server = HttpServer::new(move || {
            App::new()
                .wrap(middleware::Compress::default())
                .route("/health", web::get().to(health_check))
                .route("/metrics", web::get().to(metrics))
        })
        .listen(listener)?
        .run();

        Ok(Application { server })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
