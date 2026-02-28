use crate::*;

static BROADCAST_MAP: OnceLock<WebSocket> = OnceLock::new();

fn get_broadcast_map() -> &'static WebSocket {
    BROADCAST_MAP.get_or_init(WebSocket::new)
}

struct TaskPanicHook {
    response_body: String,
    content_type: String,
}

impl ServerHook for TaskPanicHook {
    async fn new(ctx: &mut Context) -> Self {
        let error: PanicData = ctx.try_get_task_panic_data().unwrap_or_default();
        let response_body: String = error.to_string();
        let content_type: String = ContentType::format_content_type_with_charset(TEXT_PLAIN, UTF8);
        Self {
            response_body,
            content_type,
        }
    }

    async fn handle(self, ctx: &mut Context) {
        ctx.get_mut_response()
            .set_version(HttpVersion::Http1_1)
            .set_status_code(500)
            .clear_headers()
            .set_header(SERVER, HYPERLANE)
            .set_header(CONTENT_TYPE, &self.content_type)
            .set_body(&self.response_body);
        if ctx.try_send().await.is_err() {
            ctx.set_aborted(true).set_closed(true);
        }
    }
}

struct RequestErrorHook {
    response_status_code: ResponseStatusCode,
    response_body: String,
}

impl ServerHook for RequestErrorHook {
    async fn new(ctx: &mut Context) -> Self {
        let request_error: RequestError = ctx.try_get_request_error_data().unwrap_or_default();
        Self {
            response_status_code: request_error.get_http_status_code(),
            response_body: request_error.to_string(),
        }
    }

    async fn handle(self, ctx: &mut Context) {
        ctx.get_mut_response()
            .set_version(HttpVersion::Http1_1)
            .set_status_code(self.response_status_code)
            .set_body(self.response_body);
        if ctx.try_send().await.is_err() {
            ctx.set_aborted(true).set_closed(true);
        }
    }
}

struct RequestMiddleware {
    socket_addr: String,
}

impl ServerHook for RequestMiddleware {
    async fn new(ctx: &mut Context) -> Self {
        let mut socket_addr: String = String::new();
        if let Some(stream) = ctx.try_get_stream().as_ref() {
            socket_addr = stream
                .read()
                .await
                .peer_addr()
                .ok()
                .map(|data| data.to_string())
                .unwrap_or_default();
        }
        Self { socket_addr }
    }

    async fn handle(self, ctx: &mut Context) {
        ctx.get_mut_response()
            .set_version(HttpVersion::Http1_1)
            .set_status_code(200)
            .set_header(SERVER, HYPERLANE)
            .set_header(CONNECTION, KEEP_ALIVE)
            .set_header(CONTENT_TYPE, TEXT_PLAIN)
            .set_header(ACCESS_CONTROL_ALLOW_ORIGIN, WILDCARD_ANY)
            .set_header("SocketAddr", &self.socket_addr);
    }
}

struct UpgradeHook;

impl ServerHook for UpgradeHook {
    async fn new(_ctx: &mut Context) -> Self {
        Self
    }

    async fn handle(self, ctx: &mut Context) {
        if !ctx.get_request().is_ws_upgrade_type() {
            return;
        }
        if let Some(key) = &ctx.get_request().try_get_header_back(SEC_WEBSOCKET_KEY) {
            let accept_key: String = WebSocketFrame::generate_accept_key(key);
            ctx.get_mut_response()
                .set_version(HttpVersion::Http1_1)
                .set_status_code(101)
                .set_header(UPGRADE, WEBSOCKET)
                .set_header(CONNECTION, UPGRADE)
                .set_header(SEC_WEBSOCKET_ACCEPT, &accept_key)
                .set_body(vec![]);
            if ctx.try_send().await.is_err() {
                ctx.set_aborted(true).set_closed(true);
            }
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
    async fn new(ctx: &mut Context) -> Self {
        let group_name: String = ctx.try_get_route_param("group_name").unwrap_or_default();
        let group_broadcast_type: BroadcastType<String> = BroadcastType::PointToGroup(group_name);
        let receiver_count: ReceiverCount =
            get_broadcast_map().receiver_count(group_broadcast_type.clone());
        let my_name: String = ctx.try_get_route_param("my_name").unwrap_or_default();
        let your_name: String = ctx.try_get_route_param("your_name").unwrap_or_default();
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

    async fn handle(self, _ctx: &mut Context) {
        get_broadcast_map()
            .try_send(self.group_broadcast_type, self.data.clone())
            .unwrap_or_else(|err| {
                println!("[connected_hook] send group error => {:?}", err.to_string());
                None
            });
        get_broadcast_map()
            .try_send(self.private_broadcast_type, self.data)
            .unwrap_or_else(|err| {
                println!(
                    "[connected_hook] send private error => {:?}",
                    err.to_string()
                );
                None
            });
        println!(
            "[connected_hook] receiver_count => {:?}",
            self.receiver_count
        );
        Server::flush_stdout();
    }
}

struct SendedHook {
    msg: String,
}

impl ServerHook for SendedHook {
    async fn new(ctx: &mut Context) -> Self {
        let msg: String = ctx.get_response().get_body_string();
        Self { msg }
    }

    async fn handle(self, _ctx: &mut Context) {
        println!("[sended_hook] msg => {}", self.msg);
        Server::flush_stdout();
    }
}

struct GroupChatRequestHook {
    body: RequestBody,
    receiver_count: ReceiverCount,
}

impl ServerHook for GroupChatRequestHook {
    async fn new(ctx: &mut Context) -> Self {
        let group_name: String = ctx.try_get_route_param("group_name").unwrap();
        let key: BroadcastType<String> = BroadcastType::PointToGroup(group_name);
        let mut receiver_count: ReceiverCount = get_broadcast_map().receiver_count(key.clone());
        let mut body: RequestBody = ctx.get_request().get_body().clone();
        if body.is_empty() {
            receiver_count = get_broadcast_map().receiver_count_after_closed(key);
            body = format!("receiver_count => {receiver_count:?}").into();
        }
        Self {
            body,
            receiver_count,
        }
    }

    async fn handle(self, ctx: &mut Context) {
        ctx.get_mut_response().set_body(&self.body);
        println!("[group_chat] receiver_count => {:?}", self.receiver_count);
        Server::flush_stdout();
    }
}

struct GroupClosedHook {
    body: String,
    receiver_count: ReceiverCount,
}

impl ServerHook for GroupClosedHook {
    async fn new(ctx: &mut Context) -> Self {
        let group_name: String = ctx.try_get_route_param("group_name").unwrap();
        let key: BroadcastType<String> = BroadcastType::PointToGroup(group_name);
        let receiver_count: ReceiverCount =
            get_broadcast_map().receiver_count_after_closed(key.clone());
        let body: String = format!("receiver_count => {receiver_count:?}");
        Self {
            body,
            receiver_count,
        }
    }

    async fn handle(self, ctx: &mut Context) {
        ctx.get_mut_response().set_body(&self.body);
        println!("[group_closed] receiver_count => {:?}", self.receiver_count);
        Server::flush_stdout();
    }
}

struct GroupChat;

impl ServerHook for GroupChat {
    async fn new(_ctx: &mut Context) -> Self {
        Self
    }

    async fn handle(self, ctx: &mut Context) {
        let group_name: String = ctx.try_get_route_param("group_name").unwrap();
        let key: BroadcastType<String> = BroadcastType::PointToGroup(group_name);
        let config: WebSocketConfig<String> = WebSocketConfig::new(ctx)
            .set_capacity(1024)
            .set_broadcast_type(key)
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
    async fn new(ctx: &mut Context) -> Self {
        let my_name: String = ctx.try_get_route_param("my_name").unwrap();
        let your_name: String = ctx.try_get_route_param("your_name").unwrap();
        let key: BroadcastType<String> = BroadcastType::PointToPoint(my_name, your_name);
        let mut receiver_count: ReceiverCount = get_broadcast_map().receiver_count(key.clone());
        let mut body: RequestBody = ctx.get_request().get_body().clone();
        if body.is_empty() {
            receiver_count = get_broadcast_map().receiver_count_after_closed(key);
            body = format!("receiver_count => {receiver_count:?}").into();
        }
        Self {
            body,
            receiver_count,
        }
    }

    async fn handle(self, ctx: &mut Context) {
        ctx.get_mut_response().set_body(&self.body);
        println!("[private_chat] receiver_count => {:?}", self.receiver_count);
        Server::flush_stdout();
    }
}

struct PrivateClosedHook {
    body: String,
    receiver_count: ReceiverCount,
}

impl ServerHook for PrivateClosedHook {
    async fn new(ctx: &mut Context) -> Self {
        let my_name: String = ctx.try_get_route_param("my_name").unwrap();
        let your_name: String = ctx.try_get_route_param("your_name").unwrap();
        let key: BroadcastType<String> = BroadcastType::PointToPoint(my_name, your_name);
        let receiver_count: ReceiverCount = get_broadcast_map().receiver_count_after_closed(key);
        let body: String = format!("receiver_count => {receiver_count:?}");
        Self {
            body,
            receiver_count,
        }
    }

    async fn handle(self, ctx: &mut Context) {
        ctx.get_mut_response().set_body(&self.body);
        println!(
            "[private_closed] receiver_count => {:?}",
            self.receiver_count
        );
        Server::flush_stdout();
    }
}

struct PrivateChat;

impl ServerHook for PrivateChat {
    async fn new(_ctx: &mut Context) -> Self {
        Self
    }

    async fn handle(self, ctx: &mut Context) {
        let my_name: String = ctx.try_get_route_param("my_name").unwrap();
        let your_name: String = ctx.try_get_route_param("your_name").unwrap();
        let key: BroadcastType<String> = BroadcastType::PointToPoint(my_name, your_name);
        let config: WebSocketConfig<String> = WebSocketConfig::new(ctx)
            .set_capacity(1024)
            .set_broadcast_type(key)
            .set_connected_hook::<ConnectedHook>()
            .set_request_hook::<PrivateChatRequestHook>()
            .set_sended_hook::<SendedHook>()
            .set_closed_hook::<PrivateClosedHook>();
        get_broadcast_map().run(config).await;
    }
}

#[tokio::test]
async fn main() {
    let mut server: Server = Server::default();
    server.task_panic::<TaskPanicHook>();
    server.request_error::<RequestErrorHook>();
    server.request_middleware::<RequestMiddleware>();
    server.request_middleware::<UpgradeHook>();
    server.route::<GroupChat>("/{group_name}");
    server.route::<PrivateChat>("/{my_name}/{your_name}");
    let server_control_hook_1: ServerControlHook = server.run().await.unwrap_or_default();
    let server_control_hook_2: ServerControlHook = server_control_hook_1.clone();
    spawn(async move {
        sleep(Duration::from_secs(60)).await;
        server_control_hook_2.shutdown().await;
    });
    server_control_hook_1.wait().await;
}
