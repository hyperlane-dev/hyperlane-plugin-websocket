use crate::*;

#[derive(Setter, Getter)]
pub struct WebSocket {
    #[set(pub(super))]
    #[get(pub(super))]
    pub(super) broadcast_map: BroadcastMap<Vec<u8>>,
}
