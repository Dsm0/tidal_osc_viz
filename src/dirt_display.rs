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

use std::cmp::{PartialEq, Eq};

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
    only_changed: bool,
) {
    let mut full_str = String::new();

    let cols = {
        if let Ok((cols, _rows)) = size() {
            cmp::max((cols as i32) - RIGHT_SPACE, 1_i32) as usize
        } else {
            1
        }
    };

    if let Some(msg) = dirt_window.front() {
        if let Some(config) = param_configs.get("cycle") {
            match config.style {
                crate::DisplayStyle::Cycle => {
                    full_str.push_str(msg.display_f32("cycle", |f| display_cycle(f, cols)).as_str());
                }
                _ => {
                    full_str.push_str(msg.display_f32("cycle", |f| format!("cycle: {}\n", f)).as_str());
                }
            }
        } else {
            full_str.push_str(msg.display_f32("cycle", |f| display_cycle(f, cols)).as_str());
        }

        for (id, current_msg_state) in dirt_state {
            if id == "tick" { continue; }
            // Find previous message for this id in dirt_window (skip the most recent)
            let prev_msg = if only_changed {
                dirt_window.iter().skip(1).find(|m| {
                    if let Some(DirtValue::DS(prev_id)) = m.get("_id_") {
                        prev_id == id
                    } else {
                        false
                    }
                })
            } else {
                None
            };
            let huh = display_dirt_message(current_msg_state, prev_msg, cols, param_configs, id, only_changed);
            full_str.push_str(huh.as_str());
        }

        full_str.push_str(msg.display_raw().as_str());
    } else {
        full_str.push_str("Some(msg) = dirt_window.front() failed???")
    }

    display_text(&full_str);
}

fn display_dirt_message(
    msg: &DirtMessage,
    prev_msg: Option<&DirtMessage>,
    cols: usize,
    param_configs: &HashMap<String, crate::ParamDisplayConfig>,
    msg_id: &String,
    only_changed: bool,
) -> String {
    let display_str: &mut String = &mut String::new();
    display_str.push_str(&format!("{:<15}{} id\n", "", msg_id));
    let mut sorted_params: Vec<_> = msg.keys().collect();
    sorted_params.sort();
    for param_name_str in sorted_params {
        let param_name = param_name_str.as_str();
        if param_name == "_id_" || param_name == "cycle" {
            continue;
        }
        if only_changed {
            if let Some(prev) = prev_msg {
                if let Some(prev_val) = prev.get(param_name) {
                    if let Some(cur_val) = msg.get(param_name) {
                        if prev_val == cur_val {
                            continue; // skip unchanged
                        }
                    }
                }
            }
        }
        if let Some(config) = param_configs.get(param_name) {
            match config.value_type {
                crate::DisplayValueType::Float => {
                    display_str.push_str(
                        msg.display_f32(param_name, |val| match &config.style {
                            crate::DisplayStyle::BarFloat { min, max } => {
                                format!("{} {}\n", display_bar_float(val, *min, *max, cols), config.label)
                            }
                            crate::DisplayStyle::CustomFloat => {
                                format!("{} {}\n", display_float(val, cols), config.label)
                            }
                            crate::DisplayStyle::Binary16 => {
                                format!("{} {}\n", display_bin_float(val, cols), config.label)
                            }
                            crate::DisplayStyle::Raw | crate::DisplayStyle::Cycle => {
                                format!("{}: {} {}\n", param_name, val, config.label)
                            }
                            _ => format!("{}: {} (unsupported style for f32)\n", param_name, val),
                        }).as_str(),
                    );
                }
                crate::DisplayValueType::Integer => {
                    display_str.push_str(
                        msg.display_i32(param_name, |val| match &config.style {
                            crate::DisplayStyle::BarInt { min, max } => {
                                format!("{} {}\n", display_bar_int(val, *min, *max, cols), config.label)
                            }
                            crate::DisplayStyle::Raw => {
                                format!("{}: {} {}\n", param_name, val, config.label)
                            }
                            _ => format!("{}: {} (unsupported style for i32)\n", param_name, val),
                        }).as_str(),
                    );
                }
                crate::DisplayValueType::String => {
                    display_str.push_str(
                        msg.display_string(param_name, |val| match &config.style {
                            crate::DisplayStyle::Raw => {
                                format!("{}: {} {}\n", param_name, val, config.label)
                            }
                            _ => format!("{}: {} (unsupported style for string)\n", param_name, val),
                        }).as_str(),
                    );
                }
            }
        } else {
            match msg.get(param_name) {
                Some(DirtValue::DF(f)) => display_str.push_str(&format!("{}: {}\n", param_name, f)),
                Some(DirtValue::DI(i)) => display_str.push_str(&format!("{}: {}\n", param_name, i)),
                Some(DirtValue::DS(s)) => display_str.push_str(&format!("{}: {}\n", param_name, s)),
                None => {}
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

// Implement PartialEq for DirtValue to allow comparison
impl PartialEq for DirtValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DirtValue::DI(a), DirtValue::DI(b)) => a == b,
            (DirtValue::DF(a), DirtValue::DF(b)) => a == b,
            (DirtValue::DS(a), DirtValue::DS(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for DirtValue {}