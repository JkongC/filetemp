use crate::program_args::CommandArg;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum FileType {
    CMake,
    Unknown,
}

impl FileType {
    pub fn match_type(name: &str) -> Self {
        if name.eq_ignore_ascii_case("cmake") {
            Self::CMake
        } else {
            Self::Unknown
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            FileType::CMake => "cmake",
            FileType::Unknown => "unknown",
        }
    }
}

pub mod cmake_files;

pub fn process_args(cmd: &CommandArg) -> Result<String, String> {
    match cmd.get_file_type() {
        FileType::CMake => Ok(cmake_files::process_args(cmd)),
        FileType::Unknown => Err(String::from("Unknown file type")),
    }
}

pub fn verify_existed_args(cmd: &CommandArg) -> Result<(), String> {
    match cmd.get_file_type() {
        FileType::CMake => cmake_files::verify_existed_args(cmd),
        FileType::Unknown => Err(String::from("Unknown file type")),
    }
}

pub fn generate_example(cmd: &CommandArg, path: &std::path::Path) -> Result<(), String> {
    match cmd.get_file_type() {
        FileType::CMake => cmake_files::generate_example(cmd, path),
        FileType::Unknown => Err(String::from("Unknown file type")),
    }
}

pub fn get_result_filename(ty: FileType) -> &'static str {
    if let FileType::CMake = ty {
        cmake_files::get_filename()
    } else {
        ""
    }
}
