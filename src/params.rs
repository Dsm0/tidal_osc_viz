use rosc::OscType;
use std::collections::HashMap;

#[derive(Debug)]
pub enum DirtValue {
    DI(i32),
    DF(f32),
    DS(String),
}

// working in Tidal, you should know what params
// have what types

pub trait GetDirtValue {
    fn display_i32(&self, param_name: &str, display_func: fn(&i32) -> String) -> String;
    fn display_f32(&self, param_name: &str, display_func: fn(&f32) -> String) -> String;
    fn display_string(&self, param_name: &str, display_func: fn(&String) -> String) -> String;
    fn display_raw(&self) -> String;
}

pub type DirtParamName = String;
pub type DirtMessage = HashMap<DirtParamName, DirtValue>;
pub type DirtData = Vec<DirtMessage>;

impl GetDirtValue for &DirtMessage {
    fn display_i32(&self, param_name: &str, display_func: fn(&i32) -> String) -> String {
        match self.get(param_name) {
            Some(DirtValue::DI(i)) => display_func(i),
            Some(x) => panic!("called display_i32 on DirtValue other than DirtValue::DI(i32)"),
            _ => "\n".to_string(),
        }
    }

    fn display_f32(&self, param_name: &str, display_func: fn(&f32) -> String) -> String {
        match self.get(param_name) {
            Some(DirtValue::DF(f)) => display_func(f),
            Some(x) => panic!("called display_f32 on DirtValue other than DirtValue::DF(f32)"),
            _ => "\n".to_string(),
        }
    }

    fn display_string(&self, param_name: &str, display_func: fn(&String) -> String) -> String {
        match self.get(param_name) {
            Some(DirtValue::DS(s)) => display_func(s),
            Some(x) => panic!("called display_f32 on DirtValue other than DirtValue::DF(f32)"),
            _ => "\n".to_string(),
        }
    }

    fn display_raw(&self) -> String {
        let mut huh: Vec<String> = Vec::new();
        for (param_name, value) in *self {
            match value {
                DirtValue::DF(f) => huh.push(format!("{}: {}", param_name, f)),
                DirtValue::DI(i) => huh.push(format!("{}: {}", param_name, i)),
                DirtValue::DS(s) => huh.push(format!("{}: {}", param_name, s)),
            }
        }
        huh.join(",")
    }
}

// type DirtParam = (DirtParamName, DirtValue);

// type DirtDisplay = fn(DirtValue) -> String;
// type DirtParamDisplay = fn(DirtValue) -> String;
// pub type DirtDisplayMap = HashMap<DirtParamName,DirtDisplay>;

fn get_param_name(osc_param: OscType) -> DirtParamName {
    match osc_param {
        OscType::String(s) => s,
        _ => panic!("passed non-string type OscType to 'get_param_name'"),
    }
}

pub fn to_dirt_value(osc_value: OscType) -> DirtValue {
    match osc_value {
        OscType::Float(f) => DirtValue::DF(f),
        OscType::Int(i) => DirtValue::DI(i),
        OscType::String(s) => DirtValue::DS(s),
        _ => panic!("oscValue not float, integer, or string"),
    }
}

pub fn to_dirt_message(msg: Vec<OscType>) -> DirtMessage {
    let mut dirt_message: DirtMessage = HashMap::new();
    for i in (0..msg.len()).step_by(2) {
        let param = &msg[i];
        let val = &msg[i + 1];

        let param_name = get_param_name(param.clone());
        let dirt_value = to_dirt_value(val.clone());

        dirt_message.insert(param_name, dirt_value);
    }
    dirt_message
}

fn update_dirt_message(dirt_message: &mut DirtMessage, new_msg: Vec<OscType>) {
    for i in (0..new_msg.len()).step_by(2) {
        let param = &new_msg[i];
        let val = &new_msg[i + 1];

        let param_name = get_param_name(param.clone());
        let dirt_value = to_dirt_value(val.clone());

        dirt_message.insert(param_name, dirt_value);
    }
}



pub fn update_dirt_data(dirt_data: &mut DirtData, new_msg: Vec<OscType>){

    let mut old_message = &mut dirt_data[0];

    update_dirt_message(&mut old_message,new_msg)

}