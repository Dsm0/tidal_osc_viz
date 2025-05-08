use rosc::OscType;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug)]
pub enum DirtValue {
    DI(i32),
    DF(f32),
    DS(String),
}


// working in Tidal, you should know what params have which types

pub trait GetDirtValue {
    fn display_i32<F>(&self, param_name: &str, display_func: F) -> String
    where
        F: FnOnce(&i32) -> String;
    fn display_f32<F>(&self, param_name: &str, display_func: F) -> String
    where
        F: FnOnce(&f32) -> String;
    fn display_string<F>(&self, param_name: &str, display_func: F) -> String
    where
        F: FnOnce(&String) -> String;
    fn display_raw(&self) -> String;
}

pub type DirtParamName = String;
pub type DirtMessage = HashMap<DirtParamName, DirtValue>;
pub type DirtState = HashMap<String, DirtMessage>;
pub type DirtWindow = VecDeque<DirtMessage>;

impl GetDirtValue for &DirtMessage {
    fn display_i32<F>(&self, param_name: &str, display_func: F) -> String
    where
        F: FnOnce(&i32) -> String,
    {
        match self.get(param_name) {
            Some(DirtValue::DI(i)) => display_func(i),
            Some(_x) => panic!("called display_i32 on DirtValue other than DirtValue::DI(i32)"),
            _ => "".to_string(),
        }
    }

    fn display_f32<F>(&self, param_name: &str, display_func: F) -> String
    where
        F: FnOnce(&f32) -> String,
    {
        match self.get(param_name) {
            Some(DirtValue::DF(f)) => display_func(f),
            Some(_x) => panic!("called display_f32 on DirtValue other than DirtValue::DF(f32)"),
            _ => "".to_string(),
        }
    }

    fn display_string<F>(&self, param_name: &str, display_func: F) -> String
    where
        F: FnOnce(&String) -> String,
    {
        match self.get(param_name) {
            Some(DirtValue::DS(s)) => display_func(s),
            Some(_x) => panic!("called display_f32 on DirtValue other than DirtValue::DF(f32)"),
            _ => "".to_string(),
        }
    }

    fn display_raw(&self) -> String {
        let mut huh: Vec<String> = Vec::new();
        for (param_name, value) in *self {
            match value {
                DirtValue::DF(f) => huh.push(format!("{}:{}", param_name, f)),
                DirtValue::DI(i) => huh.push(format!("{}:{}", param_name, i)),
                DirtValue::DS(s) => huh.push(format!("{}:{}", param_name, s)),
            }
        }
        huh.join(",")
    }
}

fn get_param_name(osc_param: &OscType) -> DirtParamName {
    match osc_param {
        OscType::String(s) => s.to_string(),
        _ => panic!("passed non-string type OscType to 'get_param_name'"),
    }
}

pub fn to_dirt_value(osc_value: &OscType) -> DirtValue {
    match osc_value {
        OscType::Float(f) => DirtValue::DF(*f),
        OscType::Int(i) => DirtValue::DI(*i),
        OscType::String(s) => DirtValue::DS(s.to_string()),
        _ => panic!("oscValue not float, integer, or string"),
    }
}

// Helper function to parse OSC arguments into a DirtMessage
fn parse_osc_args_to_dirt_message(args: &[OscType]) -> DirtMessage {
    let mut dirt_message: DirtMessage = HashMap::new();
    for i in (0..args.len()).step_by(2) {
        if i + 1 < args.len() { // Ensure there's a value for the parameter
            let param = &args[i];
            let val = &args[i + 1];
            let param_name = get_param_name(param);
            let dirt_value = to_dirt_value(val);
            dirt_message.insert(param_name, dirt_value);
        }
    }
    dirt_message
}

pub fn to_dirt_message(msg: Vec<OscType>) -> DirtMessage {
    parse_osc_args_to_dirt_message(&msg)
}

fn update_dirt_message(dirt_message: &mut DirtMessage, new_msg_args: Vec<OscType>) {
    dirt_message.clear();
    // Consider using extend if parse_osc_args_to_dirt_message returns an iterator
    // For now, direct insertion from the parsed new message is fine.
    let new_parsed_message = parse_osc_args_to_dirt_message(&new_msg_args);
    for (key, value) in new_parsed_message {
        dirt_message.insert(key, value);
    }
}

pub fn update_dirt_state(dirt_state: &mut DirtState, new_msg_args: Vec<OscType>, msg_window: &mut VecDeque<DirtMessage>) {
    if new_msg_args.len() < 2 { // Need at least _id_ and its value
        return;
    }
    let id: String = get_id(new_msg_args[0].to_owned(), new_msg_args[1].to_owned());

    if id.is_empty() { // Changed from id == "" for slight idiomatic improvement
        return; // just don't even bother
    }

    if msg_window.len() > 10 {
        let msg_to_remove = msg_window.pop_back();
        match msg_to_remove.expect("REASON").get("_id_") {
            Some(DirtValue::DS(id)) => {
                dirt_state.insert(id.to_string(),HashMap::new());
            }
            _ => {}
        }
    }

    if let Some(old_dirt_msg) = dirt_state.get_mut(&id) {

        update_dirt_message(old_dirt_msg, new_msg_args);
        
        msg_window.push_front(old_dirt_msg.to_owned());
    } else {
        let dirt_msg = to_dirt_message(new_msg_args);
        dirt_state.insert(id, dirt_msg.clone());

        msg_window.push_front(dirt_msg);
    }



}

fn get_id(msg0: OscType, msg1: OscType) -> String {
    let param = if let OscType::String(s) = msg0 {
        s
    } else {
        panic!("index 0 of message should be a string")
    };

    if param.as_str() != "_id_" {
        return "".to_string()
        // panic!("index 0 of message should be '__id__' but it's {}\n", param)
    }

    if let OscType::String(id) = msg1 {
        id
    } else {
        panic!("index 1 of message should be a string")
    }
}

pub fn new_dirt_window(size: usize) -> DirtWindow {
    VecDeque::with_capacity(size)
}