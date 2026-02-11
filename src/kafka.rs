//! Kafka integration for API Backend
//!
//! Re-exports from confuse-common events module for convenience.

pub use confuse_common::events::{
    EventProducer,
    SourceSyncRequestedEvent,
    config::KafkaConfig,
    topics::Topics,
};
