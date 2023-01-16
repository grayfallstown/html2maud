#![feature(decl_macro)]
#[macro_use]
extern crate rocket;

use html2maud::*;
use clap::{Parser};
use std::io::{self, BufRead};
use std::fs;
mod paro_gui;
use crate::paro_gui::start_gui;


#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'i', long="in", value_name = "input html file", value_hint = clap::ValueHint::FilePath)]
    r#in: Option<std::path::PathBuf>,

    #[arg(short = 'o', long="out", value_name = "output maud file", value_hint = clap::ValueHint::FilePath)]
    out: Option<std::path::PathBuf>,
    
    #[arg(short = 's', long="stdin", value_name = "read from stdin")]
    stdin: bool,
}


fn run_cli(args: &Args) {
    let html = if args.stdin {
        let stdin = io::stdin();
        let stdin_string: String = stdin.lock().lines()
            .map(|line| line.expect("could not read from stdin"))
            .collect::<Vec<String>>().join("\n");
        stdin_string
    } else {
        match &args.r#in {
            None => panic!("input file must be Some(file) here"),
            Some(file) => std::fs::read_to_string(file).expect("could not read input file")
        }
    };

    let maudtemplate = html2maud(&html);

    match &args.out {
        None => println!("{}", maudtemplate),
        Some(file) => fs::write(file, maudtemplate).expect("Unable to write output file"),
    }
}

fn run_gui() {
    println!("neither --stdin nor --in specified. Starting gui. If you wanted to use the cli, use html2maaud --help");
    // panic!("gui not yet implemented");
    start_gui();
}

fn main() {
    let args = Args::parse();

    if !args.stdin && args.r#in.is_none() {
        run_gui();
    } else {
        run_cli(&args);
    }
}
