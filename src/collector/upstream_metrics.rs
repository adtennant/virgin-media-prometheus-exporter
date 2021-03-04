use super::UIntGaugeVec;
use crate::snmp::{List, Table, TableEntry, OID};

use anyhow::{bail, Result};
use prometheus::{
    core::{Collector, Desc},
    proto::MetricFamily,
    GaugeVec, Opts,
};
use std::{convert::TryFrom, str::FromStr};

const DOCS_IF_UPSTREAM_CHANNEL_TABLE: OID = OID::new("1.3.6.1.2.1.10.127.1.1.2"); // docsIfUpstreamChannelTable
const DOCS_IF_UP_CHANNEL_ID: OID = OID::new("1.3.6.1.2.1.10.127.1.1.2.1.1"); // docsIfUpChannelId
const DOCS_IF_UP_CHANNEL_FREQUENCY: OID = OID::new("1.3.6.1.2.1.10.127.1.1.2.1.2"); // docsIfUpChannelFrequency
const DOCS_IF_UP_CHANNEL_TYPE: OID = OID::new("1.3.6.1.2.1.10.127.1.1.2.1.15"); // docsIfUpChannelType

#[derive(Debug)]
pub enum UpstreamChannelType {
    TDMA = 1,
    ATDMA = 2,
    SCDMA = 3,
    #[allow(non_camel_case_types)]
    TDDMA_ATDMA = 4,
}

impl FromStr for UpstreamChannelType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use UpstreamChannelType::*;

        Ok(match s {
            "1" => TDMA,
            "2" => ATDMA,
            "3" => SCDMA,
            "4" => TDDMA_ATDMA,
            _ => bail!("unknown upstream channel type"),
        })
    }
}

#[derive(Debug)]
struct UpstreamChannel {
    up_channel_id: u64,
    up_channel_frequency: u64,
    up_channel_type: f64,
}

impl TryFrom<TableEntry> for UpstreamChannel {
    type Error = anyhow::Error;

    fn try_from(entry: TableEntry) -> Result<Self, Self::Error> {
        Ok(UpstreamChannel {
            up_channel_id: entry.parse_column(&DOCS_IF_UP_CHANNEL_ID)?,
            up_channel_frequency: entry.parse_column(&DOCS_IF_UP_CHANNEL_FREQUENCY)?,
            up_channel_type: entry.parse_column(&DOCS_IF_UP_CHANNEL_TYPE)?,
        })
    }
}

const ARRIS_CM_DOC30_IF_UPSTREAM_CHANNEL_EXTENDED_TABLE: OID =
    OID::new("1.3.6.1.4.1.4115.1.3.4.1.9.2"); // arrisCmDoc30IfUpstreamChannelExtendedTable
const AR_CM_DOC30_IF_UP_CHANNEL_EXTENDED_SYMBOL_RATE: OID =
    OID::new("1.3.6.1.4.1.4115.1.3.4.1.9.2.1.2"); // arrisCmDoc30IfUpChannelExtendedSymbolRate
const AR_CM_DOC30_IF_UP_CHANNEL_EXTENDED_MODULATION: OID =
    OID::new("1.3.6.1.4.1.4115.1.3.4.1.9.2.1.3"); // arrisCmDoc30IfUpChannelExtendedModulation

#[derive(Clone, Copy, Debug)]
pub enum UpstreamChannelModulation {
    QPSK = 1,
    QAM8 = 2,
    QAM16 = 3,
    QAM32 = 4,
    QAM64 = 5,
    QAM128 = 6,
    QAM256 = 7,
}

impl FromStr for UpstreamChannelModulation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use UpstreamChannelModulation::*;

        Ok(match s {
            "1" => QPSK,
            "2" => QAM8,
            "3" => QAM16,
            "4" => QAM32,
            "5" => QAM64,
            "6" => QAM128,
            "7" => QAM256,
            _ => bail!("unknown upstream channel modulation"),
        })
    }
}

#[derive(Debug)]
struct UpstreamChannelExtended {
    symbol_rate: u64,
    modulation: UpstreamChannelModulation,
}

impl TryFrom<TableEntry> for UpstreamChannelExtended {
    type Error = anyhow::Error;

    fn try_from(entry: TableEntry) -> Result<Self, Self::Error> {
        Ok(UpstreamChannelExtended {
            symbol_rate: entry.parse_column(&AR_CM_DOC30_IF_UP_CHANNEL_EXTENDED_SYMBOL_RATE)?,
            modulation: entry.parse_column(&AR_CM_DOC30_IF_UP_CHANNEL_EXTENDED_MODULATION)?,
        })
    }
}

const DOCS_IF3_CM_STATUS_US_TABLE: OID = OID::new("1.3.6.1.4.1.4491.2.1.20.1.2"); // docsIf3CmStatusUsTable
const DOCS_IF3_CM_STATUS_US_TX_POWER: OID = OID::new("1.3.6.1.4.1.4491.2.1.20.1.2.1.1"); // docsIf3CmStatusUsTxPower
const DOCS_IF3_CM_STATUS_US_T3_TIMEOUTS: OID = OID::new("1.3.6.1.4.1.4491.2.1.20.1.2.1.2"); // docsIf3CmStatusUsT3Timeouts
const DOCS_IF3_CM_STATUS_US_T4_TIMEOUTS: OID = OID::new("1.3.6.1.4.1.4491.2.1.20.1.2.1.3"); // docsIf3CmStatusUsT4Timeouts

#[derive(Debug)]
struct CmStatusUs {
    tx_power: f64,
    t3_timeouts: u64,
    t4_timeouts: u64,
}

impl TryFrom<TableEntry> for CmStatusUs {
    type Error = anyhow::Error;

    fn try_from(entry: TableEntry) -> Result<Self, Self::Error> {
        Ok(CmStatusUs {
            tx_power: entry.parse_column::<f64>(&DOCS_IF3_CM_STATUS_US_TX_POWER)? / 10.0,
            t3_timeouts: entry.parse_column(&DOCS_IF3_CM_STATUS_US_T3_TIMEOUTS)?,
            t4_timeouts: entry.parse_column(&DOCS_IF3_CM_STATUS_US_T4_TIMEOUTS)?,
        })
    }
}

pub struct UpstreamMetrics {
    up_channel_id: UIntGaugeVec,
    up_channel_frequency: UIntGaugeVec,
    up_channel_type: UIntGaugeVec,
    up_channel_symbol_rate: UIntGaugeVec,
    up_channel_modulation: UIntGaugeVec,
    up_channel_tx_power: GaugeVec,
    up_channel_t3_timeouts: UIntGaugeVec,
    up_channel_t4_timeouts: UIntGaugeVec,
}

impl UpstreamMetrics {
    pub fn new() -> Result<Self> {
        Ok(UpstreamMetrics {
            up_channel_id: UIntGaugeVec::new(
                Opts::new("up_channel_id", "Upstream Channel ID"),
                &["index"],
            )?,
            up_channel_frequency: UIntGaugeVec::new(
                Opts::new("up_channel_frequency", "Upstream Channel Frequency (Hz)"),
                &["index"],
            )?,
            up_channel_type: UIntGaugeVec::new(
                Opts::new("up_channel_type", "Upstream Channel Type"),
                &["index"],
            )?,
            up_channel_symbol_rate: UIntGaugeVec::new(
                Opts::new(
                    "up_channel_symbol_rate",
                    "Upstream Channel Symbol Rate (ksps)",
                ),
                &["index"],
            )?,
            up_channel_modulation: UIntGaugeVec::new(
                Opts::new("up_channel_modulation", "Upstream Channel Modulation"),
                &["index"],
            )?,
            up_channel_tx_power: GaugeVec::new(
                Opts::new("up_channel_tx_power", "Upstream Channel Power (dBmV)"),
                &["index"],
            )?,
            up_channel_t3_timeouts: UIntGaugeVec::new(
                Opts::new("up_channel_t3_timeouts", "Upstream Channel T3 Timeouts"),
                &["index"],
            )?,
            up_channel_t4_timeouts: UIntGaugeVec::new(
                Opts::new("up_channel_t4_timeouts", "Upstream Channel T4 Timeouts"),
                &["index"],
            )?,
        })
    }

    pub fn set<'a>(&self, router_status: &List) -> Result<()> {
        let upstream_channel_table: Table<UpstreamChannel> =
            router_status.parse_table(&DOCS_IF_UPSTREAM_CHANNEL_TABLE)?;
        let upstream_channel_ext_table: Table<UpstreamChannelExtended> =
            router_status.parse_table(&ARRIS_CM_DOC30_IF_UPSTREAM_CHANNEL_EXTENDED_TABLE)?;
        let upstream_cm_status_table: Table<CmStatusUs> =
            router_status.parse_table(&DOCS_IF3_CM_STATUS_US_TABLE)?;

        for (index, upstream_channel_entry) in upstream_channel_table.iter() {
            let upstream_channel_ext_entry = upstream_channel_ext_table.get(index).unwrap();
            let upstream_cm_status_entry = upstream_cm_status_table.get(index).unwrap();

            self.up_channel_id
                .with_label_values(&[index])
                .set(upstream_channel_entry.up_channel_id);

            self.up_channel_frequency
                .with_label_values(&[index])
                .set(upstream_channel_entry.up_channel_frequency);

            self.up_channel_type
                .with_label_values(&[index])
                .set(upstream_channel_entry.up_channel_type as u64);

            self.up_channel_symbol_rate
                .with_label_values(&[index])
                .set(upstream_channel_ext_entry.symbol_rate);

            self.up_channel_modulation
                .with_label_values(&[index])
                .set(upstream_channel_ext_entry.modulation as u64);

            self.up_channel_tx_power
                .with_label_values(&[index])
                .set(upstream_cm_status_entry.tx_power);

            self.up_channel_t3_timeouts
                .with_label_values(&[index])
                .set(upstream_cm_status_entry.t3_timeouts);

            self.up_channel_t4_timeouts
                .with_label_values(&[index])
                .set(upstream_cm_status_entry.t4_timeouts);
        }

        Ok(())
    }
}

impl Collector for UpstreamMetrics {
    fn desc(&self) -> Vec<&Desc> {
        vec![
            self.up_channel_id.desc(),
            self.up_channel_frequency.desc(),
            self.up_channel_type.desc(),
            self.up_channel_symbol_rate.desc(),
            self.up_channel_modulation.desc(),
            self.up_channel_tx_power.desc(),
            self.up_channel_t3_timeouts.desc(),
            self.up_channel_t4_timeouts.desc(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn collect(&self) -> Vec<MetricFamily> {
        vec![
            self.up_channel_id.collect(),
            self.up_channel_frequency.collect(),
            self.up_channel_type.collect(),
            self.up_channel_symbol_rate.collect(),
            self.up_channel_modulation.collect(),
            self.up_channel_tx_power.collect(),
            self.up_channel_t3_timeouts.collect(),
            self.up_channel_t4_timeouts.collect(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}
