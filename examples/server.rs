use msp::{Msp, MspErr};

fn main() -> Result<(), MspErr> {
    let server = Msp::create_with_port("grmpixelmon.com", 25565)?;

    println!("{}", server.get_server_status()?);

    Ok(())
}
