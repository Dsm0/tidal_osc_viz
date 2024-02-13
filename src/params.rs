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
    fn display_i32(&self, param_name: &str, display_func: fn(&i32) -> String) -> String;
    fn display_f32(&self, param_name: &str, display_func: fn(&f32) -> String) -> String;
    fn display_string(&self, param_name: &str, display_func: fn(&String) -> String) -> String;
    fn display_raw(&self) -> String;
}

pub type DirtParamName = String;
pub type DirtMessage = HashMap<DirtParamName, DirtValue>;
pub type DirtState = HashMap<String, DirtMessage>;
pub type DirtWindow = VecDeque<DirtMessage>;

impl GetDirtValue for &DirtMessage {
    fn display_i32(&self, param_name: &str, display_func: fn(&i32) -> String) -> String {
        match self.get(param_name) {
            Some(DirtValue::DI(i)) => display_func(i),
            Some(_x) => panic!("called display_i32 on DirtValue other than DirtValue::DI(i32)"),
            _ => "".to_string(),
        }
    }

    fn display_f32(&self, param_name: &str, display_func: fn(&f32) -> String) -> String {
        match self.get(param_name) {
            Some(DirtValue::DF(f)) => display_func(f),
            Some(_x) => panic!("called display_f32 on DirtValue other than DirtValue::DF(f32)"),
            _ => "".to_string(),
        }
    }

    fn display_string(&self, param_name: &str, display_func: fn(&String) -> String) -> String {
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

pub fn to_dirt_message(msg: Vec<OscType>) -> DirtMessage {
    let mut dirt_message: DirtMessage = HashMap::new();
    for i in (0..msg.len()).step_by(2) {
        let param = &msg[i];
        let val = &msg[i + 1];
        let param_name = get_param_name(param);
        let dirt_value = to_dirt_value(val);
        dirt_message.insert(param_name, dirt_value);
    }
    dirt_message
}

fn update_dirt_message(dirt_message: &mut DirtMessage, new_msg: Vec<OscType>) {
    // let stream_id = "";
    dirt_message.clear();
    for i in (0..new_msg.len()).step_by(2) {
        let param = &new_msg[i];
        let val = &new_msg[i + 1];

        let param_name = get_param_name(param);
        let dirt_value = to_dirt_value(val);

        dirt_message.insert(param_name.to_string(), dirt_value);
    }
    // stream_id
}

pub fn update_dirt_state(dirt_state: &mut DirtState, new_msg: Vec<OscType>, msg_window: &mut VecDeque<DirtMessage>) {
    let id: String = get_id(new_msg[0].to_owned(), new_msg[1].to_owned());

    if id == "" {
        return // just don't even bother
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

    let dirt_msg: DirtMessage;

    if let Some(old_dirt_msg) = dirt_state.get_mut(&id) {

        update_dirt_message(old_dirt_msg, new_msg);
        
        msg_window.push_front(old_dirt_msg.to_owned());
    } else {
        dirt_msg = to_dirt_message(new_msg);
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