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
    target: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();
    let input = read_json_file(args.input).unwrap();
    let mut target = read_json_file(args.target).unwrap();
    target = merge(input, target).unwrap();
    write_json_file(args.output, target).unwrap();
}

fn merge(input: Value, mut target: Value) -> Result<Value> {
    if input.is_object() {
        let input_object = input.as_object().unwrap();
        let target_object = target.as_object_mut().unwrap();
        for key in input_object.keys() {
            if target_object.contains_key(key) {
                let value = target_object.get(key).unwrap();
                target_object.insert(
                    key.clone(),
                    merge(input_object.get(key).unwrap().clone(), value.clone())?,
                );
            }
        }
        for key in input_object.keys() {
            if !target_object.contains_key(key) {
                let value = input_object.get(key).unwrap();
                target_object.insert(key.clone(), value.clone());
            }
        }
    }
    Ok(target)
}

fn read_json_file(path: PathBuf) -> Result<Value> {
    let raw = fs::read_to_string(path)?;
    let value: Value = serde_json::de::from_str(&raw)?;
    Ok(value)
}

fn write_json_file(path: PathBuf, json: Value) -> Result<()> {
    let raw = serde_json::to_string_pretty(&json)?;
    fs::write(path, &raw)?;
    Ok(())
}
