<center>

## hyperlane-plugin-websocket

[![](https://img.shields.io/crates/v/hyperlane-plugin-websocket.svg)](https://crates.io/crates/hyperlane-plugin-websocket)
[![](https://img.shields.io/crates/d/hyperlane-plugin-websocket.svg)](https://img.shields.io/crates/d/hyperlane-plugin-websocket.svg)
[![](https://docs.rs/hyperlane-plugin-websocket/badge.svg)](https://docs.rs/hyperlane-plugin-websocket)
[![](https://github.com/hyperlane-dev/hyperlane-plugin-websocket/workflows/Rust/badge.svg)](https://github.com/hyperlane-dev/hyperlane-plugin-websocket/actions?query=workflow:Rust)
[![](https://img.shields.io/crates/l/hyperlane-plugin-websocket.svg)](./LICENSE)

</center>

[Official Documentation](https://docs.ltpp.vip/hyperlane-plugin-websocket/)

[Api Docs](https://docs.rs/hyperlane-plugin-websocket/latest/http_type/)

> A WebSocket plugin for the Hyperlane framework, providing robust WebSocket communication capabilities and integrating with hyperlane-broadcast for efficient message dissemination.

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
    let group_name: String = ctx.try_get_route_param("group_name").await.unwrap();
    let broadcast_type: BroadcastType<String> = BroadcastType::PointToGroup(group_name);
    let receiver_count: ReceiverCount =
        get_broadcast_map().receiver_count_after_increment(broadcast_type.clone());
    let data: String = format!("receiver_count => {:?}", receiver_count).into();
    tokio::spawn(async move {
        tokio::task::yield_now().await;
        get_broadcast_map()
            .send(broadcast_type, data.clone())
            .unwrap_or_else(|err| {
                println!("[connected_hook]send error => {:?}", err.to_string());
                None
            });
    });
    println!("[connected_hook]receiver_count => {:?}", receiver_count);
    let _ = std::io::Write::flush(&mut std::io::stderr());
}

async fn group_chat_hook(ws_ctx: Context) {
    let group_name: String = ws_ctx.try_get_route_param("group_name").await.unwrap();
    let key: BroadcastType<String> = BroadcastType::PointToGroup(group_name);
    let mut receiver_count: ReceiverCount = get_broadcast_map().receiver_count(key.clone());
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
    let group_name: String = ctx.try_get_route_param("group_name").await.unwrap();
    let key: BroadcastType<String> = BroadcastType::PointToGroup(group_name);
    let receiver_count: ReceiverCount =
        get_broadcast_map().receiver_count_after_decrement(key.clone());
    let body: String = format!("receiver_count => {:?}", receiver_count);
    ctx.set_response_body(body).await;
    println!("[group_closed]receiver_count => {:?}", receiver_count);
    let _ = std::io::Write::flush(&mut std::io::stderr());
}

async fn private_chat_hook(ctx: Context) {
    let my_name: String = ctx.try_get_route_param("my_name").await.unwrap();
    let your_name: String = ctx.try_get_route_param("your_name").await.unwrap();
    let key: BroadcastType<String> = BroadcastType::PointToPoint(my_name, your_name);
    let mut receiver_count: ReceiverCount = get_broadcast_map().receiver_count(key.clone());
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
    let my_name: String = ctx.try_get_route_param("my_name").await.unwrap();
    let your_name: String = ctx.try_get_route_param("your_name").await.unwrap();
    let key: BroadcastType<String> = BroadcastType::PointToPoint(my_name, your_name);
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
    let my_name: String = ctx.try_get_route_param("my_name").await.unwrap();
    let your_name: String = ctx.try_get_route_param("your_name").await.unwrap();
    let key: BroadcastType<String> = BroadcastType::PointToPoint(my_name, your_name);
    let config: WebSocketConfig<String> = WebSocketConfig::new()
        .set_context(ctx.clone())
        .set_broadcast_type(key)
        .set_buffer_size(4096)
        .set_capacity(1024)
        .set_request_hook(private_chat_hook)
        .set_sended_hook(sended)
        .set_closed_hook(private_closed);
    get_broadcast_map().run(config).await;
}

async fn group_chat(ctx: Context) {
    let group_name: String = ctx.try_get_route_param("group_name").await.unwrap();
    let key: BroadcastType<String> = BroadcastType::PointToGroup(group_name);
    let config: WebSocketConfig<String> = WebSocketConfig::new()
        .set_context(ctx.clone())
        .set_broadcast_type(key)
        .set_buffer_size(4096)
        .set_capacity(1024)
        .set_request_hook(group_chat_hook)
        .set_sended_hook(sended)
        .set_closed_hook(group_closed);
    get_broadcast_map().run(config).await;
}

#[tokio::main]
async fn main() {
    let server: Server = Server::new().await;
    let config: ServerConfig = ServerConfig::new().await;
    config.host("0.0.0.0").await;
    config.port(60000).await;
    config.enable_nodelay().await;
    config.http_buffer(4096).await;
    config.ws_buffer(4096).await;
    server.config(config).await;
    server.disable_ws_hook("/{group_name}").await;
    server.route("/{group_name}", group_chat).await;
    server.disable_ws_hook("/{my_name}/{your_name}").await;
    server.route("/{my_name}/{your_name}", private_chat).await;
    server.connected_hook(connected_hook).await;
    let server_hook: ServerHook = server.run().await.unwrap_or_default();
    server_hook.wait().await;
}
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Contact

For any inquiries, please reach out to the author at [root@ltpp.vip](mailto:root@ltpp.vip).
