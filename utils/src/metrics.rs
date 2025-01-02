use lazy_static::lazy_static;
use prometheus::{
    opts, register_counter_vec, register_gauge, register_histogram_vec, register_int_gauge,
    CounterVec, Gauge, HistogramVec, IntGauge,
};

lazy_static! {
    pub static ref GRPC_RQE_COUNTER: CounterVec = register_counter_vec!(
        opts!("grpc_request_total", "Number of GRPC requests made.",),
        &["api"]
    )
    .unwrap();
    pub static ref GRPC_REQ_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "grpc_request_duration_seconds",
        "The GRPC request latencies in seconds.",
        &["api"]
    )
    .unwrap();
    pub static ref GRPC_REQ_GAUGE: Gauge = register_gauge!(opts!(
        "grpc_request_size_bytes",
        "The GRPC request sizes in bytes.",
    ))
    .unwrap();
    pub static ref CHAIN_PROGRESS: Gauge =
        register_gauge!(opts!("sync_progress", "The chain log sync progress.",)).unwrap();
    pub static ref EPOCH_QUORUMS: IntGauge =
        register_int_gauge!(opts!("quorums", "The quorums for latest epoch.",)).unwrap();
    pub static ref REGISTERED_EPOCH: Gauge =
        register_gauge!(opts!("epoch", "The latest registered epoch.",)).unwrap();
    pub static ref MINER_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "miner_duration_seconds",
        "The miner duration for each stage in seconds.",
        &["stage"]
    )
    .unwrap();
}
