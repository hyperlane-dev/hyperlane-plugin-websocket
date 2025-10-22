use crate::*;

#[tokio::test]
async fn test_server() {
    struct RequestMiddleware;
    struct UpgradeHook;
    struct GroupChat;
    struct PrivateChat;
    struct ConnectedHook;
    struct PrivateClosedHook;
    struct SendedHook;
    struct GroupChatRequestHook;
    struct GroupClosedHook;
    struct PrivateChatRequestHook;

    static BROADCAST_MAP: OnceLock<WebSocket> = OnceLock::new();

    fn get_broadcast_map() -> &'static WebSocket {
        BROADCAST_MAP.get_or_init(|| WebSocket::new())
    }

    impl ServerHook for RequestMiddleware {
        async fn new(_ctx: &Context) -> Self {
            Self
        }

        async fn handle(self, ctx: &Context) {
            let socket_addr: String = ctx.get_socket_addr_string().await;
            ctx.set_response_version(HttpVersion::HTTP1_1)
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
                .set_response_header("SocketAddr", &socket_addr)
                .await;
        }
    }

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
                ctx.set_response_status_code(101)
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
                    .await
                    .unwrap();
            }
        }
    }

    impl ServerHook for ConnectedHook {
        async fn new(_ctx: &Context) -> Self {
            Self
        }

        async fn handle(self, ctx: &Context) {
            let group_name: String = ctx
                .try_get_route_param("group_name")
                .await
                .unwrap_or_default();
            let group_broadcast_type: BroadcastType<String> =
                BroadcastType::PointToGroup(group_name);
            let receiver_count: ReceiverCount =
                get_broadcast_map().receiver_count(group_broadcast_type.clone());
            let my_name: String = ctx.try_get_route_param("my_name").await.unwrap_or_default();
            let your_name: String = ctx
                .try_get_route_param("your_name")
                .await
                .unwrap_or_default();
            let private_broadcast_type: BroadcastType<String> =
                BroadcastType::PointToPoint(my_name, your_name);
            let data: String = format!("receiver_count => {:?}", receiver_count).into();
            get_broadcast_map()
                .send(group_broadcast_type, data.clone())
                .unwrap_or_else(|err| {
                    println!("[connected_hook]send group error => {:?}", err.to_string());
                    None
                });
            get_broadcast_map()
                .send(private_broadcast_type, data)
                .unwrap_or_else(|err| {
                    println!(
                        "[connected_hook]send private error => {:?}",
                        err.to_string()
                    );
                    None
                });
            println!("[connected_hook]receiver_count => {:?}", receiver_count);
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
    }

    impl ServerHook for GroupChatRequestHook {
        async fn new(_ctx: &Context) -> Self {
            Self
        }

        async fn handle(self, ctx: &Context) {
            let group_name: String = ctx.try_get_route_param("group_name").await.unwrap();
            let key: BroadcastType<String> = BroadcastType::PointToGroup(group_name);
            let mut receiver_count: ReceiverCount = get_broadcast_map().receiver_count(key.clone());
            let mut body: RequestBody = ctx.get_request_body().await;
            if body.is_empty() {
                receiver_count = get_broadcast_map().receiver_count_after_closed(key);
                body = format!("receiver_count => {:?}", receiver_count).into();
            }
            ctx.set_response_body(&body).await;
            println!("[group_chat]receiver_count => {:?}", receiver_count);
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
    }

    impl ServerHook for GroupClosedHook {
        async fn new(_ctx: &Context) -> Self {
            Self
        }

        async fn handle(self, ctx: &Context) {
            let group_name: String = ctx.try_get_route_param("group_name").await.unwrap();
            let key: BroadcastType<String> = BroadcastType::PointToGroup(group_name);
            let receiver_count: ReceiverCount =
                get_broadcast_map().receiver_count_after_closed(key.clone());
            let body: String = format!("receiver_count => {:?}", receiver_count);
            ctx.set_response_body(&body).await;
            println!("[group_closed]receiver_count => {:?}", receiver_count);
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
    }

    impl ServerHook for PrivateChatRequestHook {
        async fn new(_ctx: &Context) -> Self {
            Self
        }

        async fn handle(self, ctx: &Context) {
            let my_name: String = ctx.try_get_route_param("my_name").await.unwrap();
            let your_name: String = ctx.try_get_route_param("your_name").await.unwrap();
            let key: BroadcastType<String> = BroadcastType::PointToPoint(my_name, your_name);
            let mut receiver_count: ReceiverCount = get_broadcast_map().receiver_count(key.clone());
            let mut body: RequestBody = ctx.get_request_body().await;
            if body.is_empty() {
                receiver_count = get_broadcast_map().receiver_count_after_closed(key);
                body = format!("receiver_count => {:?}", receiver_count).into();
            }
            ctx.set_response_body(&body).await;
            println!("[private_chat]receiver_count => {:?}", receiver_count);
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
    }

    impl ServerHook for PrivateClosedHook {
        async fn new(_ctx: &Context) -> Self {
            Self
        }

        async fn handle(self, ctx: &Context) {
            let my_name: String = ctx.try_get_route_param("my_name").await.unwrap();
            let your_name: String = ctx.try_get_route_param("your_name").await.unwrap();
            let key: BroadcastType<String> = BroadcastType::PointToPoint(my_name, your_name);
            let receiver_count: ReceiverCount =
                get_broadcast_map().receiver_count_after_closed(key);
            let body: String = format!("receiver_count => {:?}", receiver_count);
            ctx.set_response_body(&body).await;
            println!("[private_closed]receiver_count => {:?}", receiver_count);
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
    }

    impl ServerHook for SendedHook {
        async fn new(_ctx: &Context) -> Self {
            Self
        }

        async fn handle(self, ctx: &Context) {
            let msg: String = ctx.get_response_body_string().await;
            println!("[sended_hook]msg => {}", msg);
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
    }

    impl ServerHook for PrivateChat {
        async fn new(_ctx: &Context) -> Self {
            Self
        }

        async fn handle(self, ctx: &Context) {
            let my_name: String = ctx.try_get_route_param("my_name").await.unwrap();
            let your_name: String = ctx.try_get_route_param("your_name").await.unwrap();
            let key: BroadcastType<String> = BroadcastType::PointToPoint(my_name, your_name);
            let config: WebSocketConfig<String> = WebSocketConfig::new()
                .set_context(ctx.clone())
                .set_broadcast_type(key)
                .set_buffer_size(4096)
                .set_capacity(1024)
                .set_connected_hook::<ConnectedHook>()
                .set_request_hook::<PrivateChatRequestHook>()
                .set_sended_hook::<SendedHook>()
                .set_closed_hook::<PrivateClosedHook>();
            get_broadcast_map().run(config).await;
        }
    }

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
                .set_buffer_size(4096)
                .set_capacity(1024)
                .set_connected_hook::<ConnectedHook>()
                .set_request_hook::<GroupChatRequestHook>()
                .set_sended_hook::<SendedHook>()
                .set_closed_hook::<GroupClosedHook>();
            get_broadcast_map().run(config).await;
        }
    }

    async fn main() {
        let server: Server = Server::new().await;
        let config: ServerConfig = ServerConfig::new().await;
        config.host("0.0.0.0").await;
        config.port(60000).await;
        config.buffer(4096).await;
        config.disable_linger().await;
        config.disable_nodelay().await;
        server.config(config).await;
        server.request_middleware::<RequestMiddleware>().await;
        server.request_middleware::<UpgradeHook>().await;
        server.route::<GroupChat>("/{group_name}").await;
        server.route::<PrivateChat>("/{my_name}/{your_name}").await;
        let server_hook: ServerControlHook = server.run().await.unwrap_or_default();
        let server_hook_clone: ServerControlHook = server_hook.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            server_hook.shutdown().await;
        });
        server_hook_clone.wait().await;
    }

    main().await;
}
