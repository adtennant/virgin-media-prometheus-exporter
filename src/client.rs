use crate::snmp::List;

use anyhow::Result;
use reqwest::blocking::Client;
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

    pub fn get_router_status(&self) -> Result<List> {
        let url = format!("http://{}/getRouterStatus", self.hub_ip);

        let response = self.client.get(&url).send()?;

        let router_status: List = response.json()?;
        Ok(router_status)
    }
}
