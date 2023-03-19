extern crate rosc;

use crossterm::{terminal, ExecutableCommand};
use rosc::OscPacket;
// use rosc::OscType;

use std::env;
use std::io::stdout;
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;

mod params;
mod string_constants;
mod dirt_display;

// macro_rules! PARAM_FORMAT_STR { () => { "{:<8} : {:<}" }; } 

fn main() {
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

    let mut buf = [0u8; rosc::decoder::MTU];

    loop {
        match sock.recv_from(&mut buf) {
            Ok((size, _addr)) => {
                bytes_recieved = bytes_recieved + size;
                println!("bytes recieved: {} total from {}", bytes_recieved, _addr);
                let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                handle_packet(packet);
            }
            Err(e) => {
                println!("Error receiving from socket: {}", e);
                break;
            }
        }
    }
}

fn handle_packet(packet: OscPacket) {
    match packet {
        OscPacket::Message(msg) => {
            // println!("OSC address: {}", msg.addr);
            // println!("OSC arguments: {:?}", msg.args);

            let dirt_message: params::DirtMessage = params::to_dirt_message(msg.args);

            dirt_display::display_dirt_message(&dirt_message);
        }
        OscPacket::Bundle(bundle) => {
            // println!("OSC Bundle: {:?}", bundle);
        }
    }
}




