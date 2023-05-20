use msp::{Conf, MspErr, Server};

fn main() -> Result<(), MspErr> {
    let server = Conf::create_with_port("www.example.com", 25565)?;
    let info: Server = server.get_server_status()?;

    println!("{}", info);

    Ok(())
}
