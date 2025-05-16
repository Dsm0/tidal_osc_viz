extern crate rosc;

use crossterm::{terminal, ExecutableCommand};
use params::{DirtMessage, DirtWindow};
use rosc::{OscPacket};
// use rosc::OscType;

// use crate::params::DirtMessage;
use crate::params::DirtState;
use clap::Parser;

use std::collections::{HashMap, VecDeque};
use std::env;
use std::io::{stdout, Write};
use std::net::{SocketAddrV4, UdpSocket, SocketAddr};
use std::str::FromStr;

use std::time::SystemTime;
use std::time::Duration;

use std::thread;
use std::sync::{Arc, Mutex};

use crossterm::terminal::size as term_size;
use crossterm::cursor;

mod params;
mod string_constants;
mod dirt_display;

#[derive(Debug, Clone)]
pub enum DisplayValueType {
    Float,
    Integer,
    String,
}

#[derive(Debug, Clone)]
pub enum DisplayStyle {
    Raw,
    BarFloat { min: f32, max: f32 },
    BarInt { min: i32, max: i32 },
    Binary16,
    CustomFloat,
    Cycle,
}

#[derive(Debug, Clone)]
pub struct ParamDisplayConfig {
    pub value_type: DisplayValueType,
    pub style: DisplayStyle,
    pub label: String, // e.g., " gain", " s"
}

fn parse_param_display_arg(arg_val: &str) -> Result<(String, ParamDisplayConfig), String> {
    let parts: Vec<&str> = arg_val.split(':').collect();
    if parts.len() < 3 || parts.len() > 5 {
        return Err(format!(
            "Invalid format for --param-display: {}. Expected <param_name>:<type>:<style>[:<min>:<max>]",
            arg_val
        ));
    }

    let param_name = parts[0].to_string();
    let label = format!(" {}", param_name); // Default label, can be refined

    let value_type = match parts[1] {
        "f32" => DisplayValueType::Float,
        "i32" => DisplayValueType::Integer,
        "string" => DisplayValueType::String,
        _ => return Err(format!("Invalid type '{}' for parameter {}", parts[1], param_name)),
    };

    let style = match parts[2] {
        "raw" => DisplayStyle::Raw,
        "bar_float" => {
            if parts.len() != 5 {
                return Err(format!(
                    "Style 'bar_float' requires min and max values (e.g., 0.0:2.0) for {}",
                    param_name
                ));
            }
            let min = parts[3].parse::<f32>().map_err(|e| {
                format!("Invalid min value '{}' for {}: {}", parts[3], param_name, e)
            })?;
            let max = parts[4].parse::<f32>().map_err(|e| {
                format!("Invalid max value '{}' for {}: {}", parts[4], param_name, e)
            })?;
            DisplayStyle::BarFloat { min, max }
        }
        "bar_int" => {
            if parts.len() != 5 {
                return Err(format!(
                    "Style 'bar_int' requires min and max values (e.g., 0:10) for {}",
                    param_name
                ));
            }
            let min = parts[3].parse::<i32>().map_err(|e| {
                format!("Invalid min value '{}' for {}: {}", parts[3], param_name, e)
            })?;
            let max = parts[4].parse::<i32>().map_err(|e| {
                format!("Invalid max value '{}' for {}: {}", parts[4], param_name, e)
            })?;
            DisplayStyle::BarInt { min, max }
        }
        "binary16" => DisplayStyle::Binary16,
        "custom_float" => DisplayStyle::CustomFloat,
        "cycle" => DisplayStyle::Cycle,
        _ => return Err(format!("Invalid style '{}' for parameter {}", parts[2], param_name)),
    };

    Ok((
        param_name.clone(),
        ParamDisplayConfig { value_type, style, label },
    ))
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_parser = clap::value_parser!(SocketAddrV4))]
    listen_addr: SocketAddrV4,

    #[arg(long, value_name = "CONFIG_STRING", action = clap::ArgAction::Append)]
    param_display: Vec<String>,

    /// Only display parameters for a given id that have changed since the previous message with that id
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Only display parameters for a given id that have changed since the previous message with that id")]
    only_changed: bool,

    /// Only display parameters from the last message received (single id mode)
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Only display parameters from the last message received (single id mode)")]
    single_id: bool,

    /// Flash raw data to the screen (debug)
    #[arg(long, action = clap::ArgAction::SetTrue, help = "Flash raw data to the screen (debug)")]
    flash_data: bool,

    /// Maximum number of ids to display concurrently
    #[arg(long, value_name = "MAX_IDS", default_value_t = 1, help = "Maximum number of ids to display concurrently")]
    id_display_max: usize,

    /// Display parameters with undefined ranges (not bar_float/bar_int) as raw. If false, these are not displayed.
    #[arg(long, action = clap::ArgAction::Set, default_value_t = false, help = "Display parameters with undefined ranges (not bar_float/bar_int) as raw. If false, these are not displayed.")]
    display_unknown: bool,
    
    /// Prevent text from overflowing past the height of the terminal
    #[arg(long, action = clap::ArgAction::Set, default_value_t = false, help = "Prevent text from overflowing past the height of the terminal")]
    prevent_overflow: bool,
}

// macro_rules! PARAM_FORMAT_STR { () => { "{:<8} : {:<}" }; } 

// This function will display the statistics line at the bottom without clearing the screen
fn display_stats(avg_msgs_per_sec: f32, addr: &SocketAddrV4) {
    let (cols, rows) = {
        if let Ok((cols, rows)) = term_size() {
            (cols as usize, rows as usize)
        } else {
            (1, 1)
        }
    };
    
    // Save cursor position
    let mut stdout = stdout();
    stdout.execute(cursor::SavePosition).unwrap();
    
    // Move to the last row
    stdout.execute(cursor::MoveTo(0, (rows - 1) as u16)).unwrap();
    
    // Clear the line and write the stats
    stdout.execute(terminal::Clear(terminal::ClearType::CurrentLine)).unwrap();
    let stats_msg = format!("avg msgs per sec: {} from {}", avg_msgs_per_sec, addr);
    write!(stdout, "{}", stats_msg).unwrap();
    
    // Restore cursor position
    stdout.execute(cursor::RestorePosition).unwrap();
    stdout.flush().unwrap();
}

fn main() {
    let cli = Cli::parse();

    let mut param_configs: Vec<(String, ParamDisplayConfig)> = Vec::new();
    for config_str in cli.param_display {
        match parse_param_display_arg(&config_str) {
            Ok((name, config)) => {
                param_configs.push((name, config));
            }
            Err(e) => {
                eprintln!("Error parsing --param-display argument: {}", e);
                std::process::exit(1);
            }
        }
    }

    let WINDOW_SIZE: usize = 64;
    let TIME_WINDOW_SIZE: usize = 10;
    let args: Vec<String> = env::args().collect();
    let usage = format!("Usage {} IP:PORT", &args[0]);
    
    let _ = stdout().execute(terminal::Clear(terminal::ClearType::FromCursorDown));
    let mut bytes_recieved_in_sec: usize = 0;
    let mut bytes_recieved_in_last_sec: usize = 0;

    let mut packets_recieved: usize = 0;

    let addr = cli.listen_addr;
    let sock = UdpSocket::bind(addr).unwrap();
    // println!("Listening to {}", addr);

    let msg_window: Arc<Mutex<DirtWindow>> = Arc::new(Mutex::new(params::new_dirt_window(WINDOW_SIZE)));
    let dirt_state: Arc<Mutex<DirtState>> = Arc::new(Mutex::new(HashMap::new()));
    
    let param_configs_arc = Arc::new(param_configs);
    let cli_only_changed = cli.only_changed;
    let cli_single_id = cli.single_id;
    let cli_display_unknown = cli.display_unknown;
    let cli_prevent_overflow = cli.prevent_overflow;
    
    // Create a thread to update the display and handle the flashing braces
    let msg_window_clone = Arc::clone(&msg_window);
    let dirt_state_clone = Arc::clone(&dirt_state);
    let param_configs_clone = Arc::clone(&param_configs_arc);
    let addr_clone = addr.clone();
    let avg_elapsed_arc = Arc::new(Mutex::new(0_u128));
    let avg_elapsed_clone = Arc::clone(&avg_elapsed_arc);
    
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(10)); // Update every 10 ms
            
            let mut window_lock = msg_window_clone.lock().unwrap();
            let mut updated = false;
            
            // Update should_show_braces flag for messages older than 200ms
            let now = SystemTime::now();
            for (_, info) in window_lock.iter_mut() {
                if info.should_show_braces {
                    if let Ok(elapsed) = now.duration_since(info.timestamp) {
                        if elapsed > Duration::from_millis(200) {
                            info.should_show_braces = false;
                            updated = true;
                        }
                    }
                }
            }
            
            // Only redraw the visualization if we changed something
            if updated {
                let state_lock = dirt_state_clone.lock().unwrap();
                dirt_display::display_dirt(&state_lock, &window_lock, &param_configs_clone, 
                    cli_only_changed, cli_single_id, cli_display_unknown, cli_prevent_overflow);
                
                // Always update the stats line after redrawing
                let avg_elapsed = *avg_elapsed_clone.lock().unwrap();
                if avg_elapsed > 0 {
                    display_stats(1_000_000_000f32 / avg_elapsed as f32, &addr_clone);
                }
            }
        }
    });

    let mut time_window: VecDeque<u128> = VecDeque::with_capacity(TIME_WINDOW_SIZE);
    for i in 0..TIME_WINDOW_SIZE {
        time_window.push_front(42);
    }

    let mut buf = [0u8; rosc::decoder::MTU];

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

                if cli.flash_data {
                    let state_lock = dirt_state.lock().unwrap();
                    dirt_display::display_text(&format!("{:?}", *state_lock));
                }
                thread::sleep(Duration::from_nanos(1000000));

                bytes_recieved_in_sec = bytes_recieved_in_sec + size;
                
                let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                
                // Process the packet with the shared data structures
                let mut window_lock = msg_window.lock().unwrap();
                let mut state_lock = dirt_state.lock().unwrap();
                handle_packet(packet, &mut state_lock, &mut window_lock, &param_configs_arc, 
                    cli_only_changed, cli_single_id, cli_display_unknown, cli_prevent_overflow);

                match elapsed_time.elapsed() {
                    Ok(elapsed) => {
                        elapsed_time = SystemTime::now();
                        last_elapsed = elapsed.as_nanos();

                        avg_elapsed = (time_window.iter().sum::<u128>()) / TIME_WINDOW_SIZE as u128;
                        *avg_elapsed_arc.lock().unwrap() = avg_elapsed;
                        time_window.push_front(last_elapsed);
                        time_window.pop_back();
                        
                        // Display stats with our new function
                        if let SocketAddr::V4(addr_v4) = addr {
                            display_stats(1_000_000_000f32 / avg_elapsed as f32, &addr_v4);
                        }
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

fn handle_packet(packet: OscPacket, dirt_state: &mut DirtState, msg_window: &mut VecDeque<params::DirtTimestampedMessage>, param_configs: &Vec<(String, ParamDisplayConfig)>, only_changed: bool, single_id: bool, display_unknown: bool, prevent_overflow: bool) {
    match packet {
        OscPacket::Message(msg) => {
            let packet_args = msg.args;
            params::update_dirt_state(dirt_state, packet_args, msg_window);

            dirt_display::display_dirt(dirt_state, msg_window, param_configs, only_changed, single_id, display_unknown, prevent_overflow);

        }
        OscPacket::Bundle(_bundle) => {
            // println!("OSC Bundle: {:?}", bundle);
        }
    }
}




