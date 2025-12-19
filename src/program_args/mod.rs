use std::{
    collections::{HashMap, VecDeque},
    ops::{Deref, DerefMut},
};

use crate::file_types::FileType;

const HELP_MESSAGE: &'static str = "\
filetemp 0.1.0

USAGE:
    filetemp <FILE_TYPE> <CMAKE_OPTIONS> [GENERAL_OPTIONS]

FILE_TYPE:
    CMake            Generates CMakeLists.txt

CMAKE_OPTIONS:
    SYNTAX: <--version <VER>> <--proj <NAME>> [...]

    --version <VER>          Used in \"cmake_minimum_required\"

    --proj <NAME>            Project name

    --main-lang <LANG>       Main language of the project, decides whether \"main.c\" or \"main.cpp\" is generated.
                            [possible values: C, CXX]
                            [default: CXX]

    --cstd <STD>             C standard

    --cxxstd <STD>           C++ standard

    --target-type <TYPE>     Target type
                            [possible values: executable, staticlib, sharedlib]
                            [default: executable]

    --target-name <NAME>     Target name, use project name if not specified.

GENERAL_OPTIONS:
    SYNTAX: [--show] [--path <PATH>]

    --show                   Show output content to stdout

    --path <PATH>            Path where the file is generated to
";

pub struct ArgPair<'a> {
    pub arg: &'static str,
    pub content: &'a str,
}

pub enum ArgProcessErr {
    PrintedHelp,
    InvalidArg(String),
    InvalidFileType(String),
    MissingArg(String),
}

pub struct Arg {
    pub name: &'static str,
    is_flag: bool,
    is_required: bool,
    has_default_value: bool,
    default_value: &'static str,
}

impl Arg {
    pub fn new(arg_name: &'static str) -> Self {
        Self {
            name: arg_name,
            is_flag: false,
            is_required: false,
            has_default_value: false,
            default_value: "",
        }
    }

    pub fn flag(mut self, f: bool) -> Self {
        self.is_flag = f;
        self
    }

    pub fn required(mut self, req: bool) -> Self {
        self.is_required = req;
        self
    }

    pub fn default_val(mut self, v: &'static str) -> Self {
        self.default_value = v;
        self.has_default_value = true;
        self
    }
}

pub struct ArgGroup {
    definition: Arg,
    found: bool,
}

impl Deref for ArgGroup {
    type Target = Arg;

    fn deref(&self) -> &Arg {
        &self.definition
    }
}

impl DerefMut for ArgGroup {
    fn deref_mut(&mut self) -> &mut Arg {
        &mut self.definition
    }
}

impl ArgGroup {
    fn new(arg: Arg) -> Self {
        Self {
            definition: arg,
            found: false,
        }
    }
}

pub struct CommandArg {
    file_type: FileType,
    defined_args: HashMap<FileType, Vec<ArgGroup>>,
    general_args: Vec<ArgGroup>,
    arg_map: HashMap<&'static str, String>,
}

pub struct ArgFileTypeView<'a> {
    arg_ref: &'a mut CommandArg,
    ty: FileType,
}

impl<'a> Deref for ArgFileTypeView<'a> {
    type Target = CommandArg;

    fn deref(&self) -> &Self::Target {
        self.arg_ref
    }
}

impl<'a> DerefMut for ArgFileTypeView<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.arg_ref
    }
}

impl CommandArg {
    pub fn new() -> Self {
        Self {
            file_type: FileType::Unknown,
            defined_args: HashMap::new(),
            general_args: Vec::new(),
            arg_map: HashMap::new(),
        }
    }

    pub fn define_file_type(&mut self, ty: FileType) -> ArgFileTypeView<'_> {
        ArgFileTypeView { arg_ref: self, ty }
    }

    pub fn add_general_arg_def(&mut self, arg: Arg) -> &mut Self {
        self.add_arg_def(FileType::Unknown, arg)
    }

    pub fn add_arg_def(&mut self, file_type: FileType, arg: Arg) -> &mut Self {
        if let FileType::Unknown = file_type {
            self.general_args.push(ArgGroup::new(arg));
        } else {
            self.defined_args
                .entry(file_type)
                .or_default()
                .push(ArgGroup::new(arg));
        }

        self
    }

    pub fn get_arg(&self, key: &str) -> Option<&str> {
        if let Some(arg) = self.arg_map.get(key) {
            Some(arg)
        } else {
            None
        }
    }

    pub fn get_flag(&self, key: &str) -> bool {
        self.arg_map.get(key).is_some()
    }

    pub fn get_file_type(&self) -> FileType {
        self.file_type
    }

    pub fn process_program_args(&mut self) -> Result<(), ArgProcessErr> {
        let mut a = collect_raw_args();
        if a.is_empty() {
            println!("{}", HELP_MESSAGE);
            return Err(ArgProcessErr::PrintedHelp);
        }

        let file_type_name = a.pop_front().unwrap();
        match FileType::match_type(&file_type_name) {
            FileType::Unknown => return Err(ArgProcessErr::InvalidFileType(file_type_name)),
            ty @ _ => self.file_type = ty,
        };

        self.process_arg_impl(a)
    }

    pub fn query_valid_args(&mut self) -> impl Iterator<Item = &ArgGroup> + Clone {
        let ty_args = self.defined_args.entry(self.file_type).or_default().iter();
        let gn_args = self.general_args.iter();

        ty_args.chain(gn_args)
    }

    /// Insert an argument item if absent.
    /// Assumes that arg and content is correct.
    pub fn insert_arg_if_absent(&mut self, arg: &'static str, content: String) {
        self.arg_map.entry(arg).or_insert(content);

        for valid_args in self.defined_args.get_mut(&self.file_type).unwrap().iter_mut().chain(self.general_args.iter_mut()) {
            if valid_args.name == arg {
                valid_args.found = true;
            }
        }
    }

    pub fn extract_args(&self) -> Vec<ArgPair<'_>> {
        let mut args: Vec<ArgPair> = Vec::new();
        for (&arg, content) in self.arg_map.iter() {
            args.push(ArgPair { arg, content });
        }

        args
    }

    fn process_arg_impl(&mut self, args: VecDeque<String>) -> Result<(), ArgProcessErr> {
        let valid_args = self.defined_args.get_mut(&self.file_type).unwrap();
        let general_args: &mut Vec<ArgGroup> = &mut self.general_args;

        let mut found_arg = false;
        let mut arg_ref: &'static str = "";

        for arg in args.into_iter() {
            if found_arg {
                self.arg_map.entry(arg_ref).or_insert(arg);
                found_arg = false;
            } else {
                let mut verified = false;

                for valid_arg in valid_args.iter_mut().chain(general_args.iter_mut()) {
                    if !verify_arg(&arg, valid_arg.name) {
                        continue;
                    }

                    if !valid_arg.is_flag {
                        arg_ref = &valid_arg.name;
                        found_arg = true;
                    } else {
                        self.arg_map.entry(valid_arg.name).or_insert(String::from("true"));
                    }

                    valid_arg.found = true;
                    verified = true;
                    break;
                }

                if !verified {
                    return Err(ArgProcessErr::InvalidArg(arg));
                }
            }
        }

        Ok(())
    }

    pub fn assert_required_args_exist(&mut self) -> Result<(), ArgProcessErr> {
        let valid_args = self.defined_args.get_mut(&self.file_type).unwrap();
        let general_args: &mut Vec<ArgGroup> = &mut self.general_args;
        let all_valid_args = valid_args.iter_mut().chain(general_args.iter_mut());

        let mut missing_args = false;
        let mut missing_msg = String::new();
        for valid_arg in all_valid_args {
            if valid_arg.found {
                continue;
            }

            if valid_arg.is_required {
                if missing_args {
                    missing_msg.push_str(", ");
                }

                missing_msg.push_str(valid_arg.name);
                missing_args = true;

                continue;
            }

            if valid_arg.has_default_value {
                self.arg_map
                    .insert(valid_arg.name, valid_arg.default_value.to_string());
            }
        }

        if missing_args {
            Err(ArgProcessErr::MissingArg(missing_msg))
        } else {
            Ok(())
        }
    }
}

impl<'a> ArgFileTypeView<'a> {
    pub fn add_arg_def(&mut self, arg: Arg) -> &mut Self {
        self.arg_ref.add_arg_def(self.ty, arg);
        self
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
