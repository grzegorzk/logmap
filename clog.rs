use std::io::{self, BufRead};
use std::collections::HashMap;

struct LogFilters {
    // Each vector line stores a vector of individual words variations
    // line_filters (Vec)
    //    |
    //    |- words (Vec)
    //          |
    //          |- word_variations (Vec)
    //                   |
    //                   |- word (String)
    line_filters: Vec<Vec<Vec<String>>>,
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

    fn _update_hash(&mut self, word: &String, filter_index: u32) {
        self.words_hash.entry(word.clone()).or_insert(vec![filter_index]);
        let mut vector_indexes = self.words_hash.get_mut(word).unwrap();
        vector_indexes.push(filter_index);
        vector_indexes.sort();
        vector_indexes.dedup();
    }

    fn _add_to_filters(&mut self, log_line: &str) {
        let words_iterator = log_line.split(|c|
            c == ' ' ||
            c == '/' ||
            c == ',' ||
            c == '.' ||
            c == ':' ||
            c == '"' ||
            c == '(' ||
            c == ')' ||
            c == '[' ||
            c == ']');
        let mut words = Vec::new();
        let expected_index = self.line_filters.len() as u32;

        for word in words_iterator {
            let word = word.to_string();
            if word.len() > 0 {
                self._update_hash(&word, expected_index);

                let word_variations = vec![word];
                words.push(word_variations);
            }
        }

        self.line_filters.push(words);
    }

    fn _find_best_match(min_consequent_matches: u32) {
        // TODO
    }

    fn learn_line(&mut self, log_line: &str) {
        self._add_to_filters(log_line);
    }

    fn save_filters(self){
        // TODO
    }

    fn load_filters(self){
        // TODO
    }

    fn print(self) {
        if self.line_filters.len() > 0 {
            for elem in self.line_filters {
                println!("{:?}", elem);
            }
        }
        else {
            println!("No filters added yet");
        }
        println!();
        if self.words_hash.len() > 0 {
            for (key, value) in self.words_hash {
                println!("{} : {:?}", key, value);
            }
        }
        else {
            println!("No words with references to filters added yet");
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

