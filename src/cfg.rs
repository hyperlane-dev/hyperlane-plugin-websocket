use crate::*;

#[tokio::test]
async fn test() {
    static BROADCAST_MAP: OnceLock<WebSocket> = OnceLock::new();

    fn get_broadcast_map() -> &'static WebSocket {
        BROADCAST_MAP.get_or_init(|| WebSocket::new())
    }

    async fn callback(ws_ctx: Context) {
        let group_name: String = ws_ctx.get_route_param("group_name").await.unwrap();
        let receiver_count: OptionReceiverCount =
            get_broadcast_map().receiver_count(BroadcastType::PointToGroup(&group_name));
        let body: RequestBody = ws_ctx.get_request_body().await;
        ws_ctx.set_response_body(body).await;
        println!("receiver_count => {:?}", receiver_count);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    async fn send_callback(_: Context) {}

    async fn client_closed_callback(ctx: Context) {
        callback(ctx).await;
    }

    async fn private_chat(ctx: Context) {
        let my_name: String = ctx.get_route_param("my_name").await.unwrap();
        let your_name: String = ctx.get_route_param("your_name").await.unwrap();
        get_broadcast_map()
            .run(
                &ctx,
                DEFAULT_BUFFER_SIZE,
                BroadcastType::PointToPoint(&my_name, &your_name),
                callback,
                send_callback,
                client_closed_callback,
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
                send_callback,
                client_closed_callback,
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
        server.websocket_buffer_size(4096).await;
        server.disable_inner_websocket_handle("/{group_name}").await;
        server.route("/{group_name}", group_chat).await;
        server
            .disable_inner_websocket_handle("/{my_name}/{your_name}")
            .await;
        server.route("/{my_name}/{your_name}", private_chat).await;
        server.run().await.unwrap();
    }

    let _ = tokio::time::timeout(std::time::Duration::from_secs(60), main()).await;
}
