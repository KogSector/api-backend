//! Kafka Module
//!
//! Event-driven messaging infrastructure for API Backend.

pub mod producer;
pub mod events;

pub use producer::EventProducer;
pub use events::*;
