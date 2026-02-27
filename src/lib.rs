//! A WebSocket plugin for the Hyperlane framework.
//!
//! A WebSocket plugin for the Hyperlane framework,
//! providing robust WebSocket communication capabilities and integrating
//! with hyperlane-broadcast for efficient message dissemination.

mod r#const;
mod r#enum;
mod r#impl;
mod r#struct;
#[cfg(test)]
mod test;
mod r#trait;

pub use {r#enum::*, r#struct::*};

use {r#const::*, r#trait::*};

use std::{
    convert::Infallible,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    num::{
        NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
    },
};
#[cfg(test)]
use std::{sync::OnceLock, time::Duration};

#[cfg(test)]
use tokio::{spawn, time::sleep};
use {
    hyperlane::{
        tokio::sync::broadcast::{Receiver, error::SendError},
        *,
    },
    hyperlane_broadcast::*,
};
