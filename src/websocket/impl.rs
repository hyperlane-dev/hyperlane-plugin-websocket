use crate::*;

impl<'a> BroadcastType<'a> {
    pub fn get_key(broadcast_type: BroadcastType) -> String {
        match broadcast_type {
            BroadcastType::PointToPoint(key1, key2) => {
                let (first_key, second_key) = if key1 <= key2 {
                    (key1, key2)
                } else {
                    (key2, key1)
                };
                format!("{}-{}-{}", POINT_TO_POINT_KEY, first_key, second_key)
            }
            BroadcastType::PointToGroup(key) => {
                format!("{}-{}", POINT_TO_GROUP_KEY, key)
            }
        }
    }
}

impl WebSocket {
    pub fn new() -> Self {
        Self {
            broadcast_map: BroadcastMap::default(),
        }
    }

    fn subscribe_unwrap_or_insert(
        &self,
        broadcast_type: BroadcastType,
    ) -> BroadcastMapReceiver<Vec<u8>> {
        let key: String = BroadcastType::get_key(broadcast_type);
        self.broadcast_map.subscribe_unwrap_or_insert(&key)
    }

    fn point_to_point(&self, key1: &str, key2: &str) -> BroadcastMapReceiver<Vec<u8>> {
        self.subscribe_unwrap_or_insert(BroadcastType::PointToPoint(key1, key2))
    }

    fn point_to_group(&self, key: &str) -> BroadcastMapReceiver<Vec<u8>> {
        self.subscribe_unwrap_or_insert(BroadcastType::PointToGroup(key))
    }

    pub async fn run<'a, F1, Fut1, F2, Fut2>(
        &self,
        ctx: &Context,
        buffer_size: usize,
        broadcast_type: BroadcastType<'a>,
        callback: F1,
        send_callback: F2,
    ) where
        F1: FuncWithoutPin<Fut1>,
        Fut1: Future<Output = ()> + Send + 'static,
        F2: FuncWithoutPin<Fut2>,
        Fut2: Future<Output = ()> + Send + 'static,
    {
        let mut receiver: Receiver<Vec<u8>> = match broadcast_type {
            BroadcastType::PointToPoint(key1, key2) => self.point_to_point(key1, key2),
            BroadcastType::PointToGroup(key) => self.point_to_group(key),
        };
        let key: String = BroadcastType::get_key(broadcast_type);
        loop {
            tokio::select! {
                request_res = ctx.websocket_request_from_stream(buffer_size) => {
                    if request_res.is_err() {
                        break;
                    }
                    callback(ctx.clone()).await;
                    let body: ResponseBody = ctx.get_response_body().await;
                    let send_res: BroadcastMapSendResult<_> = self.broadcast_map.send(&key, body);
                    send_callback(ctx.clone()).await;
                    if send_res.is_err() {
                        break;
                    }
                },
                msg_res = receiver.recv() => {
                    if let Ok(msg) = msg_res {
                        if ctx.send_response_body(msg).await.is_err() || ctx.flush().await.is_err() {
                            break;
                        }
                    }
               }
            }
        }
    }
}
