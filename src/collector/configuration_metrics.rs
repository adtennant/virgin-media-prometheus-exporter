use super::UIntGauge;
use crate::snmp::{List, Table, TableEntry, OID};

use anyhow::{bail, Context, Result};
use prometheus::{
    core::{Collector, Desc},
    proto::MetricFamily,
};
use std::{convert::TryFrom, str::FromStr};

const DOCSIS_BASE_CAPABILITY: OID = OID::new("1.3.6.1.2.1.10.127.1.1.5"); // DocsisBaseCapability

const DOCS_QOS_SERVICE_FLOW_TABLE: OID = OID::new("1.3.6.1.4.1.4491.2.1.21.1.3"); // docsQosServiceFlowTable
const DOCS_QOS_SERVICE_FLOW_DIRECTION: OID = OID::new("1.3.6.1.4.1.4491.2.1.21.1.3.1.7"); // docsQosServiceFlowDirection
const DOCS_QOS_SERVICE_FLOW_PRIMARY: OID = OID::new("1.3.6.1.4.1.4491.2.1.21.1.3.1.8"); // docsQosServiceFlowPrimary

#[derive(Debug, Eq, PartialEq)]
enum QOSServiceFlowDirection {
    Downstream = 1,
    Upstream = 2,
}

impl FromStr for QOSServiceFlowDirection {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use QOSServiceFlowDirection::*;

        Ok(match s {
            "1" => Downstream,
            "2" => Upstream,
            _ => bail!("unknown docsQosServiceFlowDirection: {}", s),
        })
    }
}

#[derive(Debug)]
struct QOSServiceFlow {
    direction: QOSServiceFlowDirection,
    primary: bool,
}

impl TryFrom<TableEntry> for QOSServiceFlow {
    type Error = anyhow::Error;

    fn try_from(entry: TableEntry) -> Result<Self, Self::Error> {
        Ok(QOSServiceFlow {
            direction: entry.parse_column(&DOCS_QOS_SERVICE_FLOW_DIRECTION)?,
            primary: entry.get_column(&DOCS_QOS_SERVICE_FLOW_PRIMARY)? == "1",
        })
    }
}

const DOCS_QOS_PARAM_SET_TABLE: OID = OID::new("1.3.6.1.4.1.4491.2.1.21.1.2"); // docsQosParamSetTable
const DOCS_QOS_PARAM_SET_MAX_TRAFFIC_RATE: OID = OID::new("1.3.6.1.4.1.4491.2.1.21.1.2.1.6"); // docsQosParamSetMaxTrafficRate
const DOCS_QOS_PARAM_SET_MAX_TRAFFIC_BURST: OID = OID::new("1.3.6.1.4.1.4491.2.1.21.1.2.1.7"); // docsQosParamSetMaxTrafficBurst
const DOCS_QOS_PARAM_SET_MIN_RESERVED_RATE: OID = OID::new("1.3.6.1.4.1.4491.2.1.21.1.2.1.8"); // docsQosParamSetMinReservedRate
const DOCS_QOS_PARAM_SET_MAX_CONCAT_BURST: OID = OID::new("1.3.6.1.4.1.4491.2.1.21.1.2.1.12"); // docsQosParamSetMaxConcatBurst
const DOCS_QOS_PARAM_SET_SCHEDULING_TYPE: OID = OID::new("1.3.6.1.4.1.4491.2.1.21.1.2.1.13"); // docsQosParamSetSchedulingType

#[derive(Clone, Copy, Debug)]
enum QOSSchedulingType {
    Undefined = 1,
    BestEffort = 2,
    NonRealTimePollingService = 3,
    RealTimePollingService = 4,
    UnsolicitedGrantServiceWithAD = 5,
    UnsolicitedGrantService = 6,
}

impl FromStr for QOSSchedulingType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use QOSSchedulingType::*;

        Ok(match s {
            "1" => Undefined,
            "2" => BestEffort,
            "3" => NonRealTimePollingService,
            "4" => RealTimePollingService,
            "5" => UnsolicitedGrantServiceWithAD,
            "6" => UnsolicitedGrantService,
            _ => bail!("unknown docsQosParamSetSchedulingType: {}", s),
        })
    }
}

#[derive(Debug)]
struct QOSParamSet {
    max_traffic_rate: u64,
    max_traffic_burst: u64,
    min_reserved_rate: u64,
    max_concat_burst: u64,
    scheduling_type: QOSSchedulingType,
}

impl TryFrom<TableEntry> for QOSParamSet {
    type Error = anyhow::Error;

    fn try_from(entry: TableEntry) -> Result<Self, Self::Error> {
        Ok(QOSParamSet {
            max_traffic_rate: entry.parse_column(&DOCS_QOS_PARAM_SET_MAX_TRAFFIC_RATE)?,
            max_traffic_burst: entry.parse_column(&DOCS_QOS_PARAM_SET_MAX_TRAFFIC_BURST)?,
            min_reserved_rate: entry.parse_column(&DOCS_QOS_PARAM_SET_MIN_RESERVED_RATE)?,
            max_concat_burst: entry.parse_column(&DOCS_QOS_PARAM_SET_MAX_CONCAT_BURST)?,
            scheduling_type: entry.parse_column(&DOCS_QOS_PARAM_SET_SCHEDULING_TYPE)?,
        })
    }
}

pub struct ConfigurationMetrics {
    docsis_mode: UIntGauge,

    primary_downstream_sfid: UIntGauge,
    primary_downstream_max_traffic_rate: UIntGauge,
    primary_downstream_max_traffic_burst: UIntGauge,
    primary_downstream_min_reserved_rate: UIntGauge,

    primary_upstream_sfid: UIntGauge,
    primary_upstream_max_traffic_rate: UIntGauge,
    primary_upstream_max_traffic_burst: UIntGauge,
    primary_upstream_min_reserved_rate: UIntGauge,
    primary_upstream_max_concat_burst: UIntGauge,
    primary_upstream_scheduling_type: UIntGauge,
}

impl ConfigurationMetrics {
    pub fn new() -> Result<Self> {
        Ok(ConfigurationMetrics {
            docsis_mode: UIntGauge::new("docsis_mode", "DOCSIS Mode")?,

            primary_downstream_sfid: UIntGauge::new(
                "primary_downstream_sfid",
                "Primary Downstream Service Flow SFID",
            )?,
            primary_downstream_max_traffic_rate: UIntGauge::new(
                "primary_downstream_max_traffic_rate",
                "Primary Downstream Service Flow Max Traffic Rate",
            )?,
            primary_downstream_max_traffic_burst: UIntGauge::new(
                "primary_downstream_max_traffic_burst",
                "Primary Downstream Service Flow Max Traffic Burst",
            )?,
            primary_downstream_min_reserved_rate: UIntGauge::new(
                "primary_downstream_min_reserved_rate",
                "Primary Downstream Service Flow Min Traffic Rate",
            )?,

            primary_upstream_sfid: UIntGauge::new(
                "primary_upstream_sfid",
                "Primary Upstream Service Flow SFID",
            )?,
            primary_upstream_max_traffic_rate: UIntGauge::new(
                "primary_upstream_max_traffic_rate",
                "Primary Upstream Service Flow Max Traffic Rate",
            )?,
            primary_upstream_max_traffic_burst: UIntGauge::new(
                "primary_upstream_max_traffic_burst",
                "Primary Upstream Service Flow Max Traffic Burst",
            )?,
            primary_upstream_min_reserved_rate: UIntGauge::new(
                "primary_upstream_min_reserved_rate",
                "Primary Upstream Service Flow Min Traffic Rate",
            )?,
            primary_upstream_max_concat_burst: UIntGauge::new(
                "primary_upstream_max_concat_burst",
                "Primary Upstream Service Flow Max Concatenated Burst",
            )?,
            primary_upstream_scheduling_type: UIntGauge::new(
                "primary_upstream_scheduling_type",
                "Primary Upstream Service Flow Scheduling Type",
            )?,
        })
    }

    pub fn set(&self, router_status: &List) -> Result<()> {
        let docsis_mode = router_status.parse_scalar(&DOCSIS_BASE_CAPABILITY)?;
        self.docsis_mode.set(docsis_mode);

        let qos_service_flow_table: Table<QOSServiceFlow> =
            router_status.parse_table(&DOCS_QOS_SERVICE_FLOW_TABLE)?;

        let qos_param_set_table: Table<QOSParamSet> =
            router_status.parse_table(&DOCS_QOS_PARAM_SET_TABLE)?;

        let primary_downstream_flow_index = qos_service_flow_table
            .iter()
            .filter(|(_, entry)| {
                entry.direction == QOSServiceFlowDirection::Downstream && entry.primary
            })
            .map(|(key, _)| key)
            .next()
            .context("primary downstream flow key not found")?;

        let primary_downstream_param_set = qos_param_set_table
            .get(primary_downstream_flow_index)
            .context("primary downstream param set not found")?;

        self.primary_downstream_sfid.set(
            primary_downstream_flow_index
                .split('.')
                .next_back()
                .context("unable to extract primary downstream SFID")?
                .parse()?,
        );

        self.primary_downstream_max_traffic_rate
            .set(primary_downstream_param_set.max_traffic_rate);
        self.primary_downstream_max_traffic_burst
            .set(primary_downstream_param_set.max_traffic_burst);
        self.primary_downstream_min_reserved_rate
            .set(primary_downstream_param_set.min_reserved_rate);

        let primary_upstream_flow_index = qos_service_flow_table
            .iter()
            .filter(|(_, entry)| {
                entry.direction == QOSServiceFlowDirection::Upstream && entry.primary
            })
            .map(|(key, _)| key)
            .next()
            .context("primary upstream flow key not found")?;

        let primary_upstream_param_set = qos_param_set_table
            .get(primary_upstream_flow_index)
            .context("primary upstream param set not found")?;

        self.primary_upstream_sfid.set(
            primary_upstream_flow_index
                .split('.')
                .next_back()
                .context("unable to extract primary downstream SFID")?
                .parse()?,
        );

        self.primary_upstream_max_traffic_rate
            .set(primary_upstream_param_set.max_traffic_rate);
        self.primary_upstream_max_traffic_burst
            .set(primary_upstream_param_set.max_traffic_burst);
        self.primary_upstream_min_reserved_rate
            .set(primary_upstream_param_set.min_reserved_rate);
        self.primary_upstream_max_concat_burst
            .set(primary_upstream_param_set.max_concat_burst);
        self.primary_upstream_scheduling_type
            .set(primary_upstream_param_set.scheduling_type as u64);

        Ok(())
    }
}

impl Collector for ConfigurationMetrics {
    fn desc(&self) -> Vec<&Desc> {
        vec![
            self.docsis_mode.desc(),
            self.primary_downstream_sfid.desc(),
            self.primary_downstream_max_traffic_rate.desc(),
            self.primary_downstream_max_traffic_burst.desc(),
            self.primary_downstream_min_reserved_rate.desc(),
            self.primary_upstream_sfid.desc(),
            self.primary_upstream_max_traffic_rate.desc(),
            self.primary_upstream_max_traffic_burst.desc(),
            self.primary_upstream_min_reserved_rate.desc(),
            self.primary_upstream_max_concat_burst.desc(),
            self.primary_upstream_scheduling_type.desc(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn collect(&self) -> Vec<MetricFamily> {
        vec![
            self.docsis_mode.collect(),
            self.primary_downstream_sfid.collect(),
            self.primary_downstream_max_traffic_rate.collect(),
            self.primary_downstream_max_traffic_burst.collect(),
            self.primary_downstream_min_reserved_rate.collect(),
            self.primary_upstream_sfid.collect(),
            self.primary_upstream_max_traffic_rate.collect(),
            self.primary_upstream_max_traffic_burst.collect(),
            self.primary_upstream_min_reserved_rate.collect(),
            self.primary_upstream_max_concat_burst.collect(),
            self.primary_upstream_scheduling_type.collect(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}
