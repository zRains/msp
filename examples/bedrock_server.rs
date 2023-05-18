use msp::{Msp, MspErr};

fn main() -> Result<(), MspErr> {
    let server = Msp::create_with_port("kupars.top", 19132)?;

    println!("{}", server.get_bedrock_server_status()?);

    Ok(())
}
