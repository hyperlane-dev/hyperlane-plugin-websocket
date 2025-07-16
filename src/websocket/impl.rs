use crate::*;

impl BroadcastTypeTrait for String {}
impl BroadcastTypeTrait for &str {}
impl BroadcastTypeTrait for char {}
impl BroadcastTypeTrait for bool {}
impl BroadcastTypeTrait for i8 {}
impl BroadcastTypeTrait for i16 {}
impl BroadcastTypeTrait for i32 {}
impl BroadcastTypeTrait for i64 {}
impl BroadcastTypeTrait for i128 {}
impl BroadcastTypeTrait for isize {}
impl BroadcastTypeTrait for u8 {}
impl BroadcastTypeTrait for u16 {}
impl BroadcastTypeTrait for u32 {}
impl BroadcastTypeTrait for u64 {}
impl BroadcastTypeTrait for u128 {}
impl BroadcastTypeTrait for usize {}
impl BroadcastTypeTrait for f32 {}
impl BroadcastTypeTrait for f64 {}
impl BroadcastTypeTrait for IpAddr {}
impl BroadcastTypeTrait for Ipv4Addr {}
impl BroadcastTypeTrait for Ipv6Addr {}
impl BroadcastTypeTrait for SocketAddr {}
impl BroadcastTypeTrait for NonZeroU8 {}
impl BroadcastTypeTrait for NonZeroU16 {}
impl BroadcastTypeTrait for NonZeroU32 {}
impl BroadcastTypeTrait for NonZeroU64 {}
impl BroadcastTypeTrait for NonZeroU128 {}
impl BroadcastTypeTrait for NonZeroUsize {}
impl BroadcastTypeTrait for NonZeroI8 {}
impl BroadcastTypeTrait for NonZeroI16 {}
impl BroadcastTypeTrait for NonZeroI32 {}
impl BroadcastTypeTrait for NonZeroI64 {}
impl BroadcastTypeTrait for NonZeroI128 {}
impl BroadcastTypeTrait for NonZeroIsize {}
impl BroadcastTypeTrait for Infallible {}

impl BroadcastTypeTrait for &String {}
impl BroadcastTypeTrait for &&str {}
impl BroadcastTypeTrait for &char {}
impl BroadcastTypeTrait for &bool {}
impl BroadcastTypeTrait for &i8 {}
impl BroadcastTypeTrait for &i16 {}
impl BroadcastTypeTrait for &i32 {}
impl BroadcastTypeTrait for &i64 {}
impl BroadcastTypeTrait for &i128 {}
impl BroadcastTypeTrait for &isize {}
impl BroadcastTypeTrait for &u8 {}
impl BroadcastTypeTrait for &u16 {}
impl BroadcastTypeTrait for &u32 {}
impl BroadcastTypeTrait for &u64 {}
impl BroadcastTypeTrait for &u128 {}
impl BroadcastTypeTrait for &usize {}
impl BroadcastTypeTrait for &f32 {}
impl BroadcastTypeTrait for &f64 {}
impl BroadcastTypeTrait for &IpAddr {}
impl BroadcastTypeTrait for &Ipv4Addr {}
impl BroadcastTypeTrait for &Ipv6Addr {}
impl BroadcastTypeTrait for &SocketAddr {}
impl BroadcastTypeTrait for &NonZeroU8 {}
impl BroadcastTypeTrait for &NonZeroU16 {}
impl BroadcastTypeTrait for &NonZeroU32 {}
impl BroadcastTypeTrait for &NonZeroU64 {}
impl BroadcastTypeTrait for &NonZeroU128 {}
impl BroadcastTypeTrait for &NonZeroUsize {}
impl BroadcastTypeTrait for &NonZeroI8 {}
impl BroadcastTypeTrait for &NonZeroI16 {}
impl BroadcastTypeTrait for &NonZeroI32 {}
impl BroadcastTypeTrait for &NonZeroI64 {}
impl BroadcastTypeTrait for &NonZeroI128 {}
impl BroadcastTypeTrait for &NonZeroIsize {}
impl BroadcastTypeTrait for &Infallible {}

impl<B: BroadcastTypeTrait> BroadcastType<B> {
    pub fn get_key(broadcast_type: BroadcastType<B>) -> String {
        match broadcast_type {
            BroadcastType::PointToPoint(key1, key2) => {
                let (first_key, second_key) = if key1 <= key2 {
                    (key1, key2)
                } else {
                    (key2, key1)
                };
                format!(
                    "{}-{}-{}",
                    POINT_TO_POINT_KEY,
                    first_key.to_string(),
                    second_key.to_string()
                )
            }
            BroadcastType::PointToGroup(key) => {
                format!("{}-{}", POINT_TO_GROUP_KEY, key.to_string())
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

    fn subscribe_unwrap_or_insert<B: BroadcastTypeTrait>(
        &self,
        broadcast_type: BroadcastType<B>,
    ) -> BroadcastMapReceiver<Vec<u8>> {
        let key: String = BroadcastType::get_key(broadcast_type);
        self.broadcast_map.subscribe_unwrap_or_insert(&key)
    }

    fn point_to_point<B: BroadcastTypeTrait>(
        &self,
        key1: &B,
        key2: &B,
    ) -> BroadcastMapReceiver<Vec<u8>> {
        self.subscribe_unwrap_or_insert(BroadcastType::PointToPoint(key1.clone(), key2.clone()))
    }

    fn point_to_group<B: BroadcastTypeTrait>(&self, key: &B) -> BroadcastMapReceiver<Vec<u8>> {
        self.subscribe_unwrap_or_insert(BroadcastType::PointToGroup(key.clone()))
    }

    pub fn receiver_count<'a, B: BroadcastTypeTrait>(
        &self,
        broadcast_type: BroadcastType<B>,
    ) -> ReceiverCount {
        let key: String = BroadcastType::get_key(broadcast_type);
        self.broadcast_map.receiver_count(&key).unwrap_or(0)
    }

    pub fn receiver_count_after_increment<B: BroadcastTypeTrait>(
        &self,
        broadcast_type: BroadcastType<B>,
    ) -> ReceiverCount {
        let count: ReceiverCount = self.receiver_count(broadcast_type);
        count.max(0).min(ReceiverCount::MAX - 1) + 1
    }

    pub fn receiver_count_after_decrement<B: BroadcastTypeTrait>(
        &self,
        broadcast_type: BroadcastType<B>,
    ) -> ReceiverCount {
        let count: ReceiverCount = self.receiver_count(broadcast_type);
        count.max(1).min(ReceiverCount::MAX) - 1
    }

    pub fn send<T, B>(
        &self,
        broadcast_type: BroadcastType<B>,
        data: T,
    ) -> BroadcastMapSendResult<Vec<u8>>
    where
        T: Into<Vec<u8>>,
        B: BroadcastTypeTrait,
    {
        let key: String = BroadcastType::get_key(broadcast_type);
        self.broadcast_map.send(&key, data.into())
    }

    pub async fn run<'a, F1, Fut1, F2, Fut2, F3, Fut3, B>(
        &self,
        ctx: &Context,
        buffer_size: usize,
        broadcast_type: BroadcastType<B>,
        request_handler: F1,
        on_sended: F2,
        on_client_closed: F3,
    ) where
        F1: FnSendSyncStatic<Fut1>,
        Fut1: FutureSendStatic,
        F2: FnSendSyncStatic<Fut2>,
        Fut2: FutureSendStatic,
        F3: FnSendSyncStatic<Fut3>,
        Fut3: FutureSendStatic,
        B: BroadcastTypeTrait,
    {
        let mut receiver: Receiver<Vec<u8>> = match &broadcast_type {
            BroadcastType::PointToPoint(key1, key2) => self.point_to_point(key1, key2),
            BroadcastType::PointToGroup(key) => self.point_to_group(key),
        };
        let key: String = BroadcastType::get_key(broadcast_type);
        let result_handle = || async {
            ctx.aborted().await;
            ctx.closed().await;
        };
        loop {
            tokio::select! {
                request_res = ctx.ws_from_stream(buffer_size) => {
                    let mut need_break = false;
                    if request_res.is_ok() {
                        request_handler(ctx.clone()).await;
                    } else {
                        need_break = true;
                        on_client_closed(ctx.clone()).await;
                    }
                    let body: ResponseBody = ctx.get_response_body().await;
                    let send_res: BroadcastMapSendResult<_> = self.broadcast_map.send(&key, body);
                    on_sended(ctx.clone()).await;
                    if need_break || send_res.is_err() {
                        break;
                    }
                },
                msg_res = receiver.recv() => {
                    if let Ok(msg) = msg_res {
                        if ctx.set_response_body(msg).await.send_body().await.is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
               }
            }
        }
        result_handle().await;
    }
}
