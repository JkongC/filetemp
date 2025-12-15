use std::{fs, io, path::Path};

use crate::{file_types::{FileType, get_cmake_filename, process_cmake}, program_args::{Arg, ArgProcessErr, CommandArg}};

mod file_types;
mod program_args;

fn main() {
    let mut cmd = CommandArg::new();
    define_args(&mut cmd);

    match cmd.process_program_args() {
        Err(ref e) => {
            match e {
                ArgProcessErr::InvalidArg(inv) => println!("Invalid argument: \"{}\"", inv),
                ArgProcessErr::InvalidFileType(invf) => println!("Invalid file type: \"{}\"", invf),
                ArgProcessErr::MissingArg(ma) => println!("Missing argument: \"{}\"", ma),
                _ => {}
            };
            return;
        },
        Ok(_) => {}
    };

    let file_type = cmd.get_file_type();

    let process_result = if let FileType::CMake = file_type {
        process_cmake(&cmd)
    } else {
        return;
    };

    let result_str = match process_result {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    if cmd.get_flag("show") {
        print!("{}", result_str);
    }

    let path = match cmd.get_arg("path") {
        Some(p) => p,
        None => { return; }
    };

    if let Err(_) = write_to_file(file_type, path, &result_str) {
        println!("Failed to write to file.");
    }
}

fn write_to_file(ty: FileType, path: &str, content: &str) -> io::Result<()> {
    let base = Path::new(path);
    let full_path = if let FileType::CMake = ty {
        base.join(get_cmake_filename())
    } else {
        base.to_path_buf()
    };

    fs::write(full_path, content)?;

    Ok(())
}

fn define_args(cmd: &mut CommandArg) {
    cmd.define_file_type(FileType::CMake)
            .add_arg(Arg::new("version").required(true))
            .add_arg(Arg::new("proj").required(true))
            .add_arg(Arg::new("main-lang").default_val("cxx"))
            .add_arg(Arg::new("cstd"))
            .add_arg(Arg::new("cxxstd"))
            .add_arg(Arg::new("target-type"))
            .add_arg(Arg::new("target-name"))
            .finish()
        .add_general_arg(Arg::new("path"))
        .add_general_arg(Arg::new("show").flag(true));
}