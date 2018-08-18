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
        filters_with_words.dedup();
        return filters_with_words;
    }

    fn _count_consequent_matches(&self, words: &Vec<String>, filter_index: u32) -> u32 {
        if self.filters.len() <= filter_index as usize {
            return 0;
        }
        let mut consequent_matches = 0;
        let mut max_consequent_matches = 0;
        let mut new_alternatives = 0;

        let mut extra_allowed_new_alternatives = 0;
        let filter_length = self.filters.get(filter_index as usize).unwrap().len();
        if filter_length < words.len() {
            extra_allowed_new_alternatives = (words.len() - filter_length) as u32;
        }

        let mut last_matching_index = -1;
        for word in words {
            let mathing_index = self._get_word_index_in_filter(word, filter_index);
            // TODO: handle same words in a row (i.e. "aaa aaa aaa")
            if mathing_index >= 0 && mathing_index > last_matching_index {
                last_matching_index = mathing_index;
                consequent_matches += 1;
                if consequent_matches > max_consequent_matches {
                    max_consequent_matches = consequent_matches;
                }
            }
            else {
                new_alternatives += 1;
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

    fn _simple_filter_from_string(words: &str) -> Vec<Vec<String>> {
        // TODO: below must be kept in sync with LogFilters::learn_line
        let words_iterator = words.split(|c|
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

        let mut filter = Vec::new();
        for word in words_iterator {
            let word = word.to_string();
            filter.push(vec![word]);
        }
        return filter;
    }

    fn _add_word_alternative(mut filter: Vec<Vec<String>>, index: usize, word: &str) -> Vec<Vec<String>> {
        if filter.get(index).is_some() {
            filter.get_mut(index).unwrap().push(word.to_string());
            return filter;
        }
        else
        {
            panic!("Failed to create test data! Extending {:?} at {}", filter, index);
        }
    }

    fn _add_test_filter(test_filters: &mut LogFilters, filter: Vec<Vec<String>>) {
        let next_filter_index = test_filters.filters.len() as u32;
        for word_alternatives in &filter {
            for word in word_alternatives {
                if test_filters.words_hash.get(word).is_some() {
                    let filter_indexes = test_filters.words_hash.get_mut(word).unwrap();
                    if !filter_indexes.contains(&next_filter_index) {
                        filter_indexes.push(next_filter_index);
                    }
                }
                else {
                    test_filters.words_hash.insert(word.clone(), vec![next_filter_index]);
                }
            }
        }
        test_filters.filters.push(filter);
    }

    fn _init_test_data() -> LogFilters {
        let mut log_filters = LogFilters::new();
        let mut complex_filter = _simple_filter_from_string("aaa qqq ccc sss");
        complex_filter = _add_word_alternative(complex_filter, 1, "bbb");
        complex_filter = _add_word_alternative(complex_filter, 2, "rrr");
        complex_filter = _add_word_alternative(complex_filter, 3, "ddd");
        _add_test_filter(&mut log_filters, complex_filter);
        _add_test_filter(&mut log_filters, _simple_filter_from_string("eee fff ggg hhh x y z"));
        _add_test_filter(&mut log_filters, _simple_filter_from_string("iii jjj kkk lll"));
        _add_test_filter(&mut log_filters, _simple_filter_from_string("mmm nnn ooo ppp"));
        _add_test_filter(&mut log_filters, _add_word_alternative(
            _simple_filter_from_string("qqq rrr sss ttt"), 3, "aaa"));
        return log_filters;
    }

    #[test]
    fn _is_word_only_numeric() {
        let log_filters = LogFilters::new();
        assert_eq!(log_filters._is_word_only_numeric(&"asdf".to_string()), false);
        assert_eq!(log_filters._is_word_only_numeric(&"123a".to_string()), false);
        assert_eq!(log_filters._is_word_only_numeric(&"a123".to_string()), false);
        assert_eq!(log_filters._is_word_only_numeric(&"6789".to_string()), true);
        assert_eq!(log_filters._is_word_only_numeric(&"".to_string()), true);
    }

    #[test]
    fn _get_sorted_filter_indexes_containing_words() {
        let log_filters = LogFilters::new();
        let words = vec!["aaa".to_string(), "bbb".to_string(), "ccc".to_string(), "ddd".to_string()];
        assert_eq!(log_filters._get_sorted_filter_indexes_containing_words(&words), vec![]);
        assert_eq!(log_filters._get_sorted_filter_indexes_containing_words(&vec![]), vec![]);

        let log_filters = _init_test_data();
        assert_eq!(log_filters._get_sorted_filter_indexes_containing_words(&vec![]), vec![]);
        let words = vec!["aaa".to_string(), "bbb".to_string(), "ccc".to_string(), "ddd".to_string()];
        assert_eq!(log_filters._get_sorted_filter_indexes_containing_words(&words), vec![0, 4]);
        let words = vec!["aaa".to_string(), "xxx".to_string()];
        assert_eq!(log_filters._get_sorted_filter_indexes_containing_words(&words), vec![0, 4]);
        let words = vec!["xxx".to_string()];
        assert_eq!(log_filters._get_sorted_filter_indexes_containing_words(&words), vec![]);
    }

    #[test]
    fn _count_consequent_matches() {
        // Test if method was used on empty data structure
        let mut log_filters = LogFilters::new();
        let words = vec!["aaa".to_string(), "bbb".to_string(), "ccc".to_string(), "ddd".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 0), 0);
        assert_eq!(log_filters._count_consequent_matches(&words, 1), 0);
        assert_eq!(log_filters._count_consequent_matches(&vec![], 0), 0);
        log_filters.max_allowed_new_alternatives = 0;
        assert_eq!(log_filters._count_consequent_matches(&words, 0), 0);
        assert_eq!(log_filters._count_consequent_matches(&words, 1), 0);
        assert_eq!(log_filters._count_consequent_matches(&vec![], 0), 0);

        let mut log_filters = _init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        // Test for existing pattern
        let words = vec!["aaa".to_string(), "bbb".to_string(), "ccc".to_string(), "ddd".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 0), 4);
        assert_eq!(log_filters._count_consequent_matches(&words, 1), 0);
        // Test out of bounds
        assert_eq!(log_filters._count_consequent_matches(&words, 5), 0);
        // Test empty words vector
        assert_eq!(log_filters._count_consequent_matches(&vec![], 0), 0);
        // Test if words vector can be smaller than filter
        let words = vec!["iii".to_string(), "jjj".to_string(), "lll".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 2), 3);
        let words = vec!["iii".to_string(), "lll".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 2), 2);
        let words = vec!["iii".to_string(), "jjj".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 2), 2);
        let words = vec!["jjj".to_string(), "kkk".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 2), 2);
        let words = vec!["iii".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 2), 1);
        let words = vec!["jjj".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 2), 1);
        // Test if word alternative will be matched
        let words = vec!["aaa".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 4), 1);
        // Test if 1 word alternative is allowed
        let words = vec!["aaa".to_string(), "bbb".to_string(), "ccc".to_string(), "xxx".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 0), 3);
        let words = vec!["aaa".to_string(), "xxx".to_string(), "ccc".to_string(), "ddd".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 0), 3);
        let words = vec!["aaa".to_string(), "bbb".to_string(), "zzz".to_string(), "xxx".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 0), 0);
        let words = vec!["aaa".to_string(), "xxx".to_string(), "zzz".to_string(), "ddd".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 0), 0);
        // Test if words vector can be longer than existing filter
        let words = vec!["aaa".to_string(), "bbb".to_string(), "ccc".to_string(), "ddd".to_string(),
            "eee".to_string(), "fff".to_string(), "ggg".to_string(), "hhh".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 0), 4);
        // Test if longer words vector will be allowed to contain 1 word alternative to existing word
        let words = vec!["aaa".to_string(), "xxx".to_string(), "ccc".to_string(), "ddd".to_string(),
            "eee".to_string(), "fff".to_string(), "ggg".to_string(), "hhh".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 3), 0);
        // Test if longer words vector will be allowed to contain 1 new word alternative
        let words = vec!["aaa".to_string(), "xxx".to_string(), "bbb".to_string(), "ccc".to_string(),
            "ddd".to_string(), "fff".to_string(), "ggg".to_string(), "hhh".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 4), 0);
        // Test if words vector and filter vector must contain words in the same order
        let words = vec!["ddd".to_string(), "ccc".to_string(), "bbb".to_string(), "aaa".to_string()];
        assert_eq!(log_filters._count_consequent_matches(&words, 0), 0);
    }

    #[test]
    fn _get_word_index_in_filter() {
        let log_filters = LogFilters::new();
        assert_eq!(log_filters._get_word_index_in_filter(&"aaa".to_string(), 0), -1);
        assert_eq!(log_filters._get_word_index_in_filter(&"aaa".to_string(), 100), -1);
        assert_eq!(log_filters._get_word_index_in_filter(&"".to_string(), 0), -1);

        let log_filters = _init_test_data();
        assert_eq!(log_filters._get_word_index_in_filter(&"aaa".to_string(), 0), 0);
        assert_eq!(log_filters._get_word_index_in_filter(&"aaa".to_string(), 4), 3);
        assert_eq!(log_filters._get_word_index_in_filter(&"".to_string(), 4), -1);
        assert_eq!(log_filters._get_word_index_in_filter(&"aaa".to_string(), 1), -1);
        assert_eq!(log_filters._get_word_index_in_filter(&"aaa".to_string(), 100), -1);
    }

    #[test]
    fn _is_word_in_filter() {
        let log_filters = _init_test_data();
        assert_eq!(log_filters._is_word_in_filter(&"aaa".to_string(), 0), true);
        assert_eq!(log_filters._is_word_in_filter(&"aaa".to_string(), 4), true);
        assert_eq!(log_filters._is_word_in_filter(&"hhh".to_string(), 1), true);
        assert_eq!(log_filters._is_word_in_filter(&"aaa".to_string(), 1), false);
        assert_eq!(log_filters._is_word_in_filter(&"xxx".to_string(), 2), false);
        assert_eq!(log_filters._is_word_in_filter(&"xxx".to_string(), 100), false);
        assert_eq!(log_filters._is_word_in_filter(&"".to_string(), 0), false);
    }
}
