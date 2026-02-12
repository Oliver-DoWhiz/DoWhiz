//! Channel adapters for different messaging platforms.
//!
//! This module contains implementations of the InboundAdapter and OutboundAdapter
//! traits for various messaging platforms.

pub mod postmark;

pub use postmark::{PostmarkInboundAdapter, PostmarkOutboundAdapter};
