use msp::{Conf, MspErr};

fn main() -> Result<(), MspErr> {
    let server = Conf::create_with_port("kupars.top", 19132)?;

    println!("{}", server.get_bedrock_server_status()?);

    Ok(())
}
