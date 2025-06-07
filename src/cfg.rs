use crate::*;

#[tokio::test]
async fn test() {
    static BROADCAST_MAP: OnceLock<WebSocket> = OnceLock::new();

    fn get_broadcast_map() -> &'static WebSocket {
        BROADCAST_MAP.get_or_init(|| WebSocket::new())
    }

    async fn on_ws_connected(ctx: Context) {
        tokio::spawn(async move {
            let group_name: String = ctx.get_route_param("group_name").await.unwrap();
            let receiver_count: OptionReceiverCount =
                get_broadcast_map().receiver_count(BroadcastType::PointToGroup(&group_name));
            let body: String = format!("receiver_count => {:?}", receiver_count).into();
            ctx.set_response_body(body).await;
            println!("receiver_count => {:?}", receiver_count);
            let _ = std::io::Write::flush(&mut std::io::stderr());
        });
    }

    async fn callback(ws_ctx: Context) {
        let group_name: String = ws_ctx.get_route_param("group_name").await.unwrap();
        let mut receiver_count: OptionReceiverCount =
            get_broadcast_map().receiver_count(BroadcastType::PointToGroup(&group_name));
        let mut body: RequestBody = ws_ctx.get_request_body().await;
        if body.is_empty() {
            receiver_count = get_broadcast_map()
                .pre_decrement_receiver_count(BroadcastType::PointToGroup(&group_name));
            body = format!("receiver_count => {:?}", receiver_count).into();
        }
        ws_ctx.set_response_body(body).await;
        println!("receiver_count => {:?}", receiver_count);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    async fn on_sended(_: Context) {}

    async fn private_chat(ctx: Context) {
        let my_name: String = ctx.get_route_param("my_name").await.unwrap();
        let your_name: String = ctx.get_route_param("your_name").await.unwrap();
        get_broadcast_map()
            .run(
                &ctx,
                DEFAULT_BUFFER_SIZE,
                BroadcastType::PointToPoint(&my_name, &your_name),
                callback,
                on_sended,
                callback,
            )
            .await;
    }

    async fn group_chat(ctx: Context) {
        let your_name: String = ctx.get_route_param("group_name").await.unwrap();
        get_broadcast_map()
            .run(
                &ctx,
                DEFAULT_BUFFER_SIZE,
                BroadcastType::PointToGroup(&your_name),
                callback,
                on_sended,
                callback,
            )
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
        server.route("/{group_name}", group_chat).await;
        server
            .disable_internal_ws_handler("/{my_name}/{your_name}")
            .await;
        server.route("/{my_name}/{your_name}", private_chat).await;
        server.on_ws_connected(on_ws_connected).await;
        server.run().await.unwrap();
    }

    let _ = tokio::time::timeout(std::time::Duration::from_secs(60), main()).await;
}
