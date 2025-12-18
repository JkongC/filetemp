use std::io::Read;

use crate::file_types::FileType;

pub struct ArgPair<'a> {
    arg: &'static str,
    content: &'a str,
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

pub struct ConfigReader {
    whole_cache: String,
    target_file_type: FileType,
    file_handle: std::fs::File
}

enum LineResult<'a> {
    CacheName(&'a str),
    ArgItem(ArgPair<'a>),
    ParseError(String),
}

impl ConfigReader {
    pub fn new(target_file_type: FileType, config_file: std::fs::File) -> Self {
        Self {
            whole_cache: String::from(""),
            file_handle: config_file,
            target_file_type
        }
    }

    pub fn read_from_config<I>(
        &mut self,
        valid_args: I,
        file: &mut std::fs::File,
    ) -> Result<Vec<ArgCache<'_>>, String>
    where
        I: Iterator<Item = &'static str> + Clone,
    {
        let mut caches: Vec<ArgCache> = Vec::new();

        if let Err(_) = file.read_to_string(&mut self.whole_cache) {
            return Err(String::from("Failed to read from config cache file."));
        }

        let mut current_cache = ArgCache::new();
        let mut parsing_cache = false;

        for (idx, line) in self.whole_cache.lines().enumerate() {
            if line.is_empty() && parsing_cache {
                if let FileType::Unknown = current_cache.file_type {
                    return Err(format!(
                        "Argument cache parse error: File type not specified for cache \"{}\"",
                        current_cache.cache_name
                    ));
                } else if self.target_file_type == current_cache.file_type {
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
                        if !parsing_cache {
                            return Err(format!(
                                "Invalid content in config cache file: \"{}\"",
                                line
                            ));
                        } else if arg.arg == "file_type" {
                            match FileType::match_type(arg.content) {
                                FileType::Unknown => {
                                    return Err(format!(
                                        "Argument cache parse error: Invalid file type \"{}\" for cache \"{}\"",
                                        arg.content, idx
                                    ));
                                }
                                ty @ _ => current_cache.file_type = ty,
                            }
                        }
                    }
                }
            }
        }

        if parsing_cache {
            if let FileType::Unknown = current_cache.file_type {
                return Err(format!(
                    "Argument cache parse error: File type not specified for cache \"{}\"",
                    current_cache.cache_name
                ));
            } else if self.target_file_type == current_cache.file_type {
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
                return LineResult::ArgItem(ArgPair {
                    arg: valid_arg,
                    content,
                });
            }
        }

        LineResult::ParseError(format!(
            "Argument parse error: Having invalid argument name \"{}\" at line {}",
            arg, line_num
        ))
    } else {
        let cache_name_end_size: usize = line.len() - ']'.len_utf8() + 1;

        if line.chars().last().unwrap() != ']' {
            LineResult::ParseError(line_err!("Missing ]"))
        } else if cache_name_start_size >= cache_name_end_size {
            LineResult::ParseError(line_err!("Having empty cache name"))
        } else {
            LineResult::CacheName(&line[cache_name_start_size..cache_name_end_size])
        }
    }
}
