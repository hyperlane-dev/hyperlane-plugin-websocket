use crate::*;

pub struct WebSocket {
    pub(super) broadcast_map: BroadcastMap<Vec<u8>>,
}

pub struct WebSocketConfig<B: BroadcastTypeTrait> {
    pub(super) context: Context,
    pub(super) buffer_size: usize,
    pub(super) capacity: Capacity,
    pub(super) broadcast_type: BroadcastType<B>,
    pub(super) request_hook: ArcFnPinBoxSendSync,
    pub(super) sended_hook: ArcFnPinBoxSendSync,
    pub(super) closed_hook: ArcFnPinBoxSendSync,
}
