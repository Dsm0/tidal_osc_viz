extern crate rosc;

use crossterm::{terminal, ExecutableCommand};
use params::{DirtMessage, DirtWindow};
use rosc::{OscPacket};
// use rosc::OscType;

// use crate::params::DirtMessage;
use crate::params::DirtState;

use std::collections::{HashMap, VecDeque};
use std::env;
use std::io::stdout;
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;

mod params;
mod string_constants;
mod dirt_display;

// macro_rules! PARAM_FORMAT_STR { () => { "{:<8} : {:<}" }; } 

fn main() {

    let WINDOW_SIZE: usize = 10;
    let args: Vec<String> = env::args().collect();
    let usage = format!("Usage {} IP:PORT", &args[0]);
    
    let _ = stdout().execute(terminal::Clear(terminal::ClearType::FromCursorDown));
    let mut bytes_recieved: usize = 0;

    if args.len() < 2 {
        println!("{}", usage);
        ::std::process::exit(1)
    }
    let addr = match SocketAddrV4::from_str(&args[1]) {
        Ok(addr) => addr,
        Err(_) => panic!("{}", usage),
    };
    let sock = UdpSocket::bind(addr).unwrap();
    // println!("Listening to {}", addr);

    let mut msg_window: DirtWindow = params::new_dirt_window(WINDOW_SIZE);

    let mut buf = [0u8; rosc::decoder::MTU];

    let mut dirt_state: DirtState = HashMap::new();


    loop {
        match sock.recv_from(&mut buf) {
            Ok((size, addr)) => {
                bytes_recieved = bytes_recieved + size;
                println!("bytes recieved: {} total from {}", bytes_recieved, addr);
                let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                handle_packet(packet, &mut dirt_state, &mut msg_window);
            }
            Err(e) => {
                println!("Error receiving from socket: {}", e);
                break;
            }
        }
    }

}

fn handle_packet(packet: OscPacket, dirt_state: &mut DirtState, msg_window: &mut VecDeque<DirtMessage>) {
    match packet {
        OscPacket::Message(msg) => {

            params::update_dirt_state(dirt_state,msg.args, msg_window);

            dirt_display::display_dirt(dirt_state, msg_window);

        }
        OscPacket::Bundle(_bundle) => {
            // println!("OSC Bundle: {:?}", bundle);
        }
    }
}




