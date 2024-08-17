mod instruction;
mod interpreter;
mod window;

use std::{fs, process::ExitCode};

use crate::interpreter::Interpreter;

fn main() -> ExitCode {
    let Some(rom_path) = std::env::args().nth(1) else {
        eprintln!("Invalid file path");
        return ExitCode::FAILURE;
    };
    let rom_file = fs::read(rom_path).unwrap();

    let mut interpreter = Interpreter::new(rom_file);
    interpreter.execute().expect("Failed to run ROM");
    // interpreter.dump_memory();

    ExitCode::SUCCESS
}
