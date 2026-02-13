//! Channel adapters for different messaging platforms.
//!
//! This module contains implementations of the InboundAdapter and OutboundAdapter
//! traits for various messaging platforms.

pub mod postmark;
pub mod slack;

pub use postmark::{PostmarkInboundAdapter, PostmarkOutboundAdapter};
pub use slack::{
    is_url_verification, SlackChallengeResponse, SlackEventWrapper, SlackInboundAdapter,
    SlackMessageEvent, SlackOutboundAdapter, SlackUrlVerification,
};
