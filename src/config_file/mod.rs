use std::{
    fmt::Write,
    io::{Read, Write as _},
    ops::{Deref, DerefMut},
};

use line_ending::LineEnding;

use crate::{file_types::FileType, program_args::ArgPair};

static mut CACHE_STR: Option<&'static str> = None;

/// Return the whole cache string slice.
/// UNSAFE, always ensure CACHE_STR is already initialized.
fn get_cache_str() -> &'static str {
    unsafe { CACHE_STR.unwrap() }
}

pub struct ArgCache<'a> {
    pub file_type: FileType,
    pub cache_name: &'a str,
    pub args: Vec<ArgPair<'a>>,
}

impl ArgCache<'_> {
    fn new() -> Self {
        Self {
            file_type: FileType::Unknown,
            cache_name: "",
            args: Vec::new(),
        }
    }
}

pub struct ArgCacheCollection<'a> {
    caches: Vec<ArgCache<'a>>,
}

impl<'a> ArgCacheCollection<'a> {
    pub fn new(caches: Vec<ArgCache<'a>>) -> Self {
        Self { caches }
    }

    pub fn new_empty() -> Self {
        Self { caches: Vec::new() }
    }
}

impl<'a> Deref for ArgCacheCollection<'a> {
    type Target = Vec<ArgCache<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.caches
    }
}

impl<'a> DerefMut for ArgCacheCollection<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.caches
    }
}

pub struct ConfigReader {
    file_handle: std::fs::File,
}

enum LineResult<'a> {
    CacheName(&'a str),
    FileTy(FileType),
    ArgItem(ArgPair<'a>),
    ParseError(String),
    Discard,
}

impl ConfigReader {
    pub fn new(config_file: std::fs::File) -> Self {
        Self {
            file_handle: config_file,
        }
    }

    pub fn read_from_config<'b, I>(&mut self, valid_args: I) -> Result<Vec<ArgCache<'b>>, String>
    where
        I: Iterator<Item = &'static str> + Clone,
    {
        let mut caches: Vec<ArgCache> = Vec::new();

        let mut temp_str = String::new();
        if let Err(_) = self.file_handle.read_to_string(&mut temp_str) {
            return Err(String::from("Failed to read from config cache file."));
        }
        unsafe {
            CACHE_STR = Some(Box::leak(temp_str.into_boxed_str()));
        }

        let mut current_cache = ArgCache::new();
        let mut parsing_cache = false;

        for (idx, line) in get_cache_str().lines().enumerate() {
            if line.is_empty() && parsing_cache {
                if let FileType::Unknown = current_cache.file_type {
                    return Err(format!(
                        "Argument cache parse error: File type not specified for cache \"{}\"",
                        current_cache.cache_name
                    ));
                } else {
                    caches.push(current_cache);
                    current_cache = ArgCache::new();
                    parsing_cache = false;
                }
            } else {
                match parse_line(valid_args.clone(), idx, line) {
                    LineResult::ParseError(err) => {
                        return Err(err);
                    }
                    LineResult::CacheName(cache_name) => {
                        current_cache.cache_name = cache_name;
                        parsing_cache = true;
                    }
                    LineResult::ArgItem(arg) => {
                        if parsing_cache {
                            current_cache.args.push(ArgPair {
                                arg: arg.arg,
                                content: arg.content,
                            });
                        } else {
                            return Err(format!(
                                "Invalid content in config cache file: \"{}\"",
                                line
                            ));
                        }
                    }
                    LineResult::FileTy(ty) => match ty {
                        FileType::Unknown => {
                            return Err(format!(
                                "Argument cache parse error: Invalid file type for cache \"{}\"",
                                current_cache.cache_name
                            ));
                        }
                        _ => current_cache.file_type = ty,
                    },
                    LineResult::Discard => {}
                }
            }
        }

        if parsing_cache {
            if let FileType::Unknown = current_cache.file_type {
                return Err(format!(
                    "Argument cache parse error: File type not specified for cache \"{}\"",
                    current_cache.cache_name
                ));
            } else {
                caches.push(current_cache);
            }
        }

        Ok(caches)
    }
}

fn parse_line<I>(valid_args: I, line_num: usize, line: &str) -> LineResult<'_>
where
    I: Iterator<Item = &'static str>,
{
    macro_rules! line_err {
        ($msg: literal) => {
            format!(
                concat!("Argument cache parse error: ", $msg, "at line {}"),
                line_num
            )
        };
    }

    let mut is_arg_item: bool = true;
    let mut cache_name_start_size: usize = 0;

    let mut arg_end_size: usize = 0;
    let mut ct_start_size: usize = 0;

    for (idx, (bidx, c)) in line.char_indices().enumerate() {
        if idx == 0 && c == '[' {
            is_arg_item = false;
            cache_name_start_size = '['.len_utf8();
            break;
        }

        if c == ':' {
            if idx == 0 {
                return LineResult::ParseError(line_err!("Having empty argument name"));
            }

            arg_end_size = bidx;
            ct_start_size = bidx + ':'.len_utf8();

            if ct_start_size == line.len() {
                return LineResult::ParseError(line_err!("Having empty argument content"));
            }
            break;
        }
    }

    if is_arg_item {
        let arg = &line[0..arg_end_size];
        let content = &line[ct_start_size..];

        for valid_arg in valid_args {
            if arg == valid_arg {
                if arg == "save-as" || arg == "use" {
                    return LineResult::Discard;
                } else {
                    return LineResult::ArgItem(ArgPair {
                        arg: valid_arg,
                        content,
                    });
                }
            }
        }

        if arg == "file_type" {
            return LineResult::FileTy(FileType::match_type(content));
        }

        LineResult::ParseError(format!(
            "Argument parse error: Having invalid argument name \"{}\" at line {}",
            arg, line_num
        ))
    } else {
        let cache_name_end_size: usize = line.len() - ']'.len_utf8();

        if line.chars().last().unwrap() != ']' {
            LineResult::ParseError(line_err!("Missing ]"))
        } else if cache_name_start_size >= cache_name_end_size {
            LineResult::ParseError(line_err!("Having empty cache name"))
        } else {
            LineResult::CacheName(&line[cache_name_start_size..cache_name_end_size])
        }
    }
}

pub struct ConfigWriter {
    file_handle: std::fs::File,
}

impl ConfigWriter {
    pub fn new(file: std::fs::File) -> Self {
        Self { file_handle: file }
    }

    pub fn write_to_config(
        &mut self,
        cache: ArgCacheCollection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let le = match LineEnding::from_current_platform() {
            LineEnding::CR => "\r",
            LineEnding::LF => "\n",
            LineEnding::CRLF => "\r\n",
        };

        let mut result = String::new();
        for item in cache.iter() {
            write!(&mut result, "[{}]{}", item.cache_name, le)?;
            write!(&mut result, "file_type:{}{}", item.file_type.to_str(), le)?;
            for arg_item in item.args.iter() {
                write!(&mut result, "{}:{}{}", arg_item.arg, arg_item.content, le)?;
            }
            result.push_str(le);
        }

        self.file_handle.write(result.as_bytes())?;

        Ok(())
    }
}
