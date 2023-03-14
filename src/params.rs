use rosc::OscType;
use std::collections::HashMap;


#[derive(Debug)]
pub enum DirtValue {
    DI(i32),
    DF(f32),
    DS(String)
}

pub type DirtParamName = String;

// type DirtParam = (DirtParamName, DirtValue);
pub type DirtMessage = HashMap<DirtParamName, DirtValue>;

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
        _ => panic!("oscValue not float, integer, or string")    
    }
}


pub fn to_dirt_message(msg: Vec<OscType>) -> DirtMessage {

    let mut dirt_message: DirtMessage = HashMap::new();

    for i in (0..msg.len()).step_by(2) {
        let param = &msg[i];
        let val = &msg[i+1];

        let param_name = get_param_name(param.clone());
        let dirt_value = to_dirt_value(val.clone());

        dirt_message.insert(param_name, dirt_value);
    };

    dirt_message
}


// fn get_display_func
