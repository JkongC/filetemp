use std::str::FromStr;

use crate::{program_args::CommandArg};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum FileType {
    CMake,
    Unknown
}

impl FileType {
    pub fn match_type(name: &str) -> Self {
        if name.eq_ignore_ascii_case("cmake") {
            Self::CMake
        } else {
            Self::Unknown
        }
    }
}

impl FromStr for FileType {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(FileType::match_type(s))
    }
}

pub mod cmake_files;

pub fn process_cmake(cmd: &CommandArg) -> Result<String, String> {
    cmake_files::process(cmd)
}

pub fn get_cmake_filename() -> &'static str {
    cmake_files::get_filename()
}