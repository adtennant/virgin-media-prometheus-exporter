use anyhow::Result;
use config::{Config, File};
use serde_aux::field_attributes::deserialize_number_from_string;
use std::net::IpAddr;

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub hub_ip: IpAddr,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

pub fn load_settings() -> Result<Settings> {
    let mut config = Config::default();
    config.merge(File::with_name("config/default").required(true))?;

    config.try_into().map_err(|e| e.into())
}
