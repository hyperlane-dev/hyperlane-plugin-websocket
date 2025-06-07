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
use hyperlane_utils::*;

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

#[tokio::main]
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
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Contact

For any inquiries, please reach out to the author at [root@ltpp.vip](mailto:root@ltpp.vip).
