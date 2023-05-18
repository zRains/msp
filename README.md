# Msp (alpha)

![license](https://flat.badgen.net/badge/license/MIT/blue) ![version](https://flat.badgen.net/crates/v/msp) ![download](https://flat.badgen.net/crates/d/msp)

> WIP: 目前，它处于 alpha 阶段。它已可以正常使用，但配置和功能 API 在次要版本发布之间仍可能发生变化。

一个通过 Rust 实现的快速响应、轻量、稳定、完善的类型导出和错误回馈的 Minecraft Server Protocol 客户端。通过不同协议获取服务器状态，并以强类型 JSON 数据返回。

### 支持版本

目前仅支持 Java 平台的服务器，计划支持 Bedrock 版本。后续将推出命令行版本并开启版本发布。

**Java Edition:** 适用于 1.4 版本及以上的服务端（[Protocol version number](https://wiki.vg/Protocol_version_numbers) >= 47）。

**Bedrock:** 计划中。。。

### 支持协议

- [x] [Server List Ping](https://wiki.vg/Server_List_Ping) 适用于大部分现代服务器（1.7+）。
- [x] [Netty Server Ping](https://wiki.vg/Server_List_Ping#1.6) 适用于 1.6 及之后的服务器。
- [x] [Legacy Server Ping](https://wiki.vg/Server_List_Ping#1.4_to_1.5) 适用于老版本服务器（1.4 ~ 1.5）。
- [x] [Beta Legacy Server Ping](https://wiki.vg/Server_List_Ping#Beta_1.8_to_1.3) 适用于上古版本服务器（Beta 1.8 ~ 1.3）。
- [x] [Ping via LAN](https://wiki.vg/Server_List_Ping#Ping_via_LAN_.28Open_to_LAN_in_Singleplayer.29) 局域网发现协议。
- [x] [Query Protocol](https://wiki.vg/Query) 适用于现代 Java Edition 服务端（1.9pre4 及后续版本可用），适用此协议需要服务端开启响应功能。

### 使用

1. 作为库集成到自己的 Rust 项目中，在根项目下运行：

```bash
cargo add msp
```

或者将此依赖添加到`Cargo.toml`文件：

```toml
[dependencies]
msp = "0.1.0"
```

2. 作为命令行工具（CLI）使用：

计划中。。。

### 使用方法

通过`get_server_status`获取服务器信息，老旧版本将不受支持：

```rust
use msp::{Msp, MspErr};

fn main() -> Result<(), MspErr> {
    let server = Msp::create("pvp.desteria.com");
    let info: Msp::Server = server.get_server_status()?;

    println!("{}", server.get_server_status()?);

    Ok(())
}
```

使用`query_full`获取服务器信息：

```rust
let server_query: Msp::QueryFull = server.query_full()?;
```

:warning:注意：要使用此协议，你需要开启服务端的`enable-query`选项，此选项可在根目录下的`server.properties`找到，并设置：

```toml
enable-query=true
query.port=25565 # 根据情况进行端口配置
```

### 计划

1. 支持 Bedrock 版本服务器查询。
2. 编写功能函数测试。
3. 完成命令行版发行。

### 许可

MIT.
