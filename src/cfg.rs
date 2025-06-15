use crate::*;

#[tokio::test]
async fn test() {
    static BROADCAST_MAP: OnceLock<WebSocket> = OnceLock::new();

    #[allow(non_local_definitions)]
    impl BroadcastTypeTrait for &str {}

    fn get_broadcast_map() -> &'static WebSocket {
        BROADCAST_MAP.get_or_init(|| WebSocket::new())
    }

    async fn on_ws_connected(ctx: Context) {
        let group_name: String = ctx.get_route_param("group_name").await.unwrap();
        let broadcast_type: BroadcastType<&str> = BroadcastType::PointToGroup(&group_name);
        let receiver_count: ReceiverCount =
            get_broadcast_map().receiver_count_after_increment(broadcast_type);
        let data: String = format!("receiver_count => {:?}", receiver_count).into();
        get_broadcast_map()
            .send(broadcast_type, data)
            .unwrap_or_else(|err| {
                println!("[on_ws_connected]send error => {:?}", err.to_string());
                None
            });
        println!("[on_ws_connected]receiver_count => {:?}", receiver_count);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    async fn group_chat(ws_ctx: Context) {
        let group_name: String = ws_ctx.get_route_param("group_name").await.unwrap();
        let key: BroadcastType<&str> = BroadcastType::PointToGroup(&group_name);
        let mut receiver_count: ReceiverCount = get_broadcast_map().receiver_count(key);
        let mut body: RequestBody = ws_ctx.get_request_body().await;
        if body.is_empty() {
            receiver_count = get_broadcast_map().receiver_count_after_decrement(key);
            body = format!("receiver_count => {:?}", receiver_count).into();
        }
        ws_ctx.set_response_body(body).await;
        println!("[group_chat]receiver_count => {:?}", receiver_count);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    async fn group_closed(ctx: Context) {
        let group_name: String = ctx.get_route_param("group_name").await.unwrap();
        let key: BroadcastType<&str> = BroadcastType::PointToGroup(&group_name);
        let receiver_count: ReceiverCount = get_broadcast_map().receiver_count_after_decrement(key);
        let body: String = format!("receiver_count => {:?}", receiver_count);
        ctx.set_response_body(body).await;
        println!("[group_closed]receiver_count => {:?}", receiver_count);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    async fn private_chat(ctx: Context) {
        let my_name: String = ctx.get_route_param("my_name").await.unwrap();
        let your_name: String = ctx.get_route_param("your_name").await.unwrap();
        let key: BroadcastType<&str> = BroadcastType::PointToPoint(&my_name, &your_name);
        let mut receiver_count: ReceiverCount = get_broadcast_map().receiver_count(key);
        let mut body: RequestBody = ctx.get_request_body().await;
        if body.is_empty() {
            receiver_count = get_broadcast_map().receiver_count_after_decrement(key);
            body = format!("receiver_count => {:?}", receiver_count).into();
        }
        ctx.set_response_body(body).await;
        println!("[private_chat]receiver_count => {:?}", receiver_count);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    async fn private_closed(ctx: Context) {
        let my_name: String = ctx.get_route_param("my_name").await.unwrap();
        let your_name: String = ctx.get_route_param("your_name").await.unwrap();
        let key: BroadcastType<&str> = BroadcastType::PointToPoint(&my_name, &your_name);
        let receiver_count: ReceiverCount = get_broadcast_map().receiver_count_after_decrement(key);
        let body: String = format!("receiver_count => {:?}", receiver_count);
        ctx.set_response_body(body).await;
        println!("[private_closed]receiver_count => {:?}", receiver_count);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    async fn sended(ctx: Context) {
        let msg: String = ctx.get_response_body_string().await;
        println!("[on_sended]msg => {}", msg);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    async fn private_chat_route(ctx: Context) {
        let my_name: String = ctx.get_route_param("my_name").await.unwrap();
        let your_name: String = ctx.get_route_param("your_name").await.unwrap();
        let key: BroadcastType<&str> = BroadcastType::PointToPoint(&my_name, &your_name);
        get_broadcast_map()
            .run(&ctx, 1024, key, private_chat, sended, private_closed)
            .await;
    }

    async fn group_chat_route(ctx: Context) {
        let your_name: String = ctx.get_route_param("group_name").await.unwrap();
        let key: BroadcastType<&str> = BroadcastType::PointToGroup(&your_name);
        get_broadcast_map()
            .run(&ctx, 1024, key, group_chat, sended, group_closed)
            .await;
    }

    async fn main() {
        let server: Server = Server::new();
        server.host("0.0.0.0").await;
        server.port(60000).await;
        server.enable_nodelay().await;
        server.disable_linger().await;
        server.http_line_buffer_size(4096).await;
        server.ws_buffer_size(4096).await;
        server.disable_internal_ws_handler("/{group_name}").await;
        server.route("/{group_name}", group_chat_route).await;
        server
            .disable_internal_ws_handler("/{my_name}/{your_name}")
            .await;
        server
            .route("/{my_name}/{your_name}", private_chat_route)
            .await;
        server.on_ws_connected(on_ws_connected).await;
        server.run().await.unwrap();
    }

    let _ = tokio::time::timeout(std::time::Duration::from_secs(60), main()).await;
}
