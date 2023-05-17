use msp::{Msp, MspErr};

fn main() -> Result<(), MspErr> {
    // let server = Msp::create_with_port("buzz.dogecraft.net", 25565)?;
    // let server = Msp::create_with_port("grmpixelmon.com", 25565)?;
    // let server = Msp::create_with_port("minecraftonline.com", 25565)?;
    let server = Msp::create_with_port("mc.safesurvival.net", 25565)?;
    // let server = Msp::create_with_port("192.168.1.219", 25565)?;

    // println!("{}", server.get_server_status()?);
    println!("{}", server.query_full()?);
    // println!("{}", server.get_server_status()?);
    Ok(()) // But i am not Ok...
}
