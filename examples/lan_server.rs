use msp::{get_lan_server_status, LanServer, MspErr, SocketConf};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

const STORE_CHECK_INTERVAL: u64 = 4000;
const MAXIMUM_SERVERS: usize = 100;
const SERVER_OFFLINE_TIMEOUT: u64 = 2000;

fn main() -> Result<(), MspErr> {
    let mut lan_server_map = HashMap::<LanServer, SystemTime>::new();
    let mut outer_loop_time_starter = SystemTime::now();

    let (ter, receiver) = get_lan_server_status(&SocketConf {
        read_time_out: Some(Duration::from_millis(SERVER_OFFLINE_TIMEOUT)),
        ..Default::default()
    })?;

    loop {
        println!("map: {:?}", lan_server_map);

        let inner_loop_time_starter = SystemTime::now();

        if inner_loop_time_starter
            .duration_since(outer_loop_time_starter)?
            .as_millis() as u64
            > STORE_CHECK_INTERVAL
        {
            lan_server_map.retain(|_, &mut time| {
                match inner_loop_time_starter.duration_since(time) {
                    Ok(t) => t.as_millis() as u64 <= SERVER_OFFLINE_TIMEOUT,
                    _ => false,
                }
            });

            outer_loop_time_starter = inner_loop_time_starter;
        }

        if lan_server_map.len() == MAXIMUM_SERVERS {
            ter();

            break;
        }

        match receiver.recv() {
            Ok(result) => {
                if let Ok(Some(server)) = result {
                    lan_server_map.insert(server, SystemTime::now());
                }
            }
            Err(_) => {}
        }
    }

    Ok(())
}
