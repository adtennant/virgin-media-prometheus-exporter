use super::UIntGaugeVec;
use crate::snmp::{List, Table, TableEntry, OID};

use anyhow::{bail, Result};
use prometheus::{
    core::{Collector, Desc},
    proto::MetricFamily,
    GaugeVec, Opts,
};
use std::{convert::TryFrom, str::FromStr};

const DOCS_IF_DOWNSTREAM_CHANNEL_TABLE: OID = OID::new("1.3.6.1.2.1.10.127.1.1.1"); // docsIfDownstreamChannelTable
const DOCS_IF_DOWN_CHANNEL_ID: OID = OID::new("1.3.6.1.2.1.10.127.1.1.1.1.1"); // docsIfDownChannelId
const DOCS_IF_DOWN_CHANNEL_FREQUENCY: OID = OID::new("1.3.6.1.2.1.10.127.1.1.1.1.2"); // docsIfDownChannelFrequency
const DOCS_IF_DOWN_CHANNEL_MODULATION: OID = OID::new("1.3.6.1.2.1.10.127.1.1.1.1.4"); // docsIfDownChannelModulation
const DOCS_IF_DOWN_CHANNEL_POWER: OID = OID::new("1.3.6.1.2.1.10.127.1.1.1.1.6"); // docsIfDownChannelPower

#[derive(Copy, Clone, Debug)]
pub enum DownstreamModulation {
    Unknown = 1,
    Other = 2,
    QAM64 = 3,
    QAM256 = 4,
}

impl FromStr for DownstreamModulation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use DownstreamModulation::*;

        Ok(match s {
            "1" => Unknown,
            "2" => Other,
            "3" => QAM64,
            "4" => QAM256,
            _ => bail!("unknown downstream modulation"),
        })
    }
}

#[derive(Debug)]
struct DownstreamChannel {
    down_channel_id: u64,
    down_channel_frequency: u64,
    down_channel_modulation: DownstreamModulation,
    down_channel_power: f64,
}

impl TryFrom<TableEntry> for DownstreamChannel {
    type Error = anyhow::Error;

    fn try_from(entry: TableEntry) -> Result<Self, Self::Error> {
        Ok(DownstreamChannel {
            down_channel_id: entry.parse_column(&DOCS_IF_DOWN_CHANNEL_ID)?,
            down_channel_frequency: entry.parse_column(&DOCS_IF_DOWN_CHANNEL_FREQUENCY)?,
            down_channel_modulation: entry.parse_column(&DOCS_IF_DOWN_CHANNEL_MODULATION)?,
            down_channel_power: entry.parse_column::<f64>(&DOCS_IF_DOWN_CHANNEL_POWER)? / 10.0,
        })
    }
}

const DOCS_IF3_SIGNAL_QUALITY_EXT_TABLE: OID = OID::new("1.3.6.1.4.1.4491.2.1.20.1.24"); // docsIf3SignalQualityExtTable
const DOCS_IF3_SIGNAL_QUALITY_EXT_RX_MER: OID = OID::new("1.3.6.1.4.1.4491.2.1.20.1.24.1.1"); // docsIf3SignalQualityExtRxMER

#[derive(Debug)]
struct SignalQualityExt {
    rx_mer: f64,
}

impl TryFrom<TableEntry> for SignalQualityExt {
    type Error = anyhow::Error;

    fn try_from(entry: TableEntry) -> Result<Self, Self::Error> {
        Ok(SignalQualityExt {
            rx_mer: entry.parse_column::<f64>(&DOCS_IF3_SIGNAL_QUALITY_EXT_RX_MER)? / 10.0,
        })
    }
}

const DOCS_IF_SIGNAL_QUALITY_TABLE: OID = OID::new("1.3.6.1.2.1.10.127.1.1.4"); // docsIfSignalQualityTable
const DOCS_IF_SIG_QCORRECTEDS: OID = OID::new("1.3.6.1.2.1.10.127.1.1.4.1.3"); // docsIfSigQCorrecteds
const DOCS_IF_SIG_QUNCORRECTABLES: OID = OID::new("1.3.6.1.2.1.10.127.1.1.4.1.4"); // docsIfSigQUncorrectables
const DOCS_IF_SIG_QSIGNAL_NOISE: OID = OID::new("1.3.6.1.2.1.10.127.1.1.4.1.5"); // docsIfSigQSignalNoise

#[derive(Debug)]
struct SignalQuality {
    correcteds: u64,
    uncorrectables: u64,
    signal_noise: u64,
}

impl TryFrom<TableEntry> for SignalQuality {
    type Error = anyhow::Error;

    fn try_from(entry: TableEntry) -> Result<Self, Self::Error> {
        Ok(SignalQuality {
            correcteds: entry.parse_column(&DOCS_IF_SIG_QCORRECTEDS)?,
            uncorrectables: entry.parse_column(&DOCS_IF_SIG_QUNCORRECTABLES)?,
            signal_noise: entry.parse_column::<u64>(&DOCS_IF_SIG_QSIGNAL_NOISE)? / 10,
        })
    }
}

pub struct DownstreamMetrics {
    down_channel_id: UIntGaugeVec,
    down_channel_frequency: UIntGaugeVec,
    down_channel_modulation: UIntGaugeVec,
    down_channel_power: GaugeVec,
    down_channel_rx_mer: GaugeVec,
    down_channel_correcteds: UIntGaugeVec,
    down_channel_uncorrectables: UIntGaugeVec,
    down_channel_signal_noise: UIntGaugeVec,
}

impl DownstreamMetrics {
    pub fn new() -> Result<Self> {
        Ok(DownstreamMetrics {
            down_channel_id: UIntGaugeVec::new(
                Opts::new("down_channel_id", "Downstream Channel ID"),
                &["index"],
            )?,
            down_channel_frequency: UIntGaugeVec::new(
                Opts::new(
                    "down_channel_frequency",
                    "Downstream Channel Frequency (Hz)",
                ),
                &["index"],
            )?,
            down_channel_modulation: UIntGaugeVec::new(
                Opts::new("down_channel_modulation", "Downstream Channel Modulation"),
                &["index"],
            )?,
            down_channel_power: GaugeVec::new(
                Opts::new("down_channel_power", "Downstream Channel Power (dBmV)"),
                &["index"],
            )?,
            down_channel_rx_mer: GaugeVec::new(
                Opts::new("down_channel_rx_mer", "Downstream Channel RxMER (dB)"),
                &["index"],
            )?,
            down_channel_correcteds: UIntGaugeVec::new(
                Opts::new(
                    "down_channel_correcteds",
                    "Downstream Channel Pre RS Errors",
                ),
                &["index"],
            )?,
            down_channel_uncorrectables: UIntGaugeVec::new(
                Opts::new(
                    "down_channel_uncorrectables",
                    "Downstream Channel Post RS Errors",
                ),
                &["index"],
            )?,
            down_channel_signal_noise: UIntGaugeVec::new(
                Opts::new("down_channel_signal_noise", "Downstream Channel SNR (dB)"),
                &["index"],
            )?,
        })
    }

    pub fn set(&self, router_status: &List) -> Result<()> {
        let downstream_channel_table: Table<DownstreamChannel> =
            router_status.parse_table(&DOCS_IF_DOWNSTREAM_CHANNEL_TABLE)?;
        let signal_quality_ext_table: Table<SignalQualityExt> =
            router_status.parse_table(&DOCS_IF3_SIGNAL_QUALITY_EXT_TABLE)?;
        let signal_quality_table: Table<SignalQuality> =
            router_status.parse_table(&DOCS_IF_SIGNAL_QUALITY_TABLE)?;

        for (key, downstream_channel_entry) in downstream_channel_table.iter() {
            let signal_quality_ext_entry = signal_quality_ext_table.get(key).unwrap();
            let signal_quality_entry = signal_quality_table.get(key).unwrap();

            self.down_channel_id
                .with_label_values(&[key])
                .set(downstream_channel_entry.down_channel_id);

            self.down_channel_frequency
                .with_label_values(&[key])
                .set(downstream_channel_entry.down_channel_frequency);

            self.down_channel_modulation
                .with_label_values(&[key])
                .set(downstream_channel_entry.down_channel_modulation as u64);

            self.down_channel_power
                .with_label_values(&[key])
                .set(downstream_channel_entry.down_channel_power);

            self.down_channel_rx_mer
                .with_label_values(&[key])
                .set(signal_quality_ext_entry.rx_mer);

            self.down_channel_correcteds
                .with_label_values(&[key])
                .set(signal_quality_entry.correcteds);

            self.down_channel_uncorrectables
                .with_label_values(&[key])
                .set(signal_quality_entry.uncorrectables);

            self.down_channel_signal_noise
                .with_label_values(&[key])
                .set(signal_quality_entry.signal_noise);
        }

        Ok(())
    }
}

impl Collector for DownstreamMetrics {
    fn desc(&self) -> Vec<&Desc> {
        vec![
            self.down_channel_id.desc(),
            self.down_channel_frequency.desc(),
            self.down_channel_modulation.desc(),
            self.down_channel_power.desc(),
            self.down_channel_rx_mer.desc(),
            self.down_channel_correcteds.desc(),
            self.down_channel_uncorrectables.desc(),
            self.down_channel_signal_noise.desc(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn collect(&self) -> Vec<MetricFamily> {
        vec![
            self.down_channel_id.collect(),
            self.down_channel_frequency.collect(),
            self.down_channel_modulation.collect(),
            self.down_channel_power.collect(),
            self.down_channel_rx_mer.collect(),
            self.down_channel_correcteds.collect(),
            self.down_channel_uncorrectables.collect(),
            self.down_channel_signal_noise.collect(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}
