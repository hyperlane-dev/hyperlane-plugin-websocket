use crate::*;

pub(crate) static BROADCAST_MAP: OnceLock<WebSocket> = OnceLock::new();
