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

use std::time::SystemTime;
use std::time::Duration;

use std::thread;

use crossterm::terminal::size as term_size;

mod params;
mod string_constants;
mod dirt_display;

// macro_rules! PARAM_FORMAT_STR { () => { "{:<8} : {:<}" }; } 

fn main() {

    let WINDOW_SIZE: usize = 100;
    let TIME_WINDOW_SIZE: usize = 10;
    let args: Vec<String> = env::args().collect();
    let usage = format!("Usage {} IP:PORT", &args[0]);
    
    let _ = stdout().execute(terminal::Clear(terminal::ClearType::FromCursorDown));
    let mut bytes_recieved_in_sec: usize = 0;
    let mut bytes_recieved_in_last_sec: usize = 0;

    let mut packets_recieved: usize = 0;

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
    let mut time_window: VecDeque<u128> = VecDeque::with_capacity(TIME_WINDOW_SIZE);
    for i in 0..TIME_WINDOW_SIZE {
        time_window.push_front(42);
    }

    let mut buf = [0u8; rosc::decoder::MTU];

    let mut dirt_state: DirtState = HashMap::new();

    let start_time = SystemTime::now();
    let mut elapsed_time = SystemTime::now();
    let mut last_elapsed: u128 = 0;
    let mut avg_elapsed: u128 = 0;



    loop {
        match sock.recv_from(&mut buf) {
            Ok((size, addr)) => {

                let (cols,rows) = {
                    if let Ok((cols, rows)) = term_size() {
                        (cols as usize,rows as usize)
                    } else {
                        (1,1)
                    }
                };

                // dirt_display::display_text(&(("/".repeat(cols) + "\n")).repeat(rows));
                dirt_display::display_text(&format!("{:?}",dirt_state));
                thread::sleep(Duration::from_nanos(1000000));

                bytes_recieved_in_sec = bytes_recieved_in_sec + size;
                // println!("nanos between msgs: {} total from {}", last_elapsed, addr);
                println!("avg msgs per sec: {} total from {}", (1_000_000_000f32 / avg_elapsed as f32), addr);
                let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                handle_packet(packet, &mut dirt_state, &mut msg_window);

                match elapsed_time.elapsed() {
                    Ok(elapsed) => {
                        elapsed_time = SystemTime::now();
                        last_elapsed = elapsed.as_nanos();

                        avg_elapsed = (time_window.iter().sum::<u128>()) / TIME_WINDOW_SIZE as u128;
                        time_window.push_front(last_elapsed);
                        time_window.pop_back();
                    }
                    Err(e) => {
                        println!("couldn't get system time ?????????: {}", e);
                    }
                }



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




