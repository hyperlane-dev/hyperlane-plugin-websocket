use crate::*;

#[tokio::test]
async fn test() {
    static BROADCAST_MAP: OnceLock<WebSocket> = OnceLock::new();

    fn get_broadcast_map() -> &'static WebSocket {
        BROADCAST_MAP.get_or_init(|| WebSocket::new())
    }

    async fn connected_hook(ctx: Context) {
        let group_name: String = ctx.get_route_param("group_name").await.unwrap();
        let broadcast_type: BroadcastType<&str> = BroadcastType::PointToGroup(&group_name);
        let receiver_count: ReceiverCount =
            get_broadcast_map().receiver_count_after_increment(broadcast_type);
        let data: String = format!("receiver_count => {:?}", receiver_count).into();
        get_broadcast_map()
            .send(broadcast_type, data)
            .unwrap_or_else(|err| {
                println!("[connected_hook]send error => {:?}", err.to_string());
                None
            });
        println!("[connected_hook]receiver_count => {:?}", receiver_count);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    async fn group_chat_hook(ws_ctx: Context) {
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

    async fn private_chat_hook(ctx: Context) {
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
        println!("[sended_hook]msg => {}", msg);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    async fn private_chat(ctx: Context) {
        let my_name: String = ctx.get_route_param("my_name").await.unwrap();
        let your_name: String = ctx.get_route_param("your_name").await.unwrap();
        let key: BroadcastType<&str> = BroadcastType::PointToPoint(&my_name, &your_name);
        get_broadcast_map()
            .run(&ctx, 1024, key, private_chat_hook, sended, private_closed)
            .await;
    }

    async fn group_chat(ctx: Context) {
        let group_name: String = ctx.get_route_param("group_name").await.unwrap();
        let key: BroadcastType<&str> = BroadcastType::PointToGroup(&group_name);
        get_broadcast_map()
            .run(&ctx, 1024, key, group_chat_hook, sended, group_closed)
            .await;
    }

    async fn main() {
        let server: Server = Server::new();
        server.host("0.0.0.0").await;
        server.port(60000).await;
        server.disable_ws_hook("/{group_name}").await;
        server.route("/{group_name}", group_chat).await;
        server.disable_ws_hook("/{my_name}/{your_name}").await;
        server.route("/{my_name}/{your_name}", private_chat).await;
        server.connected_hook(connected_hook).await;
        let result: ServerResult<()> = server.run().await;
        println!("Server result: {:?}", result);
        let _ = std::io::Write::flush(&mut std::io::stderr());
    }

    let _ = tokio::time::timeout(std::time::Duration::from_secs(60), main()).await;
}
