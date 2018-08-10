use std::io::{self, BufRead};
use std::collections::HashMap;

struct LogFilters {
    // Each vector line stores a vector of individual words and wildcards
    line_filters: Vec<Vec<String>>,
    // Each unique word from `line_filters` gets its own key
    // Each key stores references to lines containing the key
    words_hash: HashMap<String, Vec<u32>>
}

impl LogFilters {
    fn new() -> Self {
        let line_filters = Vec::new();
        let words_hash = HashMap::new();

        LogFilters {
            line_filters: line_filters,
            words_hash: words_hash
        }
    }

    fn _update_hash(&mut self) {
        let last_filter_index = self.line_filters.len() as u32 - 1;
        let last_filter = self.line_filters.last();
        if last_filter.is_some() {
            let last_filter = last_filter.unwrap();
            for word in last_filter {
                self.words_hash.entry(word.clone()).or_insert(vec![last_filter_index]);
                let mut vector_indexes = self.words_hash.get_mut(word).unwrap();
                if *vector_indexes.last().unwrap() != last_filter_index {
                    vector_indexes.push(last_filter_index);
                }
            }
        }
    }

    fn _add_to_filters(&mut self, log_line: &str) {
        let words_iterator = log_line.split(|c| c == ' ' || c == '/' || c == ',' || c == '.');
        let mut words = Vec::new();

        for word in words_iterator {
            let word = word.to_string();
            words.push(word);
        }

        self.line_filters.push(words);
    }

    fn learn_line(&mut self, log_line: &str) {
        self._add_to_filters(log_line);
        self._update_hash();
    }

    fn print(self) {
        for elem in self.line_filters {
            println!("{:?}", elem);
        }
        println!();
        for (key, value) in self.words_hash {
            println!("{} : {:?}", key, value);
        }
    }
}

fn main() {
    let std_in = io::stdin();
    let mut log_filters = LogFilters::new();

    for line in std_in.lock().lines() {
        let log_line = line.expect("INVALID INPUT!");
        log_filters.learn_line(&log_line);
    }
    
    log_filters.print();
}

