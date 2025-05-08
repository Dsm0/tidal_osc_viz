// use crate::params::DirtParam;
// use crate::display_funcs::display_param_float;

// use rosc::OscType;

use crate::params::DirtMessage;
use crate::params::DirtState;
use crate::params::DirtValue;
use crate::params::DirtWindow;
use crate::params::GetDirtValue;
// use crate::params::DirtDisplayMap;

use crossterm::{cursor, terminal, ExecutableCommand};
use std::io::{stdout, Write};

use crossterm::terminal::size;

use std::cmp;

use crate::string_constants::BAR_CHARS;
use crate::string_constants::BOX;

use std::collections::HashMap;

static RIGHT_SPACE: i32 = 25;

// NOTE: will probably replace when I get to using a tui library
pub fn display_text(msg: &String) {
    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();
    let _ = stdout.execute(terminal::Clear(terminal::ClearType::FromCursorDown));
    let _ = writeln!(stdout, "{}", msg);
    let _ = stdout.execute(cursor::MoveTo(0, 0));
    let _ = stdout.execute(cursor::Show).unwrap();
}

fn float_mod(f: f32, m: f32) -> f32 {
    ((f % m) + m) % m
}

pub fn display_dirt(
    dirt_state: &DirtState,
    dirt_window: &DirtWindow,
    param_configs: &HashMap<String, crate::ParamDisplayConfig>,
) {
    let mut full_str = String::new();

    let cols = {
        if let Ok((cols, _rows)) = size() {
            cmp::max((cols as i32) - RIGHT_SPACE, 1_i32) as usize
        } else {
            1
        }
    };

    // TODO: sort keys first
    if let Some(msg) = dirt_window.front() {
        // Handle "cycle" specifically or make it configurable too
        // For now, let's assume "cycle" might have its own config or a default handling
        if let Some(config) = param_configs.get("cycle") {
            match config.style {
                crate::DisplayStyle::Cycle => {
                    full_str.push_str(msg.display_f32("cycle", |f| display_cycle(f, cols)).as_str());
                }
                _ => { // Default for cycle if not 'Cycle' style
                    full_str.push_str(msg.display_f32("cycle", |f| format!("cycle: {}
", f)).as_str());
                }
            }
        } else { // Default if "cycle" is not configured
            full_str.push_str(msg.display_f32("cycle", |f| display_cycle(f, cols)).as_str());
        }

        for (id, current_msg_state) in dirt_state {
            // Skip "tick" or other meta messages if necessary, or make them configurable
             if id == "tick" { continue; }
            // display_dirt_message now needs param_configs
            let huh = display_dirt_message(current_msg_state, cols, param_configs, id);
            full_str.push_str(huh.as_str());
        }

        full_str.push_str(msg.display_raw().as_str()); // Keep raw display at the end for now
    } else {
        full_str.push_str("Some(msg) = dirt_window.front() failed???")
    }

    display_text(&full_str);
}

fn display_dirt_message(
    msg: &DirtMessage,
    cols: usize,
    param_configs: &HashMap<String, crate::ParamDisplayConfig>,
    msg_id: &String
) -> String {
    let display_str: &mut String = &mut String::new();

    // Display message ID (e.g., the 'sound source' like 's1', 's2')
    // This could also be made part of the configurable display if needed
    display_str.push_str(&format!("{:<15}{} id
", "", msg_id));

    // Iterate over parameters in the message, or iterate over configured params?
    // Iterating over message params ensures we see everything, then apply config or default.
    // For a defined order, one might iterate over a sorted list of configured keys
    // that are also present in the message. For now, iterate msg keys.
    let mut sorted_params: Vec<_> = msg.keys().collect();
    sorted_params.sort(); // Sort for consistent display order

    for param_name_str in sorted_params {
        let param_name = param_name_str.as_str();

        if param_name == "_id_" || param_name == "cycle" { // Already handled or not for individual display here
            continue;
        }

        if let Some(config) = param_configs.get(param_name) {
            // Parameter has a specific configuration
            match config.value_type {
                crate::DisplayValueType::Float => {
                    display_str.push_str(
                        msg.display_f32(param_name, |val| match &config.style {
                            crate::DisplayStyle::BarFloat { min, max } => {
                                format!("{} {}
", display_bar_float(val, *min, *max, cols), config.label)
                            }
                            crate::DisplayStyle::CustomFloat => {
                                format!("{} {}
", display_float(val, cols), config.label)
                            }
                             crate::DisplayStyle::Binary16 => { // Assuming Binary16 means f32 to i16 then binary
                                format!("{} {}
", display_bin_float(val, cols), config.label)
                            }
                            crate::DisplayStyle::Raw | crate::DisplayStyle::Cycle => { // Cycle unlikely here but for completeness
                                format!("{}: {} {}
", param_name, val, config.label)
                            }
                            _ => format!("{}: {} (unsupported style for f32)
", param_name, val), // Fallback for mismatched style
                        }).as_str(),
                    );
                }
                crate::DisplayValueType::Integer => {
                    display_str.push_str(
                        msg.display_i32(param_name, |val| match &config.style {
                            crate::DisplayStyle::BarInt { min, max } => {
                                format!("{} {}
", display_bar_int(val, *min, *max, cols), config.label)
                            }
                            crate::DisplayStyle::Raw => {
                                format!("{}: {} {}
", param_name, val, config.label)
                            }
                            _ => format!("{}: {} (unsupported style for i32)
", param_name, val), // Fallback
                        }).as_str(),
                    );
                }
                crate::DisplayValueType::String => {
                    display_str.push_str(
                        msg.display_string(param_name, |val| match &config.style {
                            crate::DisplayStyle::Raw => {
                                format!("{}: {} {}
", param_name, val, config.label)
                            }
                            _ => format!("{}: {} (unsupported style for string)
", param_name, val), // Fallback
                        }).as_str(),
                    );
                }
            }
        } else {
            // Default display for unconfigured parameters: raw value
            match msg.get(param_name) {
                Some(DirtValue::DF(f)) => display_str.push_str(&format!("{}: {}
", param_name, f)),
                Some(DirtValue::DI(i)) => display_str.push_str(&format!("{}: {}
", param_name, i)),
                Some(DirtValue::DS(s)) => display_str.push_str(&format!("{}: {}
", param_name, s)),
                None => {} // Should not happen if iterating keys from msg
            }
        }
    }

    display_str.to_string()
}

fn remap_range(s: f32, l1: f32, h1: f32, l2: f32, h2: f32) -> f32 {
    l2 + (s - l1) * (h2 - l2) / (h1 - l1)
}

fn get_box_string(val: usize) -> String {
    if val == 0 {
        return "".to_string();
    }

    let (val_div, val_mod) = (val / BAR_CHARS.len(), val % BAR_CHARS.len());

    if BAR_CHARS.len() == 0 { // Prevent division by zero if BAR_CHARS is empty
        return BOX.repeat(val_div);
    }
    if val_mod >= BAR_CHARS.len() { // Prevent out of bounds access
        return BOX.repeat(val_div);
    }

    BOX.repeat(val_div) + BAR_CHARS[val_mod as usize]
}

fn display_cycle(f: &f32, cols: usize) -> String {
    let bar = display_bar_float(&(f - f.floor()), 0.0, 1.0, cols);
    let cycle_mods = format!(
        "{}/8 {}/16 {}/24 {}/32 {}/40 {}/48 {}/56 {}/64",
        float_mod(*f, 8.0).floor() + 1.0,
        float_mod(*f, 2.0 * 8.0).floor() + 1.0,
        float_mod(*f, 3.0 * 8.0).floor() + 1.0,
        float_mod(*f, 4.0 * 8.0).floor() + 1.0,
        float_mod(*f, 5.0 * 8.0).floor() + 1.0,
        float_mod(*f, 6.0 * 8.0).floor() + 1.0,
        float_mod(*f, 7.0 * 8.0).floor() + 1.0,
        float_mod(*f, 8.0 * 8.0).floor() + 1.0
    );
    format!("{}\n {}\n", bar, cycle_mods)
}

fn display_cycle_bin(f: &f32, cols: usize) -> String {
    let bar = display_bar_float(&(f - f.floor()), 0.0, 1.0, cols);
    let cycle_mods = format!("{:032b}",*f as usize);
    format!("{}\n    {}\n", bar, cycle_mods)
}

pub fn display_bar_float(f: &f32, min: f32, max: f32, cols: usize) -> String {
    let val: f32 = remap_range(*f, min, max, 0.0, (8 * cols) as f32);

    let bar_string_index: usize = val.round() as usize;

    let bar = get_box_string(bar_string_index);
    format!("{:>3}:{:0width$}:{:<4}", min, bar, max, width = cols)
}

fn display_bar_int(i: &i32, min: i32, max: i32, cols: usize) -> String {
    let mut temp_str = String::new();
    
    let mut trueMax = max;
    if *i > max {
        trueMax = *i + 1;
    }

    for j in min..(trueMax){
        if *i == j {
            temp_str.push_str(format!(" [{}] ", j).as_str())
        } else {
            temp_str.push_str(format!(" {} ", j).as_str())
        }
    }

    format!("{:width$} ", temp_str, width = ((cols) as usize))
}

fn display_bin_float(f: &f32, _cols: usize) -> String { // cols might not be used for fixed binary
    format!("{:016b}", *f as i16) // Padded with zeros to 16 bits
}

fn display_float(f: &f32, cols: usize) -> String {
    format!("{:16}", *f as i16)
}