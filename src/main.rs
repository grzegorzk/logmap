use std::io::{self, BufRead};
mod clog;

fn main() {
    let std_in = io::stdin();
    let mut log_filters = clog::LogFilters::new();
    log_filters.max_allowed_new_alternatives = 2;

    let mut icnt = 0;
    for line in std_in.lock().lines() {
        let log_line = line.expect("INVALID INPUT!");
        log_filters.learn_line(&log_line);
        // Debug to help assessing performance
        icnt += 1;
        if icnt % 1000 == 0 {
            eprintln!("{}", icnt);
        }
    }

    log_filters.print();
}

