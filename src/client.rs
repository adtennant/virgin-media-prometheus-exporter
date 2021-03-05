use crate::snmp::List;

use anyhow::Result;
use reqwest::Client;
use std::net::IpAddr;

pub struct VirginHubClient {
    client: Client,
    hub_ip: IpAddr,
}

impl VirginHubClient {
    pub fn new(hub_ip: IpAddr) -> Self {
        VirginHubClient {
            client: Client::new(),
            hub_ip,
        }
    }

    pub async fn get_router_status(&self) -> Result<List> {
        let url = format!("http://{}/getRouterStatus", self.hub_ip);

        let response = self.client.get(&url).send().await?;

        let router_status: List = response.json().await?;
        Ok(router_status)
    }
}
