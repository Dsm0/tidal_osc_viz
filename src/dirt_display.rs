// use crate::params::DirtParam;
// use crate::display_funcs::display_param_float;

// use rosc::OscType;

use crate::params::DirtMessage;
use crate::params::DirtState;
use crate::params::DirtValue;
use crate::params::DirtWindow;
use crate::params::GetDirtValue;
use crate::params::MessageInfo;
// use crate::params::DirtDisplayMap;

use crossterm::{cursor, terminal, ExecutableCommand};
use std::io::{stdout, Write};

use crossterm::terminal::size;

use std::cmp;

use crate::string_constants::BAR_CHARS;
use crate::string_constants::BOX;

use std::collections::HashMap;

use std::cmp::{PartialEq, Eq};

use std::time::{SystemTime, Duration};

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
    param_configs: &Vec<(String, crate::ParamDisplayConfig)>,
    only_changed: bool,
    single_id: bool,
    display_unknown: bool,
) {
    let mut full_str = String::new();

    let cols = {
        if let Ok((cols, _rows)) = size() {
            cmp::max((cols as i32) - RIGHT_SPACE, 1_i32) as usize
        } else {
            1
        }
    };

    if let Some((msg, info)) = dirt_window.front() {
        if let Some(config) = param_configs.iter().find(|(name, _)| name == "cycle") {
            match config.1.style {
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

        // Display ids '1' through '9' across the top, with the most recent id(s) in braces if should_show_braces is true
        let mut id_line = String::new();
        // Find all ids in the window with should_show_braces set to true
        let mut ids_with_braces = vec![];
        
        for (m, info) in dirt_window.iter() {
            if !info.should_show_braces {
                continue;
            }
            
            if let Some(DirtValue::DS(id)) = m.get("_id_") {
                if !ids_with_braces.contains(id) {
                    ids_with_braces.push(id.clone());
                }
            }
        }
        
        for n in 1..=9 {
            let n_str = n.to_string();
            if ids_with_braces.contains(&n_str) {
                id_line.push_str(&format!("{{{}}} ", n));
            } else {
                id_line.push_str(&format!(" {}  ", n));
            }
        }
        full_str.push_str(&format!("\n{}\n\n", id_line));

        if single_id {
            // Only display the most recent message (msg)
            let msg_id_owned = msg.get("_id_").and_then(|v| if let DirtValue::DS(s) = v { Some(s.clone()) } else { None }).unwrap_or_else(|| "?".to_string());
            let huh = display_dirt_message(msg, None, cols, param_configs, &msg_id_owned, only_changed, display_unknown);
            full_str.push_str(huh.as_str());
        } else {
            for (id, current_msg_state) in dirt_state {
                if id == "tick" { continue; }
                let prev_msg = if only_changed {
                    dirt_window.iter().skip(1).find(|(m, _)| {
                        if let Some(DirtValue::DS(prev_id)) = m.get("_id_") {
                            prev_id == id
                        } else {
                            false
                        }
                    }).map(|(m,_)| m)
                } else {
                    None
                };
                let huh = display_dirt_message(current_msg_state, prev_msg, cols, param_configs, id, only_changed, display_unknown);
                full_str.push_str(huh.as_str());
            }
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
    param_configs: &Vec<(String, crate::ParamDisplayConfig)>,
    msg_id: &String,
    only_changed: bool,
    display_unknown: bool,
) -> String {
    let display_str: &mut String = &mut String::new();
    let mut already_displayed = std::collections::HashSet::new();
    // First, display parameters in the order of param_configs
    for (param_name, config) in param_configs.iter() {
        let param_name = param_name.as_str();
        if param_name == "_id_" || param_name == "cycle" {
            continue;
        }
        if !msg.contains_key(param_name) {
            continue;
        }
        already_displayed.insert(param_name.to_string());
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
        let config = config;
        match config.value_type {
            crate::DisplayValueType::Float => {
                let show = match &config.style {
                    crate::DisplayStyle::BarFloat { .. }
                    | crate::DisplayStyle::CustomFloat
                    | crate::DisplayStyle::Binary16
                    | crate::DisplayStyle::Raw
                    | crate::DisplayStyle::Cycle => true,
                    _ => display_unknown,
                };
                if show {
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
            }
            crate::DisplayValueType::Integer => {
                let show = match &config.style {
                    crate::DisplayStyle::BarInt { .. } | crate::DisplayStyle::Raw => true,
                    _ => display_unknown,
                };
                if show {
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
            }
            crate::DisplayValueType::String => {
                let show = match &config.style {
                    crate::DisplayStyle::Raw => true,
                    _ => display_unknown,
                };
                if show {
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
        }
    }
    // Then, display any extra parameters in sorted order
    let mut extra_params: Vec<_> = msg.keys()
        .filter(|k| !already_displayed.contains(*k) && *k != "_id_" && *k != "cycle")
        .collect();
    extra_params.sort();
    for param_name_str in extra_params {
        let param_name = param_name_str.as_str();
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
        if display_unknown {
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
    
    let mut true_max = max;
    if *i > max {
        true_max = *i + 1;
    }

    // Calculate the width needed for each number
    let max_width = true_max.to_string().len().max(min.to_string().len())-1;
    let item_width = max_width + 2; // Add 2 for spacing on either side
    
    for j in min..true_max {
        if *i == j {
            temp_str.push_str(&format!("[{:^width$}]", j, width = max_width));
        } else {
            temp_str.push_str(&format!(" {:^width$} ", j, width = max_width));
        }
    }

    format!("{:width$} ", temp_str, width = ((cols) as usize))
}

fn display_bin_float(f: &f32, _cols: usize) -> String { // cols might not be used for fixed binary
    format!("{:016b}", *f as i16) // Padded with zeros to 16 bits
}

fn display_float(f: &f32, cols: usize) -> String {
    // Clamp value between 0.0 and 1.0 for bar rendering
    let clamped = (*f).max(0.0).min(1.0);
    let total_units = BAR_CHARS.len() * cols;
    let filled_units = (clamped * total_units as f32).round() as usize;
    let full_blocks = filled_units / BAR_CHARS.len();
    let partial_index = filled_units % BAR_CHARS.len();

    let mut bar = String::new();
    if full_blocks > 0 {
        bar.push_str(&BOX.repeat(full_blocks));
    }
    if partial_index > 0 {
        bar.push_str(BAR_CHARS[partial_index]);
    }
    let bar_width = full_blocks + if partial_index > 0 { 1 } else { 0 };
    let empty = " ".repeat(cols.saturating_sub(bar_width));
    format!("|{}{}| {:.4}", bar, empty, f)
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