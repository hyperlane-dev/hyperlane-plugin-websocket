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
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Contact

For any inquiries, please reach out to the author at [root@ltpp.vip](mailto:root@ltpp.vip).
