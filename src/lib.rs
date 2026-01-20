//! A WebSocket plugin for the Hyperlane framework.
//!
//! A WebSocket plugin for the Hyperlane framework,
//! providing robust WebSocket communication capabilities and integrating
//! with hyperlane-broadcast for efficient message dissemination.

pub(crate) mod r#const;
pub(crate) mod r#enum;
pub(crate) mod r#impl;
pub(crate) mod r#struct;
#[cfg(test)]
pub(crate) mod test;
pub(crate) mod r#trait;

pub use {r#enum::*, r#struct::*};

pub(crate) use {r#const::*, r#trait::*};

pub(crate) use std::{
    convert::Infallible,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    num::{
        NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
    },
    sync::Arc,
};

pub(crate) use hyperlane::{tokio::sync::broadcast::Receiver, *};
pub(crate) use hyperlane_broadcast::*;

#[cfg(test)]
pub(crate) use std::sync::OnceLock;
