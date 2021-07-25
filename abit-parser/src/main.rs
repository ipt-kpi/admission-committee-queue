use anyhow::Result;
use regex::Regex;
use std::process;
use tokio::fs::File;

use crate::parser::Parser;

mod parser;

const HELP: &str = "\
apit-parser
USAGE:
  apit-parser [OPTIONS] --input PATH [INPUT]
FLAGS:
  -h, --help            Prints help information
OPTIONS:
  --year NUMBER         Sets a number of year
  --single-file STATE   Sets a state to use single-file
ARGS:
  <INPUT>
";

#[derive(Debug)]
struct AppArgs {
    single_file: bool,
    year: u8,
    input: Vec<String>,
}

#[tokio::main]
async fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(error) => {
            eprintln!("Error: {}.", error);
            process::exit(1);
        }
    };

    let parser = Parser::new(args.year);
    if args.single_file {
        let mut file = File::create("all").await.expect("Failed to create file");
        for code in args.input {
            match parser.get_info(&code, &mut file).await {
                Err(error) => {
                    eprintln!("Error: {}.", error);
                }
                _ => {}
            }
        }
    } else {
        for code in args.input {
            let mut file = File::create(&code)
                .await
                .expect(&format!("Failed to create file with code: {}", code));
            match parser.get_info(&code, &mut file).await {
                Err(error) => {
                    eprintln!("Error: {}.", error);
                }
                _ => {}
            }
        }
    }
}

fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        process::exit(0);
    }

    let regex = Regex::new("^[0-9]{6}$").expect("Failed to create regex");

    let args = AppArgs {
        single_file: pargs.opt_value_from_str("--single-file")?.unwrap_or(true),
        year: pargs.opt_value_from_fn("--year", parse_year)?.unwrap_or(21),
        input: {
            pargs
                .finish()
                .to_vec()
                .iter()
                .filter_map(|code| match code.to_str().map(String::from) {
                    Some(code) => {
                        if regex.is_match(&code) {
                            Some(code)
                        } else {
                            None
                        }
                    }
                    None => None,
                })
                .collect()
        },
    };

    Ok(args)
}

fn parse_year(s: &str) -> Result<u8, &'static str> {
    s.parse().map_err(|_| "not a number").and_then(|number| {
        if number >= 15 && number <= 21 {
            Ok(number)
        } else {
            Err("no interval from 15 to 21")
        }
    })
}
