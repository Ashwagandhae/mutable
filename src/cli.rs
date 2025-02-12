use itertools::Itertools;
use std::env;

pub fn get_config() -> Config {
    let args: Vec<String> = env::args().collect();
    let mut input_file = None;
    let mut output_file = None;
    for (arg_1, arg_2) in args.into_iter().tuple_windows::<(String, String)>() {
        match arg_1.as_str() {
            "-i" | "--input" => input_file = Some(arg_2.clone()),
            "-o" | "--output" => output_file = Some(arg_2.clone()),
            _ => (),
        }
    }
    return Config {
        input_file,
        output_file,
    };
}

#[derive(Debug, Clone)]
pub struct Config {
    pub input_file: Option<String>,
    pub output_file: Option<String>,
}
