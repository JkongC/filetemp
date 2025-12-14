use std::{collections::{HashMap, VecDeque}, str::FromStr};

use crate::file_types::FileType;

const HELP_MESSAGE: &'static str = "\
USAGE:
    filetemp <FILE_TYPE> [OPTIONS]

FILE_TYPE:
    CMake            Generates CMakeLists.txt

OPTIONS:
    -version <VER>          Used in \"cmake_minimum_required\"

    -main-lang <LANG>       Main language of the project, decides whether \"main.c\" or \"main.cpp\" is generated.
                            [possible values: C, CXX]
                            [default: CXX]

    -cstd <STD>             C standard

    -cxxstd <STD>           C++ standard

    -proj <NAME>            Project name

    -target-type <TYPE>     Target type
                            [possible values: executable, staticlib, sharedlib]
                            [default: executable]

    -target-name <NAME>     Target name
";

pub enum ArgProcessErr {
    PrintedHelp,
    InvalidArg(String),
    InvalidFileType(String)
}

struct ArgGroup {
    arg: &'static str,
    has_content: bool
}

pub struct CommandArg {
    file_type: FileType,
    cmake_args: Vec<ArgGroup>,
    arg_map: HashMap<&'static str, String>
}

impl CommandArg {
    pub fn new() -> Self {
        Self { file_type: FileType::Unknown, cmake_args: Vec::new(), arg_map: HashMap::new() }
    }

    pub fn add_cmake_flag(&mut self, f: &'static str) -> &mut Self {
        self.cmake_args.push(ArgGroup { arg: f, has_content: false });
        self
    }

    pub fn add_cmake_arg(&mut self, f: &'static str) -> &mut Self {
        self.cmake_args.push(ArgGroup { arg: f, has_content: true });
        self
    }

    pub fn get_arg(&self, key: &str) -> Option<&str> {
        if let Some(arg) = self.arg_map.get(key) {
            Some(arg)
        } else {
            None
        }
    }

    pub fn get_file_type(&self) -> &FileType {
        &self.file_type
    }
    
    pub fn process_program_args(&mut self) -> Result<(), ArgProcessErr> {
        let mut a = collect_raw_args();
        if a.is_empty() {
            println!("{}", HELP_MESSAGE);
            return Err(ArgProcessErr::PrintedHelp);
        }

        let file_type_name = a.pop_front().unwrap();

        self.process_arg_impl(file_type_name, a)?;

        Ok(())
    }

    fn process_arg_impl(&mut self, file_type_name: String, args: VecDeque<String>) -> Result<(), ArgProcessErr> {
        let file_type = FileType::match_type(&file_type_name);
        self.file_type = file_type.clone();

        let valid_arg_groups: &Vec<ArgGroup> = if let FileType::CMake = file_type {
            &self.cmake_args
        } else {
            return Err(ArgProcessErr::InvalidFileType(file_type_name));
        };
        
        let mut found_arg = false;
        let mut arg_ref: &'static str = "";

        for arg in args.into_iter() {
            if found_arg {
                self.arg_map.insert(arg_ref, arg);
                found_arg = false;
            } else {
                let mut verified = false;

                for valid_arg in valid_arg_groups.iter() {
                    if verify_arg(&arg, valid_arg.arg) {
                        if valid_arg.has_content {
                            arg_ref = &valid_arg.arg;
                            found_arg = true;
                        } else {
                            self.arg_map.insert(valid_arg.arg, String::from_str("").unwrap());
                        }

                        verified = true;
                        break;
                    }
                }

                if !verified {
                    return Err(ArgProcessErr::InvalidArg(arg));
                }
            }
        }

        Ok(())
    }
}

fn verify_arg(arg: &str, valid_arg: &str) -> bool {
    if arg.starts_with("--") && arg.len() > 2 {
        valid_arg.eq(&arg[2..])
    } else {
        false
    }
}

fn collect_raw_args() -> VecDeque<String> {
    let mut r: VecDeque<String> = std::env::args().collect();
    r.pop_front();
    r
}

