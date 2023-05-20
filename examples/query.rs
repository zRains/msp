use msp::{Conf, MspErr};

fn main() -> Result<(), MspErr> {
    let server = Conf::create_with_port("www.example.com", 25565);

    println!("{}", server.query_full()?);

    Ok(())
}
