use std::fs;
use std::path::PathBuf;

use clap::Parser;
use serde_json::Value;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct Error {}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self {}
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Self {}
    }
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    #[clap(required = true)]
    output: Vec<PathBuf>,
    #[arg(short, long)]
    debug: Option<bool>,
}

fn main() {
    let args = Args::parse();
    let input = read_json_file(&args.input).unwrap();
    println!("Input: '{}'", get_file_name(&args.input));
    for file in args.output {
        println!("Merging '{}'...", get_file_name(&file));
        let mut output = read_json_file(&file).unwrap();
        output = merge(input.clone(), output, args.debug.unwrap_or_default()).unwrap();
        write_json_file(&file, output).unwrap();
        println!("Merged.");
    }
    println!("Done.");
}

fn merge(input: Value, mut output: Value, debug: bool) -> Result<Value> {
    if input.is_object() {
        let input_object = input.as_object().unwrap();
        let output_object = output.as_object_mut().unwrap();
        for key in input_object.keys() {
            if output_object.contains_key(key) {
                let value = output_object.get(key).unwrap();
                output_object.insert(
                    key.clone(),
                    merge(input_object.get(key).unwrap().clone(), value.clone(), debug)?,
                );
                if debug {
                    println!("    inserted '{}'", key);
                }
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
