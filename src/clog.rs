#![allow(dead_code)]

use std::collections::HashMap;

pub struct LogFilters {
    // Each vector line stores a vector of individual words variations
    // filters (Vec) - collection of all log lines
    //    |
    //    |- words (Vec) - collection of word variations within log line
    //          |
    //          |- word_variations (Vec) - collection of words within word variation
    //                   |
    //                   |- word (String)
    filters: Vec<Vec<Vec<String>>>,
    // Each unique word from `filters` gets its own key
    // Each key stores references to lines containing the key
    words_hash: HashMap<String, Vec<u32>>,
    // Minimum required consequent matches to consider lines similar
    min_req_consequent_matches: u32,
    // Maximum allowed new alternatives
    max_allowed_new_alternatives: u32,
}

impl LogFilters {
    pub fn new() -> Self {
        let filters = Vec::new();
        let words_hash = HashMap::new();

        LogFilters {
            filters: filters,
            words_hash: words_hash,
            min_req_consequent_matches: 3,
            max_allowed_new_alternatives: 1
        }
    }

    pub fn save_filters(self) {
        // TODO
    }

    pub fn load_filters(self) {
        // TODO
    }

    pub fn print(self) {
        if self.filters.len() > 0 {
            for elem in self.filters {
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

    pub fn learn_line(&mut self, log_line: &str) {
        let words_iterator = log_line.split(|c|
            c == ' ' ||
            c == '/' ||
            c == ',' ||
            c == '.' ||
            c == ':' ||
            c == '"' ||
            c == '(' ||
            c == ')' ||
            c == '{' ||
            c == '}' ||
            c == '[' ||
            c == ']');
        let mut words = Vec::new();

        for word in words_iterator {
            let word = word.to_string();
            if word.len() > 0 && !self._is_word_only_numeric(&word) {
                words.push(word);
            }
        }

        let matched_filter_index = self._find_best_matching_filter_index(&words);
        if matched_filter_index >= 0 {
            self._update_filter(words, matched_filter_index as u32);
        }
        else {
            self._add_filter(words);
        }
    }

    fn _is_word_only_numeric(&self, word: &String) -> bool {
        let chars_are_numeric: Vec<bool> = word.chars().map(|c|c.is_numeric()).collect();
        return !chars_are_numeric.contains(&false);
    }

    fn _find_best_matching_filter_index(&self, words: &Vec<String>) -> i32 {
        if self.filters.len() == 0 {
            return -1
        }

        let mut best_matching_filter_index: i32 = -1;
        let mut max_consequent_matches = 0;
        for filter_index in self._get_filter_indexes_with_min_req_matches(words) {
            let max_cur_consequent_matches = self._count_consequent_matches(words, filter_index);
            if max_cur_consequent_matches > max_consequent_matches {
                max_consequent_matches = max_cur_consequent_matches;
                best_matching_filter_index = filter_index as i32;
            }
        }
        if words.len() > self.min_req_consequent_matches as usize {
            if max_consequent_matches >= self.min_req_consequent_matches {
                return best_matching_filter_index;
            }
        }
        else {
            if words.len() == max_consequent_matches as usize {
                return best_matching_filter_index;
            }
        }
        return -1;
    }

    fn _get_filter_indexes_with_min_req_matches(&self, words: &Vec<String>) -> Vec<u32> {
        let mut filter_indexes_with_min_req_matches: Vec<u32> = Vec::new();
        let filters_with_words = self._get_sorted_filter_indexes_containing_words(words);
        let mut matches = 0;
        let mut prev_index = -1;
        let mut last_inserted_index = -1;
        for filter_index in filters_with_words {
            if last_inserted_index == filter_index as i32 {
                continue;
            }
            if prev_index != filter_index as i32 {
                matches = 1;
                prev_index = filter_index as i32;
                continue;
            }
            else {
                matches = matches + 1;
            }
            if matches == self.min_req_consequent_matches {
                matches = 0;
                filter_indexes_with_min_req_matches.push(filter_index as u32);
                last_inserted_index = filter_index as i32;
            }
        }
        return filter_indexes_with_min_req_matches;
    }

    fn _get_sorted_filter_indexes_containing_words(&self, words: &Vec<String>) -> Vec<u32> {
        let mut filters_with_words: Vec<u32> = Vec::new();
        for word in words {
            if self.words_hash.get(word).is_some() {
                let vector_indexes = self.words_hash.get(word).unwrap();
                filters_with_words.extend(vector_indexes);
            }
        }
        filters_with_words.sort();
        return filters_with_words;
    }

    fn _count_consequent_matches(&self, words: &Vec<String>, filter_index: u32) -> u32 {
        let mut consequent_matches = 0;
        let mut max_consequent_matches = 0;
        let mut new_alternatives = 0;

        let mut extra_allowed_new_alternatives = 0;
        let filter_length = self.filters.get(filter_index as usize).unwrap().len();
        if filter_length < words.len() {
            extra_allowed_new_alternatives = (words.len() - filter_length) as u32;
        }

        for word in words {
            // TODO: consequent matches in filter should be ensured too
            if self._get_word_index_in_filter(word, filter_index) >= 0 {
                consequent_matches += 1;
                if consequent_matches > max_consequent_matches {
                    max_consequent_matches = consequent_matches;
                }
            }
            // TODO: handle situation if words contains more elements than filter
            else {
                new_alternatives += 1;
                consequent_matches = 0;
                if new_alternatives > self.max_allowed_new_alternatives + extra_allowed_new_alternatives {
                    return 0;
                }
            }
        }
        return max_consequent_matches;
    }

    fn _get_word_index_in_filter(&self, word: &String, filter_index: u32) -> i32 {
        let filter = self.filters.get(filter_index as usize);
        if filter.is_none() {
            return -1;
        }

        let filter = filter.unwrap();
        for word_alternative_index in 0..filter.len() {
            if filter.get(word_alternative_index).unwrap().contains(word) {
                return word_alternative_index as i32;
            }
        }
        return -1;
    }

    fn _update_filter(&mut self, words: Vec<String>, filter_index: u32) {
        self._normalise_till_first_match(&words, filter_index);
        for word in words {
        }
    }

    fn _normalise_till_first_match(&mut self, words: &Vec<String>, filter_index: u32) {
        let (first_word, first_filter) = self._get_first_matching_indexes(&words, filter_index);
        if first_word >= 0 && first_filter >= 0 {
            if first_word - first_filter > 0 {
                let mut front_words = Vec::new();
                for word in &words[0..(first_word - first_filter) as usize] {
                    self._update_hash(&word, filter_index);
                    front_words.push(vec![word.clone()]);
                }
                let filters = self.filters.get_mut(filter_index as usize).unwrap();
                filters.splice(0..0, front_words);
            }
        }
        else {
            // This should never happen
            // TODO: consider throw rather than log?
            eprintln!("Words not matching selected filter during filter update; '{:?}'", words);
        }
    }

    fn _get_first_matching_indexes(&self, words: &Vec<String>, filter_index: u32) -> (i32, i32) {
        if words.len() == 0 || self.filters.get(filter_index as usize).is_none() {
            return (-1, -1);
        }

        for word_index in 0..words.len() {
            let word = words.get(word_index).unwrap();
            let matching_filter_index = self._get_word_index_in_filter(word, filter_index);
            if  matching_filter_index > 0 {
                return (word_index as i32, matching_filter_index);
            }
        }

        return (-1, -1);
    }

    fn _add_filter(&mut self, words: Vec<String>) {
        let mut words_alternatives = Vec::new();
        let expected_index = self.filters.len() as u32;

        for word in words {
            if word.len() > 0 {
                self._update_hash(&word, expected_index);
                words_alternatives.push(vec![word]);
            }
        }
        if words_alternatives.len() > 0 {
            self.filters.push(words_alternatives);
        }
    }

    fn _update_hash(&mut self, word: &String, filter_index: u32) {
        self.words_hash.entry(word.clone()).or_insert(vec![filter_index]);
        let vector_indexes = self.words_hash.get_mut(word).unwrap();
        if ! vector_indexes.contains(&filter_index) {
            vector_indexes.push(filter_index);
            vector_indexes.sort();
        }
    }

    fn _is_word_in_filter(&self, word: &String, filter_index: u32) -> bool {
        let filter = self.filters.get(filter_index as usize);
        if filter.is_none() {
            return false;
        }
        
        let filter = filter.unwrap();
        for word_alternatives in filter {
            for word_alternative in word_alternatives {
                if word_alternative == word {
                    return true;
                }
            }
        }
        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn _is_word_only_numeric() {
        let log_filters = LogFilters::new();
        assert_eq!(log_filters._is_word_only_numeric(&"asdf".to_string()), false);
        assert_eq!(log_filters._is_word_only_numeric(&"123a".to_string()), false);
        assert_eq!(log_filters._is_word_only_numeric(&"a123".to_string()), false);
        assert_eq!(log_filters._is_word_only_numeric(&"6789".to_string()), true);
        assert_eq!(log_filters._is_word_only_numeric(&"".to_string()), true);
    }
}