mod websocket;

use {hyperlane_plugin_websocket::*, websocket::*};

use std::sync::OnceLock;

use {
    hyperlane::*,
    hyperlane_broadcast::*,
    tokio::{spawn, time::sleep},
};
