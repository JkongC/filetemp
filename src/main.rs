use cache_dir::get_data_dir;
use std::{
    fs::{self, OpenOptions}, io, path::Path
};

use crate::{
    config_file::{ArgCache, ArgCacheCollection, ConfigReader, ConfigWriter},
    file_types::{FileType, get_result_filename, process_cmake},
    program_args::{Arg, ArgProcessErr, CommandArg},
};

mod config_file;
mod file_types;
mod program_args;

fn main() {
    // Define usable arguments.
    let mut cmd = CommandArg::new();
    define_args(&mut cmd);

    // Process actual arguments, check their validity.
    if let Err(e) = cmd.process_program_args() {
        process_arg_parse_err(e);
    }

    let file_type = cmd.get_file_type();

    // Do nothing if no output is required or no possibility for cache IO.
    if !cmd.get_flag("show")
        && !cmd.get_arg("path").is_some()
        && !cmd.get_arg("save-as").is_some()
        && !cmd.get_arg("use").is_some()
    {
        return;
    }

    let arg_cache = match read_arg_cache(&mut cmd) {
        Ok(collection) => collection,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    if let Err(e) = cmd.assert_required_args_exist() {
        process_arg_parse_err(e);
    };

    let process_result: Result<String, String> = if let FileType::CMake = file_type {
        process_cmake(&cmd)
    } else {
        return;
    };

    let result_str: String = match process_result {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    if cmd.get_flag("show") {
        print!("{}", result_str);
    }

    if let Some(p) = cmd.get_arg("path") {
        if let Err(_) = write_to_file(file_type, p, &result_str) {
            eprintln!("Failed to write to file.");
        }
    }

    if let Err(e) = write_arg_cache(&mut cmd, arg_cache) {
        eprintln!("{}", e);
    }
}

fn write_to_file(ty: FileType, path: &str, content: &str) -> io::Result<()> {
    let file_name = Path::new(path).join(get_result_filename(ty));
    fs::write(&file_name, content)?;
    Ok(())
}

fn define_args(cmd: &mut CommandArg) {
    cmd.define_file_type(FileType::CMake)
            .add_arg_def(Arg::new("version").required(true))
            .add_arg_def(Arg::new("proj").required(true))
            .add_arg_def(Arg::new("main-lang").default_val("cxx"))
            .add_arg_def(Arg::new("cstd"))
            .add_arg_def(Arg::new("cxxstd"))
            .add_arg_def(Arg::new("target-type"))
            .add_arg_def(Arg::new("target-name"))
        .add_general_arg_def(Arg::new("path"))
        .add_general_arg_def(Arg::new("show").flag(true))
        .add_general_arg_def(Arg::new("save-as"))
        .add_general_arg_def(Arg::new("use"));
}

fn read_arg_cache(cmd: &mut CommandArg) -> Result<ArgCacheCollection<'static>, String> {
    let cache_name = if let Some(n) = cmd.get_arg("use") {
        n.to_string()
    } else {
        return Ok(ArgCacheCollection::new_empty());
    };

    let config_file_dir = if let Ok(path) = get_data_dir() {
        path
    } else {
        Path::new(".").to_path_buf()
    }.join(".filetemp");

    if let Err(_) = std::fs::create_dir_all(&config_file_dir) {
        return Err(format!("Failed to create cache dir: \"{:?}\"", &config_file_dir));
    }

    let config_file_path = config_file_dir.join("cache.txt");

    let config_file: fs::File = if let Ok(f) = OpenOptions::new().read(true).open(config_file_path)
    {
        f
    } else {
        return Err(String::from("Failed to open config cache file."));
    };

    let mut reader: ConfigReader = ConfigReader::new(config_file);
    let valid_args = cmd.query_valid_args().map(|arg_group| arg_group.name);
    let caches = reader.read_from_config(valid_args)?;

    let used_args = if let Some(cache_item) = caches.iter().find(|c| c.cache_name == &cache_name) {
        cache_item.args.iter()
    } else {
        return Err(format!("Used invalid cache name \"{}\"", cache_name));
    };

    for arg in used_args {
        cmd.insert_arg_if_absent(arg.arg, arg.content.to_string());
    }

    Ok(ArgCacheCollection::new(caches))
}

fn write_arg_cache<'a>(cmd: &'a mut CommandArg, mut cache: ArgCacheCollection<'a>) -> Result<(), String> {
    let cache_name: &'static str = if let Some(n) = cmd.get_arg("save-as") {
        Box::leak(n.to_string().into_boxed_str())
    } else {
        return Ok(());
    };
    
    let config_file_dir = if let Ok(path) = get_data_dir() {
        path
    } else {
        Path::new(".").to_path_buf()
    }.join(".filetemp");

    if let Err(_) = std::fs::create_dir_all(&config_file_dir) {
        return Err(format!("Failed to create cache dir: \"{:?}\"", &config_file_dir));
    }

    let config_file_path = config_file_dir.join("cache.txt");

    let config_file: fs::File = if let Ok(f) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&config_file_path)
    {
        f
    } else {
        return Err(String::from("Failed to open config cache file."));
    };

    let mut new_cache = ArgCache { cache_name: cache_name, file_type: cmd.get_file_type(), args: Vec::new() };
    for arg in cmd.extract_args() {
        new_cache.args.push(arg);
    }

    if let Some(pos) = cache.iter().position(|c| c.cache_name == cache_name) {
        cache[pos] = new_cache;
    } else {
        cache.push(new_cache);
    }

    let mut writer = ConfigWriter::new(config_file);
    if let Err(_) = writer.write_to_config(cache) {
        Err(String::from("Failed to write into cache file."))
    } else {
        Ok(())
    }
}

fn process_arg_parse_err(e: ArgProcessErr) {
    match e {
        ArgProcessErr::InvalidArg(inv) => eprintln!("Invalid argument: \"{}\"", inv),
        ArgProcessErr::InvalidFileType(invf) => eprintln!("Invalid file type: \"{}\"", invf),
        ArgProcessErr::MissingArg(ma) => eprintln!("Missing argument: \"{}\"", ma),
        _ => {}
    };
}
