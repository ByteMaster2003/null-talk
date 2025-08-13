use common::utils::resolve_path;
use std::{
    io::{self, Write},
    path::PathBuf,
};

pub fn take_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub fn take_file_input(prompt: &str) -> PathBuf {
    let mut input: String;

    loop {
        input = take_user_input(prompt);
        match resolve_path(&input) {
            Ok(path) => return path,
            Err(_) => {
                eprintln!("Invalid file path: {}", &input);
                continue;
            }
        };
    }
}
