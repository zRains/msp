use msp::{Conf, MspErr};

fn main() -> Result<(), MspErr> {
    let server = Conf::create_with_port("www.example.com", 19132);
    let info = server.get_bedrock_server_status()?;

    println!("{}", info);

    Ok(())
}
