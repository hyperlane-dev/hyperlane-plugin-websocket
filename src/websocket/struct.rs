use crate::*;

/// Represents a WebSocket instance.
///
/// This struct manages broadcast capabilities and holds the internal broadcast map
/// responsible for handling message distribution to various WebSocket connections.
#[derive(Debug, Clone)]
pub struct WebSocket {
    /// The internal broadcast map.
    ///
    /// This map is used for managing WebSocket message distribution.
    pub(super) broadcast_map: BroadcastMap<Vec<u8>>,
}

/// Configuration for a WebSocket connection.
///
/// This struct encapsulates all necessary parameters for setting up and managing
/// a WebSocket connection, including context, buffer sizes, capacity, broadcast type,
/// and various hook functions for different lifecycle events.
///
/// # Type Parameters
///
/// - `B`: The type used for broadcast keys, which must implement `BroadcastTypeTrait`.
#[derive(Clone)]
pub struct WebSocketConfig<B: BroadcastTypeTrait> {
    /// The Hyperlane context.
    ///
    /// This context is associated with this WebSocket connection.
    pub(super) context: Context,
    /// The buffer size.
    ///
    /// This is the size of the buffer used for reading from the WebSocket stream.
    pub(super) buffer_size: usize,
    /// The capacity.
    ///
    /// This is the capacity of the broadcast sender channel.
    pub(super) capacity: Capacity,
    /// The broadcast type.
    ///
    /// This defines the type of broadcast this WebSocket connection will participate in
    /// (point-to-point or point-to-group).
    pub(super) broadcast_type: BroadcastType<B>,
    /// The request hook function.
    ///
    /// This hook is executed when a new request is received on the WebSocket.
    pub(super) request_hook: ArcContextFnPinBoxSendSync,
    /// The sended hook function.
    ///
    /// This hook is executed after a message has been successfully sent over the WebSocket.
    pub(super) sended_hook: ArcContextFnPinBoxSendSync,
    /// The closed hook function.
    ///
    /// This hook is executed when the WebSocket connection is closed.
    pub(super) closed_hook: ArcContextFnPinBoxSendSync,
}
