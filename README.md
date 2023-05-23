# Msp (alpha)

![license](https://flat.badgen.net/badge/license/MIT/blue) ![version](https://flat.badgen.net/crates/v/msp) ![download](https://flat.badgen.net/crates/d/msp)

<p>English | <a href="README_ZH.md">中文</a></p>

> WIP: Currently, it is in the alpha stage. It is functional and can be used, but configuration and feature APIs may still undergo changes between minor version releases.

A Minecraft Server Protocol client implemented in Rust that offers fast response, lightweightness, stability, comprehensive type exporting, and error feedback. It enables retrieval of server status through various protocols and returns strongly-typed JSON data.

### Applicable version

Supports Java Edition and Bedrock Edition servers. The applicable version range is as follows.

- **Java Edition:** Suitable for server versions 1.4 and above ([Protocol version number](https://wiki.vg/Protocol_version_numbers) >= 47).
- **Bedrock Edition:** Suitable for modern Bedrock servers (1.16.220 and above).

### Supported protocols

Server Information Query Protocol covering most versions, with certain protocols requiring the server to enable corresponding features.

- [Server List Ping](https://wiki.vg/Server_List_Ping) Suitable for most modern servers (1.7+).
- [Netty Server Ping](https://wiki.vg/Server_List_Ping#1.6) Suitable for servers 1.6 and later.
- [Legacy Server Ping](https://wiki.vg/Server_List_Ping#1.4_to_1.5) Suitable for older versions of servers (1.4 to 1.5).
- [Beta Legacy Server Ping](https://wiki.vg/Server_List_Ping#Beta_1.8_to_1.3) Suitable for ancient versions of servers (Beta 1.8 to 1.3).
- [Ping via LAN](https://wiki.vg/Server_List_Ping#Ping_via_LAN_.28Open_to_LAN_in_Singleplayer.29) LAN Server Discovery Protocol.
- [Raknet Protocol](https://wiki.vg/Raknet_Protocol) Applicable to modern Bedrock servers.
- [Query Protocol](https://wiki.vg/Query) Applicable to modern Java Edition servers (available from version 1.9pre4 onwards).

### Usage

1. To integrate it as a library into your own Rust project, run it in the root project directory.

```bash
cargo add msp
```

Or, add this dependency to your `Cargo.toml` file:

```toml
[dependencies]
msp = "0.1.2"
```

### Examples

Here are some basic examples showcased below.

1. Use `Conf::get_server_status` to retrieve server information, return `Server`. Note that older versions are not supported:

```rust
use msp::{Conf, MspErr, Server};

fn main() -> Result<(), MspErr> {
    let server = Conf::create_with_port("www.example.com", 25565);
    let info: Server = server.get_server_status()?;

    println!("{}", info);

    Ok(())
}
```

2. Use `Conf::create_with_port` to create a connection configuration specifying the port:

```rust
use msp::{Conf, MspErr, Server};

fn main() -> Result<(), MspErr> {
    let server = Conf::create_with_port("www.example.com", 25565);
    let info: Server = server.get_server_status()?;

    println!("{}", info);

    Ok(())
}
```

3. Use `get_lan_server_status` to retrieve LAN online hosts:

```rust
use msp::{get_lan_server_status, MspErr, SocketConf};
use std::time::Duration;

const SERVER_OFFLINE_TIMEOUT: u64 = 2000;

fn main() -> Result<(), MspErr> {
    let (_ter, receiver) = get_lan_server_status(&SocketConf {
        read_time_out: Some(Duration::from_millis(SERVER_OFFLINE_TIMEOUT)),
        ..Default::default()
    })?;

    loop {
        match receiver.recv() {
            Ok(result) => {
                if let Ok(Some(server)) = result {
                    // ...
                }
            }
            Err(_) => {}
        }
    }

    Ok(())
}
```

4. Use `Conf::query_full` to retrieve server information using the Query protocol:

```rust
use msp::{Conf, MspErr};

fn main() -> Result<(), MspErr> {
    let server = Conf::create_with_port("www.example.com", 25565);

    println!("{}", server.query_full()?);

    Ok(())
}
```

:warning:To use this protocol, you need to enable the `enable-query` option on the server side. This option can be found in the `server.properties` file located in the root directory. Set it as follows:

```toml
enable-query=true
query.port=25565 # Configure the port according to your specific situation
```

Make sure to save the changes and restart the server for the configuration to take effect.

### License

MIT.
