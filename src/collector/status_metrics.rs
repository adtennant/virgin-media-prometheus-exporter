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

/*#[derive(Debug)]
pub enum AcquiredDownstreamStatus {
    Scanning,
    Ranging,
    Locked,
}

impl FromStr for AcquiredDownstreamStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use AcquiredDownstreamStatus::*;

        Ok(match s {
            "1" => Scanning,
            "2" => Ranging,
            _ => Locked,
        })
    }
}

#[derive(Debug)]
pub enum RangedUpstreamStatus {
    None,
    Ranging,
    Locked,
}

impl FromStr for RangedUpstreamStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use RangedUpstreamStatus::*;

        Ok(match s.parse::<usize>()? {
            1 => None,
            3 => Ranging,
            x if x > 3 => Locked,
            _ => bail!("invalid upstream status: {}", s),
        })
    }
}

#[derive(Debug)]
pub enum ProvisioningState {
    Online,
    Offline,
}

impl FromStr for ProvisioningState {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ProvisioningState::*;

        Ok(match s.parse::<usize>()? {
            x if x > 6 => Online,
            _ => Offline,
        })
    }
}*/

pub struct StatusMetrics {
    acquired_down_channel_frequency: UIntGauge,
    // acquired_down_channel_status: UIntGauge,
    ranged_up_channel_frequency: UIntGauge,
    // ranged_up_channel_status: UIntGauge,
    provisioning_state: UIntGauge,
}

impl StatusMetrics {
    pub fn new() -> Result<Self> {
        Ok(StatusMetrics {
            acquired_down_channel_frequency: UIntGauge::new(
                "acquired_down_channel_frequency",
                "Acquired Downstream Channel (Hz)",
            )?,
            /*acquired_down_channel_status: UIntGauge::new(
                "acquired_down_channel_status",
                "Acquired Downstream Channel Status",
            )?,*/
            ranged_up_channel_frequency: UIntGauge::new(
                "ranged_up_channel_frequency",
                "Ranged Upstream Channel (Hz)",
            )?,
            /*ranged_up_channel_status: UIntGauge::new(
                "ranged_up_channel_status",
                "Ranged Upstream Channel Status",
            )?,*/
            provisioning_state: UIntGauge::new("provisioning_state", "Provisioning State")?,
        })
    }

    pub fn set(&self, router_status: &List) -> Result<()> {
        let docsis_reg_status = router_status
            .parse_scalar(&ARRIS_CM_DOC30_SW_REGISTRATION_STATE)
            .unwrap();

        /*self.acquired_down_channel_status
            .set(docsis_reg_status.parse()?);
        self.ranged_up_channel_status
            .set(docsis_reg_status.parse()?);*/
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
            // self.acquired_down_channel_status.desc(),
            self.ranged_up_channel_frequency.desc(),
            // self.ranged_up_channel_status.desc(),
            self.provisioning_state.desc(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn collect(&self) -> Vec<MetricFamily> {
        vec![
            self.acquired_down_channel_frequency.collect(),
            // self.acquired_down_channel_status.collect(),
            self.ranged_up_channel_frequency.collect(),
            // self.ranged_up_channel_status.collect(),
            self.provisioning_state.collect(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}
