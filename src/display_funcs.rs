extern crate rosc;
extern crate termsize;

// use crate::params;
// use crate::params::DirtParam;



// fn shorten_name(name : &str) -> String {
//     let huh : String = name.chars().take(6).collect();
//     huh
// }
// 
// 
// pub fn display_param(param : params::DirtParam) -> String {
//     match param {
//         DirtParam::PF(name,f) => format!("{:<10} : {}", name, display_bar_float(f,0.0,2.0)),
//         // DirtParam::PF(name,f) => display_param_float(name,f),
//         DirtParam::PI(name,i) => display_param_int(name,i),
//         DirtParam::PS(name,s) => display_param_str(name,s),
//     }
// }
// 
// pub fn display_param_float(name : String, f : f32) -> String {
//     let display_name = shorten_name(&name);
//     match name.as_str() {
//         "gain" => {
//             let bar = "#".repeat((f * 10.0) as usize);
//             format!("{:<8} : {:}",display_name,bar)
//         }
//         _ => format!("{:<8} : {:<8}",display_name,f)
//     }
// }
// 
// pub fn display_param_str(name : String, s : String) -> String {
//     let display_name = shorten_name(&name);
//     match name.as_str() {
//         _ => format!("{:<8} : {:<8}",display_name,s)
//     }
// }
// 
// 
// pub fn display_param_int(name : String, i : i32) -> String{
//     let display_name = shorten_name(&name);
//     match name.as_str() {
//         _ => format!("{:<8} : {:<8}",display_name,i)
//     }
// }
// 
// 
// // https://forum.unity.com/threads/re-map-a-number-from-one-range-to-another.119437/
// fn remap_range(s : f32, a1: f32, a2:f32, b1: f32, b2: f32) -> f32 {
//     b1 + (s-a1)*(b2-b1)/(a2-a1)
// } 


