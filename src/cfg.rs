use crate::*;

#[tokio::test]
async fn test() {
    static BROADCAST_MAP: OnceLock<WebSocket> = OnceLock::new();

    fn get_broadcast_map() -> &'static WebSocket {
        BROADCAST_MAP.get_or_init(|| WebSocket::new())
    }

    async fn websocket_callback(ws_ctx: Context) {
        let body: RequestBody = ws_ctx.get_request_body().await;
        ws_ctx.set_response_body(body).await;
    }

    async fn private_chat(ctx: Context) {
        let my_name: String = ctx.get_route_param("my_name").await.unwrap();
        let your_name: String = ctx.get_route_param("your_name").await.unwrap();
        get_broadcast_map()
            .run(
                &ctx,
                BroadcastType::PointToPoint(&my_name, &your_name),
                websocket_callback,
            )
            .await;
    }

    async fn group_chat(ctx: Context) {
        let your_name: String = ctx.get_route_param("group_name").await.unwrap();
        get_broadcast_map()
            .run(
                &ctx,
                BroadcastType::PointToGroup(&your_name),
                websocket_callback,
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
        server.disable_inner_websocket_handle("/:group_name").await;
        server.route("/:group_name", group_chat).await;
        server
            .disable_inner_websocket_handle("/:my_name/:your_name")
            .await;
        server.route("/:my_name/:your_name", private_chat).await;
        server.run().await.unwrap();
    }

    let _ = tokio::time::timeout(std::time::Duration::from_secs(60), main()).await;
}
