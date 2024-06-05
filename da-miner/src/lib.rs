#[macro_use]
extern crate tracing;

pub mod constants;
mod line_candidate;
mod line_metadata;
mod mine;
mod service;
mod stage1;
mod stage2;
mod submitter;
mod watcher;

pub use service::DasMineService;
