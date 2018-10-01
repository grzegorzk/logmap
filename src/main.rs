extern crate getopts;

use std::io::{self, BufRead};
use std::process::exit;
use std::path::Path;
use std::env;

mod logmap;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = getopts::Options::new();

    opts.optopt("l", "load", "Load filters from given path and use to scan logs from input", "PATH");
    opts.optopt("s", "save", "Save filters under given path, does not work when piping", "PATH");
    opts.optopt("c", "columns", "Ignore first N columns of input\ncolumns are created by splitting line by .,:/[]{}() \'\"\ndefault value: 2\nnote: set this value to a number allowing to ignore time stamp)", "UINT");
    opts.optopt("a", "allowed-alternatives", "during analysis each new line will be allowed not to match N times\ndefault value: 0\nrecommended value when analysing: 1 or 2", "UINT");
    opts.optflag("i", "ignore-numeric", "DO NOT ignore words containing only numbers\ndefault value: true (words containing only values are removed before analysing)");
    opts.optflag("m", "map", "Map filters from input (extend already loaded filters if -l was used)");
    opts.optflag("p", "passive", "Works only in conjunction with `l`. Analyse logs using loaded filters.");
    opts.optflag("d", "debug", "Print internal data structure");
    opts.optflag("h", "help", "Print this help menu");

    let matches = match opts.parse(&args) {
        Ok(_option) => _option,
        Err(_) => {
            println!("{}", opts.usage(""));
            exit(1);
        }
    };

    if matches.opt_present("h") {
        println!("{}", opts.usage(""));
        exit(0);
    }

    let mut log_filters = logmap::LogFilters::new();
    log_filters.ignore_first_columns = 2;
    log_filters.max_allowed_new_alternatives = 0;
    log_filters.ignore_numeric_words = true;

    if matches.opt_str("c").is_some() {
        log_filters.ignore_first_columns = match matches.opt_str("c").unwrap()
        .to_string().parse::<usize>() {
            Err(_) => panic!("Couldn't parse `columns` to UINT: {}",
                matches.opt_str("c").unwrap()),
            Ok(value) => value,
        };
    }
    if matches.opt_str("a").is_some() {
        log_filters.max_allowed_new_alternatives = match matches.opt_str("a").unwrap()
        .to_string().parse::<usize>() {
            Err(_) => panic!("Couldn't parse `columns` to UINT: {}",
                matches.opt_str("a").unwrap()),
            Ok(value) => value,
        };
    }
    if matches.opt_str("i").is_some() {
        log_filters.ignore_numeric_words = false;
    }
    if matches.opt_str("l").is_some() {
        let file_path_str = matches.opt_str("l").unwrap();
        let load_file_path = Path::new(&file_path_str);
        log_filters = logmap::LogFilters::load(load_file_path);
    }
    if matches.opt_present("m") {
        let std_in = io::stdin();
        let mut icnt = 0;
        for line in std_in.lock().lines() {
            let log_line = line.expect("INVALID INPUT!");
            log_filters.learn_line(&log_line);

            // Debug to help assessing performance
            icnt += 1;
            if icnt % 10000 == 0 {
                eprintln!("Already processed {} lines.", icnt);
            }
        }
    }
    if matches.opt_present("d") {
        log_filters.print();
    }
    if matches.opt_present("p") {
        let std_in = io::stdin();
        for line in std_in.lock().lines() {
            let log_line = line.expect("INVALID INPUT!");
            match log_filters.is_line_known(&log_line) {
                false => println!("{}", &log_line),
                true => continue,
            }
        }
    }
    if matches.opt_str("s").is_some() {
        let file_path_str = matches.opt_str("s").unwrap();
        let save_file_path = Path::new(&file_path_str);
        log_filters.save(&save_file_path);
    }
    exit(0);
}
