use crate::{file_types::{FileType, cmake_files::{CMakeListsFile, LanguageType, TargetType}}, program_args::{ArgProcessErr, CommandArg}};

mod file_types;
mod program_args;

fn main() {
    let mut cmd = CommandArg::new();
    cmd.add_cmake_arg("version")
        .add_cmake_arg("main-lang")
        .add_cmake_arg("cstd")
        .add_cmake_arg("cxxstd")
        .add_cmake_arg("proj")
        .add_cmake_arg("target-type")
        .add_cmake_arg("target-name");

    match cmd.process_program_args() {
        Err(ref e) => {
            match e {
                ArgProcessErr::InvalidArg(inv) => println!("Invalid argument: {}", inv),
                ArgProcessErr::InvalidFileType(invf) => println!("Invalid file type: {}", invf),
                _ => {}
            };
            return;
        },
        _ => {}
    };

    match cmd.get_file_type() {
        FileType::CMake => process_cmake(&cmd),
        FileType::Unknown => println!("Unknown file type.")
    }
}

fn process_cmake(cmd: &CommandArg) {
    let mut f: CMakeListsFile = CMakeListsFile::new();

    macro_rules! use_argument {
        ($name:ident, $func:ident) => {
            if let Some($name) = cmd.get_arg(stringify!($name)) {
                f.$func($name);
            }
        };
        ($name:ident, $str_name:literal, $func:ident) => {
            if let Some($name) = cmd.get_arg(stringify!($str_name)) {
                f.$func($name);
            }
        }
    }

    macro_rules! use_parsed_argument {
        ($type:ty, $name:ident, $func:ident, $err: literal) => {
            if let Some($name) = cmd.get_arg(stringify!($name)) {
                match $name.parse::<$type>() {
                    Ok(result) => f.$func(result),
                    Err(_) => {
                        println!($err, $name);
                        return;
                    }
                };
            }
        };
        ($type:ty, $name:ident, $str_name:literal, $func:ident, $err: literal) => {
            if let Some($name) = cmd.get_arg($str_name) {
                match $name.parse::<$type>() {
                    Ok(result) => f.$func(result),
                    Err(_) => {
                        println!($err, $name);
                        return;
                    }
                };
            }
        }
    }

    use_argument!(version, require_version);
    use_parsed_argument!(i32, cstd, require_c_standard, "Invalid C standard: {}");
    use_parsed_argument!(i32, cxxstd, require_cxx_standard, "Invalid C++ standard: {}");
    use_parsed_argument!(LanguageType, ml, "main-lang", set_main_language, "Invalid main language type: {}");
    use_argument!(tn, "target-name", set_target_name);
    use_parsed_argument!(TargetType, tt, "target-type", set_target_type, "Invalid target type: {}");

    print!("{}", f.output_string());
}