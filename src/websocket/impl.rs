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

impl<B: BroadcastTypeTrait> Default for BroadcastType<B> {
    fn default() -> Self {
        BroadcastType::Unknown
    }
}

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
            BroadcastType::Unknown => String::new(),
        }
    }
}

impl<B: BroadcastTypeTrait> Default for WebSocketConfig<B> {
    fn default() -> Self {
        let default_hook: ArcFnPinBoxSendSync = Arc::new(move |_| Box::pin(async {}));
        Self {
            context: Context::default(),
            buffer_size: DEFAULT_BUFFER_SIZE,
            capacity: DEFAULT_BROADCAST_SENDER_CAPACITY,
            broadcast_type: BroadcastType::default(),
            request_hook: default_hook.clone(),
            sended_hook: default_hook.clone(),
            closed_hook: default_hook,
        }
    }
}

impl<B: BroadcastTypeTrait> WebSocketConfig<B> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }

    pub fn set_capacity(mut self, capacity: Capacity) -> Self {
        self.capacity = capacity;
        self
    }

    pub fn set_context(mut self, context: Context) -> Self {
        self.context = context;
        self
    }

    pub fn set_broadcast_type(mut self, broadcast_type: BroadcastType<B>) -> Self {
        self.broadcast_type = broadcast_type;
        self
    }

    pub fn set_request_hook<F, Fut>(mut self, hook: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.request_hook = Arc::new(move |ctx| Box::pin(hook(ctx)));
        self
    }

    pub fn set_sended_hook<F, Fut>(mut self, hook: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.sended_hook = Arc::new(move |ctx| Box::pin(hook(ctx)));
        self
    }

    pub fn set_closed_hook<F, Fut>(mut self, hook: F) -> Self
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.closed_hook = Arc::new(move |ctx| Box::pin(hook(ctx)));
        self
    }

    pub fn get_context(&self) -> &Context {
        &self.context
    }

    pub fn get_buffer_size(&self) -> usize {
        self.buffer_size
    }

    pub fn get_capacity(&self) -> Capacity {
        self.capacity
    }

    pub fn get_broadcast_type(&self) -> &BroadcastType<B> {
        &self.broadcast_type
    }

    pub fn get_request_hook(&self) -> &ArcFnPinBoxSendSync {
        &self.request_hook
    }

    pub fn get_sended_hook(&self) -> &ArcFnPinBoxSendSync {
        &self.sended_hook
    }

    pub fn get_closed_hook(&self) -> &ArcFnPinBoxSendSync {
        &self.closed_hook
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
        capacity: Capacity,
    ) -> BroadcastMapReceiver<Vec<u8>> {
        let key: String = BroadcastType::get_key(broadcast_type);
        self.broadcast_map.subscribe_or_insert(&key, capacity)
    }

    fn point_to_point<B: BroadcastTypeTrait>(
        &self,
        key1: &B,
        key2: &B,
        capacity: Capacity,
    ) -> BroadcastMapReceiver<Vec<u8>> {
        self.subscribe_unwrap_or_insert(
            BroadcastType::PointToPoint(key1.clone(), key2.clone()),
            capacity,
        )
    }

    fn point_to_group<B: BroadcastTypeTrait>(
        &self,
        key: &B,
        capacity: Capacity,
    ) -> BroadcastMapReceiver<Vec<u8>> {
        self.subscribe_unwrap_or_insert(BroadcastType::PointToGroup(key.clone()), capacity)
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

    pub async fn run<B: BroadcastTypeTrait>(&self, config: WebSocketConfig<B>) {
        let ctx: Context = config.get_context().clone();
        if ctx.to_string() == Context::default().to_string() {
            panic!("Context must be set");
        }
        let buffer_size: usize = config.get_buffer_size();
        let capacity: Capacity = config.get_capacity();
        let broadcast_type: BroadcastType<B> = config.get_broadcast_type().clone();
        let mut receiver: Receiver<Vec<u8>> = match &broadcast_type {
            BroadcastType::PointToPoint(key1, key2) => self.point_to_point(key1, key2, capacity),
            BroadcastType::PointToGroup(key) => self.point_to_group(key, capacity),
            BroadcastType::Unknown => panic!("BroadcastType must be PointToPoint or PointToGroup"),
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
                        config.get_request_hook()(ctx.clone()).await;
                    } else {
                        need_break = true;
                        config.get_closed_hook()(ctx.clone()).await;
                    }
                    let body: ResponseBody = ctx.get_response_body().await;
                    let is_err: bool = self.broadcast_map.send(&key, body).is_err();
                    config.get_sended_hook()(ctx.clone()).await;
                    if need_break || is_err {
                        break;
                    }
                },
                msg_res = receiver.recv() => {
                    if let Ok(msg) = msg_res {
                        if ctx.set_response_body(msg).await.send_body().await.is_ok() {
                            continue;
                        }
                    }
                    break;
                }
            }
        }
        result_handle().await;
    }
}
