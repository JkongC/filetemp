use std::{fmt::Write, str::FromStr};

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

pub enum LanguageType {
    C,
    CXX
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
            main_language: LanguageType::CXX,
            c_standard: None,
            cxx_standard: None,
            target_type: TargetType::Executable,
            target_name: "foo",
        }
    }

    pub fn require_version(&mut self, ver: &'a str) -> &mut Self {
        self.cmake_version = ver;
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
        write!(&mut out, "cmake_minimum_required(VERSION {})\n\n", self.cmake_version).unwrap();

        if let Some(v) = self.c_standard {
            write!(&mut out, "set(CMAKE_C_STANDARD {})\nset(CMAKE_C_STANDARD_REQUIRED ON)\n\n", v).unwrap();
        }

        if let Some(v) = self.cxx_standard {
            write!(&mut out, "set(CMAKE_CXX_STANDARD {})\nset(CMAKE_CXX_STANDARD_REQUIRED ON)\n\n", v).unwrap();
        }

        match self.target_type {
            TargetType::Executable => {
                write!(&mut out, "add_executable({})\n\n", self.target_name).unwrap();
            },
            TargetType::StaticLib => {
                write!(&mut out, "add_library({} STATIC)\n\n", self.target_name).unwrap();
            },
            TargetType::SharedLib => {
                write!(&mut out, "add_library({} SHARED)\n\n", self.target_name).unwrap();
            }
        }

        write!(&mut out, "target_include_directories({pn} PRIVATE src)\ntarget_sources({pn} PRIVATE src/main.{ext})",
            pn = self.target_name, ext = if let LanguageType::CXX = self.main_language {"cpp"} else {"c"}).unwrap();

        out
    }
}
