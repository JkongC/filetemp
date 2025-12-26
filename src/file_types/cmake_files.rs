use std::{fmt::Write, str::FromStr};

use crate::program_args::CommandArg;

const C_EXAMPLE: &'static str = "\
#include <stdio.h>

int main()
{
    printf(\"Hello World\");
    return 0;
}";

const CXX_OLD_EXAMPLE: &'static str = "\
#include <iostream>

int main()
{
    std::cout << \"Hello World\" << std::endl;
}";

const CXX_23_EXAMPLE: &'static str = "\
#include <print>

int main()
{
    std::println(\"Hello World\");
}";

#[derive(PartialEq, Eq)]
pub enum TargetType {
    Executable,
    StaticLib,
    SharedLib,
}

impl FromStr for TargetType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("executable") {
            Ok(Self::Executable)
        } else if s.eq_ignore_ascii_case("staticlib") {
            Ok(Self::StaticLib)
        } else if s.eq_ignore_ascii_case("sharedlib") {
            Ok(Self::SharedLib)
        } else {
            Err(())
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum LanguageType {
    C,
    CXX,
}

impl FromStr for LanguageType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("C") {
            Ok(Self::C)
        } else if s.eq_ignore_ascii_case("CXX") {
            Ok(Self::CXX)
        } else {
            Err(())
        }
    }
}

pub struct CMakeListsFile<'a> {
    cmake_version: &'a str,
    project_name: &'a str,
    main_language: LanguageType,
    c_standard: Option<i32>,
    cxx_standard: Option<i32>,
    target_type: TargetType,
    target_name: &'a str,
}

impl<'a> CMakeListsFile<'a> {
    pub fn new() -> Self {
        Self {
            cmake_version: "",
            project_name: "",
            main_language: LanguageType::CXX,
            c_standard: None,
            cxx_standard: None,
            target_type: TargetType::Executable,
            target_name: "",
        }
    }

    pub fn require_version(&mut self, ver: &'a str) -> &mut Self {
        self.cmake_version = ver;
        self
    }

    pub fn set_project_name(&mut self, name: &'a str) -> &mut Self {
        self.project_name = name;
        self
    }

    pub fn set_main_language(&mut self, lang: LanguageType) -> &mut Self {
        self.main_language = lang;
        self
    }

    pub fn require_c_standard(&mut self, standard: i32) -> &mut Self {
        self.c_standard = Some(standard);
        self
    }

    pub fn require_cxx_standard(&mut self, standard: i32) -> &mut Self {
        self.cxx_standard = Some(standard);
        self
    }

    pub fn set_target_type(&mut self, ty: TargetType) -> &mut Self {
        self.target_type = ty;
        self
    }

    pub fn set_target_name(&mut self, name: &'a str) -> &mut Self {
        self.target_name = name;
        self
    }

    pub fn output_string(&self) -> String {
        let mut out = String::new();
        write!(
            &mut out,
            "cmake_minimum_required(VERSION {})\n\n",
            self.cmake_version
        )
        .unwrap();

        if let Some(v) = self.c_standard {
            write!(
                &mut out,
                "set(CMAKE_C_STANDARD {})\nset(CMAKE_C_STANDARD_REQUIRED ON)\n\n",
                v
            )
            .unwrap();
        }

        if let Some(v) = self.cxx_standard {
            write!(
                &mut out,
                "set(CMAKE_CXX_STANDARD {})\nset(CMAKE_CXX_STANDARD_REQUIRED ON)\n\n",
                v
            )
            .unwrap();
        }

        write!(&mut out, "project({})\n\n", self.project_name).unwrap();

        match self.target_type {
            TargetType::Executable => {
                write!(&mut out, "add_executable({})\n\n", self.target_name).unwrap();
            }
            TargetType::StaticLib => {
                write!(&mut out, "add_library({} STATIC)\n\n", self.target_name).unwrap();
            }
            TargetType::SharedLib => {
                write!(&mut out, "add_library({} SHARED)\n\n", self.target_name).unwrap();
            }
        }

        write!(&mut out, "target_include_directories({pn} PRIVATE src)\ntarget_sources({pn} PRIVATE src/main.{ext})",
            pn = self.target_name, ext = if let LanguageType::CXX = self.main_language {"cpp"} else {"c"}).unwrap();

        out
    }
}

pub(super) fn process_args(cmd: &CommandArg) -> String {
    let mut f: CMakeListsFile = CMakeListsFile::new();

    macro_rules! use_argument {
        ($str_name:literal, $func:ident) => {
            if let Some(a) = cmd.get_arg($str_name) {
                f.$func(a);
            }
        };
        ($type:ty, $str_name:literal, $func:ident) => {
            if let Some(a) = cmd.get_arg($str_name) {
                f.$func(a.parse::<$type>().unwrap());
            }
        };
    }

    use_argument!("version", require_version);
    use_argument!("proj", set_project_name);
    use_argument!(i32, "cstd", require_c_standard);
    use_argument!(i32, "cxxstd", require_cxx_standard);
    use_argument!(LanguageType, "main-lang", set_main_language);
    use_argument!(TargetType, "target-type", set_target_type);

    if let Some(tn) = cmd.get_arg("target-name") {
        f.set_target_name(tn);
    } else {
        f.set_target_name(cmd.get_arg("proj").unwrap());
    }

    f.output_string()
}

pub(super) fn verify_existed_args(cmd: &CommandArg) -> Result<(), String> {
    macro_rules! assert_parse_ok {
        ($type: ty, $arg: literal, $errfmt: literal) => {
            if let Some(r) = cmd.get_arg($arg)
                && r.parse::<$type>().is_err()
            {
                return Err(format!($errfmt, r));
            }
        };
    }

    assert_parse_ok!(i32, "cstd", "Invalid C standard: {}");
    assert_parse_ok!(i32, "cxxstd", "Invalid C++ standard: {}");
    assert_parse_ok!(LanguageType, "main-lang", "Invalid main language: {}");
    assert_parse_ok!(TargetType, "target-type", "Invalid target type: {}");

    Ok(())
}

pub(super) fn generate_example(cmd: &CommandArg, path: &std::path::Path) -> Result<(), String> {
    let src_path = path.join("src");
    if let Err(_) = std::fs::create_dir_all(&src_path) {
        return Err(String::from("Failed to create source directory"));
    }

    let main_path;
    let main_content;
    if let LanguageType::C = cmd.get_arg_parsed_unsafe("main-lang") {
        main_path = src_path.join("main.c");
        main_content = C_EXAMPLE;
    } else {
        main_path = src_path.join("main.cpp");
        main_content = if cmd
            .get_arg("cxxstd")
            .map(|s| s.parse::<i32>().unwrap() >= 23)
            .unwrap_or(false)
        {
            CXX_23_EXAMPLE
        } else {
            CXX_OLD_EXAMPLE
        };
    }

    if let Err(_) = std::fs::write(&main_path, main_content.as_bytes()) {
        Err(String::from("Failed to create example main file"))
    } else {
        Ok(())
    }
}

pub(super) fn get_filename() -> &'static str {
    "CMakeLists.txt"
}
