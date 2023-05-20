use msp::{get_lan_server_status, MspErr, SocketConf};

fn main() -> Result<(), MspErr> {
    get_lan_server_status(&SocketConf::default())?;

    Ok(())
}
