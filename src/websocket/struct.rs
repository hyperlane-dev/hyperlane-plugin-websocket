use crate::*;

pub struct WebSocket {
    pub(super) broadcast_map: BroadcastMap<Vec<u8>>,
}
