//! The jiaoyi service, phase 1: load each permitted account's own daily bars
//! and costs from cTrader into PostgreSQL, record the live quote stream,
//! and report on them.

pub mod config;
pub mod http;
pub mod loader;
pub mod recorder;
pub mod report;
pub mod service;
pub mod shutdown;
