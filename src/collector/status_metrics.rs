use super::UIntGauge;
use crate::snmp::{List, OID};

use anyhow::{Context, Result};
use prometheus::{
    core::{Collector, Desc},
    proto::MetricFamily,
};

const DOCS_IF_DOWNSTREAM_CHANNEL_TABLE: OID = OID::new("1.3.6.1.2.1.10.127.1.1.1"); // docsIfDownstreamChannelTable
const DOCS_IF_DOWN_CHANNEL_FREQUENCY: OID = OID::new("1.3.6.1.2.1.10.127.1.1.1.1.2"); // docsIfDownChannelFrequency

const DOCS_IF_UPSTREAM_CHANNEL_TABLE: OID = OID::new("1.3.6.1.2.1.10.127.1.1.2"); // docsIfUpstreamChannelTable
const DOCS_IF_UP_CHANNEL_FREQUENCY: OID = OID::new("1.3.6.1.2.1.10.127.1.1.2.1.2"); // docsIfUpChannelFrequency

const ARRIS_CM_DOC30_SW_REGISTRATION_STATE: OID = OID::new("1.3.6.1.4.1.4115.1.3.4.1.5.9"); // arrisCmDoc30SwRegistrationState

pub struct StatusMetrics {
    acquired_down_channel_frequency: UIntGauge,
    ranged_up_channel_frequency: UIntGauge,
    provisioning_state: UIntGauge,
}

impl StatusMetrics {
    pub fn new() -> Result<Self> {
        Ok(StatusMetrics {
            acquired_down_channel_frequency: UIntGauge::new(
                "acquired_down_channel_frequency",
                "Acquired Downstream Channel (Hz)",
            )?,
            ranged_up_channel_frequency: UIntGauge::new(
                "ranged_up_channel_frequency",
                "Ranged Upstream Channel (Hz)",
            )?,
            provisioning_state: UIntGauge::new("provisioning_state", "Provisioning State")?,
        })
    }

    pub fn set(&self, router_status: &List) -> Result<()> {
        let docsis_reg_status =
            router_status.parse_scalar(&ARRIS_CM_DOC30_SW_REGISTRATION_STATE)?;
        self.provisioning_state.set(docsis_reg_status);

        let downstream_channel_table =
            router_status.get_table(&DOCS_IF_DOWNSTREAM_CHANNEL_TABLE)?;
        let acquired_down_channel_frequency = downstream_channel_table
            .get("1")
            .context("missing acquired downstream channel")?
            .get_column(&DOCS_IF_DOWN_CHANNEL_FREQUENCY)
            .context("missing acquired downstream channel frequency")?;
        self.acquired_down_channel_frequency
            .set(acquired_down_channel_frequency.parse()?);

        let upstream_channel_table = router_status.get_table(&DOCS_IF_UPSTREAM_CHANNEL_TABLE)?;
        let ranged_up_channel_frequency = upstream_channel_table
            .get("1")
            .context("missing ranged upstream channel")?
            .get_column(&DOCS_IF_UP_CHANNEL_FREQUENCY)
            .context("missing ranged upstream channel frequency")?;
        self.ranged_up_channel_frequency
            .set(ranged_up_channel_frequency.parse()?);

        Ok(())
    }
}

impl Collector for StatusMetrics {
    fn desc(&self) -> Vec<&Desc> {
        vec![
            self.acquired_down_channel_frequency.desc(),
            self.ranged_up_channel_frequency.desc(),
            self.provisioning_state.desc(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn collect(&self) -> Vec<MetricFamily> {
        vec![
            self.acquired_down_channel_frequency.collect(),
            self.ranged_up_channel_frequency.collect(),
            self.provisioning_state.collect(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}
