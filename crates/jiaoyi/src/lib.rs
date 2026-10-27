//! The jiaoyi service, phase 1: load each permitted account's own daily bars
//! and costs from cTrader into PostgreSQL, record the live quote stream,
//! and report on them.

pub mod admission;
pub mod config;
pub mod edge;
pub mod fills;
pub mod history;
pub mod http;
pub mod limits;
pub mod loader;
pub mod operations;
pub mod recorder;
pub mod report;
pub mod research;
pub mod service;
pub mod shutdown;
pub mod status_page;
pub mod stop_loss_review;
pub mod strategy3;
pub mod supervisor;
pub mod telegram_report;
pub mod trading;
pub mod trading_report_data;
pub mod trading_report_snapshot;
pub mod trading_report_view;
