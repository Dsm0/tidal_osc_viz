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

static RIGHT_SPACE: i32 = 25;

// NOTE: will probably replace when I get to using a tui library
fn display_text(msg: &String) {
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

pub fn display_dirt(dirt_state: &DirtState, dirt_window: &DirtWindow) {
    let mut full_str = String::new();

    // TODO: sort keys first
    if let Some(msg) = dirt_window.front() {
        full_str.push_str(msg.display_f32("cycle", display_cycle).as_str());
    };

    for (_id, msg) in dirt_state {
        let huh = display_dirt_message(msg);
        full_str.push_str(huh.as_str());
    }

    display_text(&full_str);
}

fn display_dirt_message(msg: &DirtMessage) -> String {
    let display_str: &mut String = &mut String::new();

    match msg.get("_id_") {
        Some(DirtValue::DS(s)) => {
            if s == "tick" {
                return "".to_owned();
            } else {
            }
        }
        _ => (),
    }

    let cols = {
        if let Ok((cols, _rows)) = size() {
            // the - 25 is just to make sure the string
            // doesn't wrap around the term when it's printed
            cmp::max((cols as i32) - RIGHT_SPACE, 1 as i32) as usize
        } else {
            1
        }
    };

    display_str.push_str(
        msg.display_string("_id_", |s| format!("{:<15}{} id ", "", s.to_string()))
            .as_str(),
    );

    // display_str.push_str(
    //     msg.display_f32("cycle", |f| {
    //         let bar = display_bar_float(&(f - f.floor()), 0.0, 1.0);
    //         let cycle_mods = format!(
    //             "{}/8 {}/16 {}/24 {}/32 {}/40 {}/48 {}/56 {}/64",
    //             float_mod(*f, 8.0).floor() + 1.0,
    //             float_mod(*f, 2.0 * 8.0).floor() + 1.0,
    //             float_mod(*f, 3.0 * 8.0).floor() + 1.0,
    //             float_mod(*f, 4.0 * 8.0).floor() + 1.0,
    //             float_mod(*f, 5.0 * 8.0).floor() + 1.0,
    //             float_mod(*f, 6.0 * 8.0).floor() + 1.0,
    //             float_mod(*f, 7.0 * 8.0).floor() + 1.0,
    //             float_mod(*f, 8.0 * 8.0).floor() + 1.0
    //         );
    //         format!("{}\n {}", bar, cycle_mods)
    //     })
    //     .as_str(),
    // );

    display_str.push_str(
        msg.display_f32("delta", |f| format!("| {} delta\n", f.to_string()))
            .as_str(),
    );

    display_str.push_str(
        msg.display_f32("n", |f| format!("{}  n ", display_bin_float(f)))
            .as_str(),
    );

    display_str.push_str(msg.display_string("s", |s| format!("| {} s\n", s)).as_str());

    display_str.push_str(
        msg.display_i32("orbit", |i| {
            format!("   {} orbit\n", display_bar_int(i, 0, 9))
        })
        .as_str(),
    );

    display_str.push_str(
        msg.display_f32("gain", |f| {
            format!("{} gain\n", display_bar_float(f, 0.0, 2.0))
        })
        .as_str(),
    );

    display_str.push_str(
        msg.display_f32("amp", |f| {
            format!("{} amp\n", display_bar_float(f, 0.0, 2.0))
        })
        .as_str(),
    );

    display_str.push_str(
        msg.display_f32("pan", |f| {
            format!("{} pan\n", display_bar_float(f, 0.0, 1.0))
        })
        .as_str(),
    );

    display_str.push_str(
        msg.display_f32("begin", |f| {
            format!("{} begin\n", display_bar_float(f, 0.0, 1.0))
        })
        .as_str(),
    );

    display_str.push_str(
        msg.display_f32("end", |f| {
            format!("{} end\n", display_bar_float(f, 0.0, 1.0))
        })
        .as_str(),
    );

    display_str.push_str(
        msg.display_f32("speed", |f| {
            format!("{} speed\n", display_bar_float(f, -10.0, 10.0))
        })
        .as_str(),
    );

    display_str.push_str(
        msg.display_f32("release", |f| {
            format!("{} rel\n", display_bar_float(f, 0.0, 4.0))
        })
        .as_str(),
    );

    display_str.push_str(
        msg.display_i32("cut", |i| format!("   {} cut\n", display_bar_int(i, -1, 9)))
            .as_str(),
    );

    display_str.push_str(
        msg.display_f32("cutoff", |f| {
            format!("{} cutoff\n", display_bar_float(f, 0.0, 20_000.0))
        })
        .as_str(),
    );

    display_str.push_str(
        msg.display_f32("hcutoff", |f| {
            format!("{} hcutoff\n", display_bar_float(f, 0.0, 20_000.0))
        })
        .as_str(),
    );

    // display_str.push_str("\n-------------------------------------\n");

    // display_str.push_str(msg.display_raw().as_str());

    display_str.to_string()
}

// pub fn display_param_str(name : String, s : String) -> String {
//     let display_name = shorten_name(&name);
//     match name.as_str() {
//         _ => format!("{:<8} : {:<8}",display_name,s)
//     }
// }

// pub fn display_param_int(name : String, i : i32) -> String{
//     let display_name = shorten_name(&name);
//     match name.as_str() {
//         _ => format!("{:<8} : {:<8}",display_name,i)
//     }
// }

fn remap_range(s: f32, l1: f32, h1: f32, l2: f32, h2: f32) -> f32 {
    l2 + (s - l1) * (h2 - l2) / (h1 - l1)
}

fn get_box_string(val: usize) -> String {
    if val == 0 {
        return "".to_string();
    }

    let (val_div, val_mod) = (val / BAR_CHARS.len(), val % BAR_CHARS.len());

    BOX.repeat(val_div) + BAR_CHARS[val_mod as usize]
}

fn display_cycle(f: &f32) -> String {
    let bar = display_bar_float(&(f - f.floor()), 0.0, 1.0);
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

fn display_cycle_bin(f: &f32) -> String {
    let bar = display_bar_float(&(f - f.floor()), 0.0, 1.0);
    let cycle_mods = format!("{:032b}",*f as usize);
    format!("{}\n    {}\n", bar, cycle_mods)
}

pub fn display_bar_float(f: &f32, min: f32, max: f32) -> String {
    let cols = {
        if let Ok((cols, _rows)) = size() {
            // the - 25 is just to make sure the string
            // doesn't wrap around the term when it's printed
            cmp::max((cols as i32) - RIGHT_SPACE, 1 as i32) as usize
        } else {
            1
        }
    };

    let val: f32 = remap_range(*f, min, max, 0.0, (8 * cols) as f32);

    let bar_string_index: usize = val.round() as usize;

    let bar = get_box_string(bar_string_index);
    format!("{:>3}:{:0width$}:{:<4}", min, bar, max, width = cols)
    //
}

pub fn display_bar_int(i: &i32, min: i32, max: i32) -> String {
    let cols = {
        if let Ok((cols, _rows)) = size() {
            cmp::max((cols as i32) - RIGHT_SPACE, 1 as i32)
        } else {
            1
        }
    };

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

    temp_str
}

pub fn display_bin_float(f: &f32) -> String {
    let cols = {
        if let Ok((cols, _rows)) = size() {
            cmp::max((cols as i32) - RIGHT_SPACE, 1)
        } else {
            1
        }
    };

    // let ahh = " ".repeat(cols as usize);
    // format!("{}{:016b}", ahh, *f as i32)

    format!("{:16b}", *f as i16)
}
