use crate::*;

static BROADCAST_MAP: OnceLock<WebSocket> = OnceLock::new();

fn get_broadcast_map() -> &'static WebSocket {
    BROADCAST_MAP.get_or_init(WebSocket::new)
}

struct ServerPanic {
    response_body: String,
    content_type: String,
}

impl ServerHook for ServerPanic {
    async fn new(ctx: &Context) -> Self {
        let error: Panic = ctx.try_get_panic().await.unwrap_or_default();
        let response_body: String = error.to_string();
        let content_type: String = ContentType::format_content_type_with_charset(TEXT_PLAIN, UTF8);
        Self {
            response_body,
            content_type,
        }
    }

    async fn handle(self, ctx: &Context) {
        ctx.set_response_version(HttpVersion::Http1_1)
            .await
            .set_response_status_code(500)
            .await
            .clear_response_headers()
            .await
            .set_response_header(SERVER, HYPERLANE)
            .await
            .set_response_header(CONTENT_TYPE, &self.content_type)
            .await
            .set_response_body(&self.response_body)
            .await
            .send()
            .await;
    }
}

struct RequestError;

impl ServerHook for RequestError {
    async fn new(_ctx: &Context) -> Self {
        Self
    }

    async fn handle(self, ctx: &Context) {
        ctx.set_response_version(HttpVersion::Http1_1)
            .await
            .send()
            .await;
    }
}

struct RequestMiddleware {
    socket_addr: String,
}

impl ServerHook for RequestMiddleware {
    async fn new(ctx: &Context) -> Self {
        let socket_addr: String = ctx.get_socket_addr_string().await;
        Self { socket_addr }
    }

    async fn handle(self, ctx: &Context) {
        ctx.set_response_version(HttpVersion::Http1_1)
            .await
            .set_response_status_code(200)
            .await
            .set_response_header(SERVER, HYPERLANE)
            .await
            .set_response_header(CONNECTION, KEEP_ALIVE)
            .await
            .set_response_header(CONTENT_TYPE, TEXT_PLAIN)
            .await
            .set_response_header(ACCESS_CONTROL_ALLOW_ORIGIN, WILDCARD_ANY)
            .await
            .set_response_header("SocketAddr", &self.socket_addr)
            .await;
    }
}

struct UpgradeHook;

impl ServerHook for UpgradeHook {
    async fn new(_ctx: &Context) -> Self {
        Self
    }

    async fn handle(self, ctx: &Context) {
        if !ctx.get_request().await.is_ws() {
            return;
        }
        if let Some(key) = &ctx.try_get_request_header_back(SEC_WEBSOCKET_KEY).await {
            let accept_key: String = WebSocketFrame::generate_accept_key(key);
            ctx.set_response_version(HttpVersion::Http1_1)
                .await
                .set_response_status_code(101)
                .await
                .set_response_header(UPGRADE, WEBSOCKET)
                .await
                .set_response_header(CONNECTION, UPGRADE)
                .await
                .set_response_header(SEC_WEBSOCKET_ACCEPT, &accept_key)
                .await
                .set_response_body(&vec![])
                .await
                .send()
                .await;
        }
    }
}

struct ConnectedHook {
    receiver_count: ReceiverCount,
    data: String,
    group_broadcast_type: BroadcastType<String>,
    private_broadcast_type: BroadcastType<String>,
}

impl ServerHook for ConnectedHook {
    async fn new(ctx: &Context) -> Self {
        let group_name: String = ctx
            .try_get_route_param("group_name")
            .await
            .unwrap_or_default();
        let group_broadcast_type: BroadcastType<String> = BroadcastType::PointToGroup(group_name);
        let receiver_count: ReceiverCount =
            get_broadcast_map().receiver_count(group_broadcast_type.clone());
        let my_name: String = ctx.try_get_route_param("my_name").await.unwrap_or_default();
        let your_name: String = ctx
            .try_get_route_param("your_name")
            .await
            .unwrap_or_default();
        let private_broadcast_type: BroadcastType<String> =
            BroadcastType::PointToPoint(my_name, your_name);
        let data: String = format!("receiver_count => {receiver_count:?}");
        Self {
            receiver_count,
            data,
            group_broadcast_type,
            private_broadcast_type,
        }
    }

    async fn handle(self, _ctx: &Context) {
        get_broadcast_map()
            .send(self.group_broadcast_type, self.data.clone())
            .unwrap_or_else(|err| {
                println!("[connected_hook]send group error => {:?}", err.to_string());
                None
            });
        get_broadcast_map()
            .send(self.private_broadcast_type, self.data)
            .unwrap_or_else(|err| {
                println!(
                    "[connected_hook]send private error => {:?}",
                    err.to_string()
                );
                None
            });
        println!(
            "[connected_hook]receiver_count => {:?}",
            self.receiver_count
        );
        Server::flush_stdout();
    }
}

struct SendedHook {
    msg: String,
}

impl ServerHook for SendedHook {
    async fn new(ctx: &Context) -> Self {
        let msg: String = ctx.get_response_body_string().await;
        Self { msg }
    }

    async fn handle(self, _ctx: &Context) {
        println!("[sended_hook]msg => {}", self.msg);
        Server::flush_stdout();
    }
}

struct GroupChatRequestHook {
    body: RequestBody,
    receiver_count: ReceiverCount,
}

impl ServerHook for GroupChatRequestHook {
    async fn new(ctx: &Context) -> Self {
        let group_name: String = ctx.try_get_route_param("group_name").await.unwrap();
        let key: BroadcastType<String> = BroadcastType::PointToGroup(group_name);
        let mut receiver_count: ReceiverCount = get_broadcast_map().receiver_count(key.clone());
        let mut body: RequestBody = ctx.get_request_body().await;
        if body.is_empty() {
            receiver_count = get_broadcast_map().receiver_count_after_closed(key);
            body = format!("receiver_count => {receiver_count:?}").into();
        }
        Self {
            body,
            receiver_count,
        }
    }

    async fn handle(self, ctx: &Context) {
        ctx.set_response_body(&self.body).await;
        println!("[group_chat]receiver_count => {:?}", self.receiver_count);
        Server::flush_stdout();
    }
}

struct GroupClosedHook {
    body: String,
    receiver_count: ReceiverCount,
}

impl ServerHook for GroupClosedHook {
    async fn new(ctx: &Context) -> Self {
        let group_name: String = ctx.try_get_route_param("group_name").await.unwrap();
        let key: BroadcastType<String> = BroadcastType::PointToGroup(group_name);
        let receiver_count: ReceiverCount =
            get_broadcast_map().receiver_count_after_closed(key.clone());
        let body: String = format!("receiver_count => {receiver_count:?}");
        Self {
            body,
            receiver_count,
        }
    }

    async fn handle(self, ctx: &Context) {
        ctx.set_response_body(&self.body).await;
        println!("[group_closed]receiver_count => {:?}", self.receiver_count);
        Server::flush_stdout();
    }
}

struct GroupChat;

impl ServerHook for GroupChat {
    async fn new(_ctx: &Context) -> Self {
        Self
    }

    async fn handle(self, ctx: &Context) {
        let group_name: String = ctx.try_get_route_param("group_name").await.unwrap();
        let key: BroadcastType<String> = BroadcastType::PointToGroup(group_name);
        let config: WebSocketConfig<String> = WebSocketConfig::new()
            .set_context(ctx.clone())
            .set_broadcast_type(key)
            .set_request_config(RequestConfig::default())
            .set_capacity(1024)
            .set_connected_hook::<ConnectedHook>()
            .set_request_hook::<GroupChatRequestHook>()
            .set_sended_hook::<SendedHook>()
            .set_closed_hook::<GroupClosedHook>();
        get_broadcast_map().run(config).await;
    }
}

struct PrivateChatRequestHook {
    body: RequestBody,
    receiver_count: ReceiverCount,
}

impl ServerHook for PrivateChatRequestHook {
    async fn new(ctx: &Context) -> Self {
        let my_name: String = ctx.try_get_route_param("my_name").await.unwrap();
        let your_name: String = ctx.try_get_route_param("your_name").await.unwrap();
        let key: BroadcastType<String> = BroadcastType::PointToPoint(my_name, your_name);
        let mut receiver_count: ReceiverCount = get_broadcast_map().receiver_count(key.clone());
        let mut body: RequestBody = ctx.get_request_body().await;
        if body.is_empty() {
            receiver_count = get_broadcast_map().receiver_count_after_closed(key);
            body = format!("receiver_count => {receiver_count:?}").into();
        }
        Self {
            body,
            receiver_count,
        }
    }

    async fn handle(self, ctx: &Context) {
        ctx.set_response_body(&self.body).await;
        println!("[private_chat]receiver_count => {:?}", self.receiver_count);
        Server::flush_stdout();
    }
}

struct PrivateClosedHook {
    body: String,
    receiver_count: ReceiverCount,
}

impl ServerHook for PrivateClosedHook {
    async fn new(ctx: &Context) -> Self {
        let my_name: String = ctx.try_get_route_param("my_name").await.unwrap();
        let your_name: String = ctx.try_get_route_param("your_name").await.unwrap();
        let key: BroadcastType<String> = BroadcastType::PointToPoint(my_name, your_name);
        let receiver_count: ReceiverCount = get_broadcast_map().receiver_count_after_closed(key);
        let body: String = format!("receiver_count => {receiver_count:?}");
        Self {
            body,
            receiver_count,
        }
    }

    async fn handle(self, ctx: &Context) {
        ctx.set_response_body(&self.body).await;
        println!(
            "[private_closed]receiver_count => {:?}",
            self.receiver_count
        );
        Server::flush_stdout();
    }
}

struct PrivateChat {
    config: WebSocketConfig<String>,
}

impl ServerHook for PrivateChat {
    async fn new(ctx: &Context) -> Self {
        let my_name: String = ctx.try_get_route_param("my_name").await.unwrap();
        let your_name: String = ctx.try_get_route_param("your_name").await.unwrap();
        let key: BroadcastType<String> = BroadcastType::PointToPoint(my_name, your_name);
        let config: WebSocketConfig<String> = WebSocketConfig::new()
            .set_context(ctx.clone())
            .set_broadcast_type(key)
            .set_request_config(RequestConfig::default())
            .set_capacity(1024)
            .set_connected_hook::<ConnectedHook>()
            .set_request_hook::<PrivateChatRequestHook>()
            .set_sended_hook::<SendedHook>()
            .set_closed_hook::<PrivateClosedHook>();
        Self { config }
    }

    async fn handle(self, _ctx: &Context) {
        get_broadcast_map().run(self.config).await;
    }
}

#[tokio::test]
async fn main() {
    let server: Server = Server::new().await;
    server.panic::<ServerPanic>().await;
    server.request_error::<RequestError>().await;
    server.request_middleware::<RequestMiddleware>().await;
    server.request_middleware::<UpgradeHook>().await;
    server.route::<GroupChat>("/{group_name}").await;
    server.route::<PrivateChat>("/{my_name}/{your_name}").await;
    let server_control_hook_1: ServerControlHook = server.run().await.unwrap_or_default();
    let server_control_hook_2: ServerControlHook = server_control_hook_1.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        server_control_hook_2.shutdown().await;
    });
    server_control_hook_1.wait().await;
}
