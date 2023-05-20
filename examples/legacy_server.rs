use msp::{Conf, MspErr};

fn main() -> Result<(), MspErr> {
    // let server = Msp::create_with_port("play.elysion.network", 25565)?;
    let server = Conf::create_with_port("bteam.mineyourmind.net", 25565);

    println!("{}", server.get_netty_server_status()?);

    Ok(())
}
