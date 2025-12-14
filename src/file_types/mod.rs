use std::{str::FromStr};

#[derive(Clone)]
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