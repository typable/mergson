#[macro_use]
extern crate log;

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process;

use clap::Parser;
use env_logger::Target;
use log::LevelFilter;
use serde_json::Value;

use mergson::Result;

#[derive(Debug, Parser)]
#[clap(author, version)]
struct Args {
    #[arg(short, long, help = "The JSON file for the merge")]
    input: PathBuf,
    #[arg(short, long, help = "The JSON files affected by the merge")]
    #[clap(required = true)]
    output: Vec<PathBuf>,
    #[arg(
        long,
        action,
        help = "Displays additional information during the merge"
    )]
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
    let input = read_json_file(&args.input);
    if let Err(err) = input {
        error!("Failed to read {}!", get_file_name(&args.input));
        error!("    {}", err);
        process::exit(1);
    }
    info!("Using file {} as input", get_file_name(&args.input));
    let total = args.output.len();
    let mut passed = 0;
    let mut changed = 0;
    for file in args.output {
        let file_name = get_file_name(&file);
        info!("Merging file {}", &file_name);
        match merge_file(input.as_ref().unwrap(), &file) {
            Ok(count) => {
                info!("Merged {} keys into {}", count, &file_name);
                passed += 1;
                if count > 0 {
                    changed += 1;
                }
            }
            Err(err) => {
                error!("Failed to merge {}!", &file_name);
                error!("    {}", err);
            }
        }
    }
    info!(
        "Merge finished ({} total, {} changed, {} passed, {} failed)",
        total,
        changed,
        passed,
        total - passed
    );
}

fn merge_file(input: &Value, path: &PathBuf) -> Result<usize> {
    let target = read_json_file(path)?;
    let (output, count) = merge(input, target, vec![])?;
    write_json_file(path, output)?;
    Ok(count)
}

fn merge(input: &Value, mut output: Value, tree: Vec<String>) -> Result<(Value, usize)> {
    let mut count = 0;
    if input.is_object() {
        let input_object = input.as_object().unwrap();
        let output_object = output.as_object_mut().unwrap();
        for key in input_object.keys() {
            if output_object.contains_key(key) {
                let value = output_object.get(key).unwrap();
                let mut sub_tree = tree.clone();
                sub_tree.push(key.clone());
                let (sub_output, sub_count) =
                    merge(input_object.get(key).unwrap(), value.clone(), sub_tree)?;
                output_object.insert(key.clone(), sub_output);
                count += sub_count;
            }
        }
        for key in input_object.keys() {
            if !output_object.contains_key(key) {
                let value = input_object.get(key).unwrap();
                let mut path = tree.clone();
                path.push(key.clone());
                debug!("    + {}", path.join("."));
                output_object.insert(key.clone(), value.clone());
                count += 1;
            }
        }
    }
    Ok((output, count))
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

fn get_file_name(path: &Path) -> String {
    path.file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap()
}
