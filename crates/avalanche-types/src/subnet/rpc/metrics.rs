//! Support for Prometheus metrics.
use crate::proto::pb::io::prometheus::client::{
    Bucket, Counter, Gauge, Histogram, LabelPair, Metric, MetricFamily, Quantile, Summary,
};

/// A list of LabelPair
#[derive(Clone, Debug)]
pub struct LabelPairs {
    pub lps: Vec<LabelPair>,
}

impl From<&[prometheus::proto::LabelPair]> for LabelPairs {
    fn from(item: &[prometheus::proto::LabelPair]) -> Self {
        let mut lps = Vec::with_capacity(item.len());
        for lp in item.iter() {
            lps.push(LabelPair::from(lp));
        }
        Self { lps }
    }
}

impl From<&prometheus::proto::LabelPair> for LabelPair {
    fn from(item: &prometheus::proto::LabelPair) -> Self {
        LabelPair {
            name: Some(item.get_name().to_owned()),
            value: Some(item.get_value().to_owned()),
        }
    }
}

impl From<&prometheus::proto::Gauge> for Gauge {
    fn from(item: &prometheus::proto::Gauge) -> Self {
        Gauge {
            value: Some(item.get_value()),
        }
    }
}

impl From<&prometheus::proto::Counter> for Counter {
    fn from(item: &prometheus::proto::Counter) -> Self {
        Counter {
            value: Some(item.get_value()),
            exemplar: None,
        }
    }
}

impl From<&prometheus::proto::Histogram> for Histogram {
    fn from(item: &prometheus::proto::Histogram) -> Self {
        Histogram {
            bucket: Buckets::from(item.get_bucket()).bs,
            sample_count: Some(item.get_sample_count()),
            sample_sum: Some(item.get_sample_sum()),
        }
    }
}

impl From<&prometheus::proto::Bucket> for Bucket {
    fn from(item: &prometheus::proto::Bucket) -> Self {
        Bucket {
            cumulative_count: Some(item.get_cumulative_count()),
            upper_bound: Some(item.get_upper_bound()),
            exemplar: None,
        }
    }
}

/// A list of Bucket
#[derive(Clone, Debug)]
pub struct Buckets {
    pub bs: Vec<Bucket>,
}

impl From<&[prometheus::proto::Bucket]> for Buckets {
    fn from(item: &[prometheus::proto::Bucket]) -> Self {
        let mut bs = Vec::with_capacity(item.len());
        for b in item.iter() {
            bs.push(Bucket::from(b));
        }
        Self { bs }
    }
}

impl From<&prometheus::proto::Summary> for Summary {
    fn from(item: &prometheus::proto::Summary) -> Self {
        Summary {
            sample_sum: Some(item.get_sample_sum()),
            sample_count: Some(item.get_sample_count()),
            quantile: Quantiles::from(item.get_quantile()).qs,
        }
    }
}

impl From<&prometheus::proto::Quantile> for Quantile {
    fn from(item: &prometheus::proto::Quantile) -> Self {
        Quantile {
            quantile: Some(item.get_quantile()),
            value: Some(item.get_value()),
        }
    }
}

/// A list of Quantile
#[derive(Clone, Debug)]
pub struct Quantiles {
    pub qs: Vec<Quantile>,
}

impl From<&[prometheus::proto::Quantile]> for Quantiles {
    fn from(item: &[prometheus::proto::Quantile]) -> Self {
        let mut qs = Vec::with_capacity(item.len());
        for q in item.iter() {
            qs.push(Quantile::from(q));
        }
        Self { qs }
    }
}

impl From<&prometheus::proto::Metric> for Metric {
    fn from(item: &prometheus::proto::Metric) -> Self {
        Metric {
            label: LabelPairs::from(item.get_label()).lps,
            counter: Some(Counter::from(item.get_counter())),
            gauge: Some(Gauge::from(item.get_gauge())),
            histogram: Some(Histogram::from(item.get_histogram())),
            summary: Some(Summary::from(item.get_summary())),
            timestamp_ms: Some(item.get_timestamp_ms()),
            untyped: None, // deprecated
        }
    }
}

/// A list of MetricFamily
#[derive(Clone, Debug)]
pub struct MetricsFamilies {
    pub mfs: Vec<MetricFamily>,
}

impl From<&Vec<prometheus::proto::MetricFamily>> for MetricsFamilies {
    fn from(item: &Vec<prometheus::proto::MetricFamily>) -> Self {
        let mut mfs = Vec::with_capacity(item.len());
        for mf in item.iter() {
            let mut metric = Vec::new();
            for m in mf.get_metric().iter() {
                metric.push(Metric::from(m));
            }
            mfs.push(MetricFamily {
                name: Some(mf.get_name().to_owned()),
                help: Some(mf.get_help().to_owned()),
                r#type: Some(mf.get_field_type() as i32),
                metric,
            });
        }
        Self { mfs }
    }
}

#[test]
#[ignore]
fn test_gather_process() {
    let families = vec![
        "process_cpu_seconds_total",
        "process_max_fds",
        "process_open_fds",
        "process_resident_memory_bytes",
        "process_start_time_seconds",
        "process_cpu_seconds_total",
        "process_virtual_memory_bytes",
    ];

    let metric_families = MetricsFamilies::from(&prometheus::gather()).mfs;
    if !metric_families.is_empty() {
        for (i, family) in families.iter().enumerate() {
            if let Some(name) = &metric_families[i].name {
                assert_eq!(name, family);
            } else {
                panic!("expected some {} found none", family)
            }
        }
    }
}
