<center>

## hyperlane-plugin-websocket

[![](https://img.shields.io/crates/v/hyperlane-plugin-websocket.svg)](https://crates.io/crates/hyperlane-plugin-websocket)
[![](https://img.shields.io/crates/d/hyperlane-plugin-websocket.svg)](https://img.shields.io/crates/d/hyperlane-plugin-websocket.svg)
[![](https://docs.rs/hyperlane-plugin-websocket/badge.svg)](https://docs.rs/hyperlane-plugin-websocket)
[![](https://github.com/eastspire/hyperlane-plugin-websocket/workflows/Rust/badge.svg)](https://github.com/eastspire/hyperlane-plugin-websocket/actions?query=workflow:Rust)
[![](https://img.shields.io/crates/l/hyperlane-plugin-websocket.svg)](./LICENSE)

</center>

[Official Documentation](https://docs.ltpp.vip/hyperlane-plugin-websocket/)

[Api Docs](https://docs.rs/hyperlane-plugin-websocket/latest/http_type/)

> A websocket plugin for the hyperlane framework.

## Installation

To use this crate, you can run cmd:

```shell
cargo add hyperlane-plugin-websocket
```

## Use

```rust
use hyperlane::*;
use hyperlane_plugin_websocket::*;

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
    println!("[sended_hook]msg => {}", msg);
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
    let group_name: String = ctx.get_route_param("group_name").await.unwrap();
    let key: BroadcastType<&str> = BroadcastType::PointToGroup(&group_name);
    get_broadcast_map()
        .run(&ctx, 1024, key, group_chat, sended, group_closed)
        .await;
}

#[tokio::main]
async fn main() {
    let server: Server = Server::new();
    server.host("0.0.0.0").await;
    server.port(60000).await;
    server.disable_ws_hook("/{group_name}").await;
    server.route("/{group_name}", group_chat_route).await;
    server.disable_ws_hook("/{my_name}/{your_name}").await;
    server
        .route("/{my_name}/{your_name}", private_chat_route)
        .await;
    server.connected_hook(connected_hook).await;
    let result: ServerResult<()> = server.run().await;
    println!("Server result: {:?}", result);
    let _ = std::io::Write::flush(&mut std::io::stderr());
}
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Contact

For any inquiries, please reach out to the author at [root@ltpp.vip](mailto:root@ltpp.vip).
