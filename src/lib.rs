//! ConFuse API Backend
//!
//! Central API Gateway for the ConFuse Knowledge Intelligence Platform

pub mod config;
pub mod error;
pub mod middleware;
pub mod routes;
pub mod clients;
pub mod models;

pub use config::Config;
pub use error::{AppError, Result};
