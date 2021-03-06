use crate::client::VirginHubClient;

mod configuration_metrics;
mod downstream_metrics;
mod status_metrics;
mod upstream_metrics;

use anyhow::Result;
use configuration_metrics::ConfigurationMetrics;
use downstream_metrics::DownstreamMetrics;
use prometheus::{
    core::{AtomicU64, Desc, GenericGauge, GenericGaugeVec},
    proto::MetricFamily,
};
use status_metrics::StatusMetrics;
use upstream_metrics::UpstreamMetrics;

pub type UIntGauge = GenericGauge<AtomicU64>;
pub type UIntGaugeVec = GenericGaugeVec<AtomicU64>;

pub struct Collector {
    client: VirginHubClient,

    up: UIntGauge,
    status_metrics: StatusMetrics,
    downstream_metrics: DownstreamMetrics,
    upstream_metrics: UpstreamMetrics,
    configuration_metrics: ConfigurationMetrics,
}

impl Collector {
    pub fn new(client: VirginHubClient) -> Result<Self> {
        Ok(Collector {
            client,

            up: UIntGauge::new("up", "Whether the Virgin Media scrape was successful.")?,

            status_metrics: StatusMetrics::new()?,
            downstream_metrics: DownstreamMetrics::new()?,
            upstream_metrics: UpstreamMetrics::new()?,
            configuration_metrics: ConfigurationMetrics::new()?,
        })
    }

    fn update(&self) -> Result<()> {
        let router_status = self.client.get_router_status()?;

        self.status_metrics.set(&router_status)?;
        self.downstream_metrics.set(&router_status)?;
        self.upstream_metrics.set(&router_status)?;
        self.configuration_metrics.set(&router_status)?;

        Ok(())
    }
}

impl prometheus::core::Collector for Collector {
    fn desc(&self) -> Vec<&Desc> {
        vec![
            self.up.desc(),
            self.status_metrics.desc(),
            self.downstream_metrics.desc(),
            self.upstream_metrics.desc(),
            self.configuration_metrics.desc(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn collect(&self) -> Vec<MetricFamily> {
        if let Err(e) = self.update() {
            log::error!("error updating metrics: {:?}", e);

            self.up.set(0);
            return self.up.collect();
        }

        self.up.set(1);

        vec![
            self.up.collect(),
            self.status_metrics.collect(),
            self.downstream_metrics.collect(),
            self.upstream_metrics.collect(),
            self.configuration_metrics.collect(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}
