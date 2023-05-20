use msp::{Conf, MspErr, SocketConf};

fn main() -> Result<(), MspErr> {
    let conf = Conf::create("www.example.com");

    assert_eq!(conf.host, "www.example.com");
    assert_eq!(conf.port, 25565);
    assert_eq!(conf.socket_conf, SocketConf::default());

    let conf = Conf::create_with_port("www.example.com", 19132);
    assert_eq!(conf.port, 19132);

    let conf = Conf::create_from_str("192.168.1.10:25565")?;

    assert_eq!(conf.host, "192.168.1.10");
    assert_eq!(conf.port, 25565);

    let conf = Conf::create_from_str("www.example.com:25565")?;

    assert_eq!(conf.host, "www.example.com");
    assert_eq!(conf.port, 25565);
    Ok(())
}
