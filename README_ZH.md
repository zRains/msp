# Msp (alpha)

![license](https://flat.badgen.net/badge/license/MIT/blue) ![version](https://flat.badgen.net/crates/v/msp) ![download](https://flat.badgen.net/crates/d/msp)

<p><a href="README.md">English</a> | 中文</p>

> WIP: 目前，它处于 alpha 阶段。它已可以正常使用，但配置和功能 API 在次要版本发布之间仍可能发生变化。

一个通过 Rust 实现的快速响应、轻量、稳定、完善的类型导出和错误回馈的 Minecraft Server Protocol 客户端。通过不同协议获取服务器状态，并以强类型 JSON 数据返回。

### 支持版本

支持 Java Edition 和 BedrockEdition 服务器，适用版本范围如下。

**Java Edition:** 适用于 1.4 版本及以上的服务端（[Protocol version number](https://wiki.vg/Protocol_version_numbers) >= 47）。

**Bedrock Edition:** 适用于现代 Bedrock 服务器（1.16.220 及以上）。

### 支持协议

- [x] [Server List Ping](https://wiki.vg/Server_List_Ping) 适用于大部分现代服务器（1.7+）。
- [x] [Netty Server Ping](https://wiki.vg/Server_List_Ping#1.6) 适用于 1.6 及之后的服务器。
- [x] [Legacy Server Ping](https://wiki.vg/Server_List_Ping#1.4_to_1.5) 适用于老版本服务器（1.4 ~ 1.5）。
- [x] [Beta Legacy Server Ping](https://wiki.vg/Server_List_Ping#Beta_1.8_to_1.3) 适用于上古版本服务器（Beta 1.8 ~ 1.3）。
- [x] [Ping via LAN](https://wiki.vg/Server_List_Ping#Ping_via_LAN_.28Open_to_LAN_in_Singleplayer.29) 局域网服务器发现协议。
- [x] [Raknet Protocol](https://wiki.vg/Raknet_Protocol) 适用于现代 Bedrock 服务器。
- [x] [Query Protocol](https://wiki.vg/Query) 适用于现代 Java Edition 服务端（1.9pre4 及后续版本可用），适用此协议需要服务端开启响应功能。

### 使用

1. 作为库集成到自己的 Rust 项目中，在根项目下运行：

```bash
cargo add msp
```

或者将此依赖添加到你的`Cargo.toml`文件：

```toml
[dependencies]
msp = "0.1.2"
```

### 使用方法

下面是一些基本使用示例。

1. 通过`get_server_status`获取服务器信息，注意老旧版本将不受支持：

```rust
use msp::{Conf, MspErr, Server};

fn main() -> Result<(), MspErr> {
    let server = Conf::create_with_port("www.example.com", 25565);
    let info: Server = server.get_server_status()?;

    println!("{}", info);

    Ok(())
}
```

2. 使用`Conf::create_with_port`创建一个指定端口的连接配置：

```rust
use msp::{Conf, MspErr, Server};

fn main() -> Result<(), MspErr> {
    let server = Conf::create_with_port("www.example.com", 25565);
    let info: Server = server.get_server_status()?;

    println!("{}", info);

    Ok(())
}
```

3. 使用`get_lan_server_status`发现局域网内的在线服务器：

```rust
use msp::{get_lan_server_status, MspErr, SocketConf};

fn main() -> Result<(), MspErr> {
    get_lan_server_status(&SocketConf::default())?;

    Ok(())
}
```

4. 使用`Conf::query_full`通过 Query 协议获取服务器信息：

```rust
use msp::{Conf, MspErr};

fn main() -> Result<(), MspErr> {
    let server = Conf::create_with_port("www.example.com", 25565);

    println!("{}", server.query_full()?);

    Ok(())
}
```

:warning:注意：要使用此协议，你需要开启服务端的`enable-query`选项，此选项可在根目录下的`server.properties`找到，并设置：

```toml
enable-query=true
query.port=25565 # 根据情况进行端口配置
```

### 许可

MIT.
