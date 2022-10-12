#[macro_use]
extern crate log;

use std::path::PathBuf;
use std::{fmt, fs};

use clap::Parser;
use env_logger::Target;
use log::LevelFilter;
use serde_json::Value;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct Error {
    message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    #[clap(required = true)]
    output: Vec<PathBuf>,
    #[arg(long, action)]
    debug: bool,
}

fn main() {
    let args = Args::parse();
    let mut level = LevelFilter::Info;
    if args.debug {
        level = LevelFilter::Debug;
    }
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(level)
        .target(Target::Stdout)
        .init();
    let input = read_json_file(&args.input).unwrap();
    info!("Using file '{}' as input", get_file_name(&args.input));
    let total = args.output.len();
    let mut passed = 0;
    for (i, file) in args.output.iter().enumerate() {
        let file_name = get_file_name(&file);
        info!("[{}/{}] Merging file '{}'", i + 1, total, &file_name);
        match merge_file(&input, &file) {
            Ok(_) => passed += 1,
            Err(err) => error!("Failed to merge '{}'! Reason: {}", &file_name, err),
        }
    }
    info!(
        "Merge finished ({} total, {} passed, {} failed)",
        total,
        passed,
        total - passed
    );
}

fn merge_file(input: &Value, path: &PathBuf) -> Result<()> {
    let mut output = read_json_file(path)?;
    output = merge(input, output)?;
    write_json_file(path, output)?;
    Ok(())
}

fn merge(input: &Value, mut output: Value) -> Result<Value> {
    if input.is_object() {
        let input_object = input.as_object().unwrap();
        let output_object = output.as_object_mut().unwrap();
        for key in input_object.keys() {
            if output_object.contains_key(key) {
                let value = output_object.get(key).unwrap();
                output_object.insert(
                    key.clone(),
                    merge(&input_object.get(key).unwrap(), value.clone())?,
                );
                debug!("inserted '{}", key);
            }
        }
        for key in input_object.keys() {
            if !output_object.contains_key(key) {
                let value = input_object.get(key).unwrap();
                println!("{}", key);
                output_object.insert(key.clone(), value.clone());
            }
        }
    }
    Ok(output)
}

fn read_json_file(path: &PathBuf) -> Result<Value> {
    let raw = fs::read_to_string(path)?;
    let value: Value = serde_json::de::from_str(&raw)?;
    Ok(value)
}

fn write_json_file(path: &PathBuf, json: Value) -> Result<()> {
    let raw = serde_json::to_string_pretty(&json)?;
    fs::write(path, &raw)?;
    Ok(())
}

fn get_file_name(path: &PathBuf) -> String {
    path.file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap()
}
