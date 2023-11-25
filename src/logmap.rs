use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Default)]
pub struct LogFilters {
    /// Each `filters` element stores a vector of individual words variations
    /// filters (Vec) - collection of all log lines
    ///    |
    ///    |- filter (Vec) - collection of word variations within log line
    ///          |
    ///          |- word_variations (Vec) - collection of words within word variation
    ///                   |
    ///                   |- word1 (String)
    ///                   |- word2 (String)
    filters: Vec<Vec<Vec<String>>>,
    /// Each unique word from `filters` gets its own key
    /// Each key stores references to lines containing the key
    words_hash: HashMap<String, Vec<usize>>,
    /// Maximum allowed new alternatives when analysing any new line
    pub max_allowed_new_alternatives: usize,
    /// If `denote_optional` is found within alternatives then column is treated as optional
    denote_optional: String,
    /// Should words that contain only numbers be ignored
    pub ignore_numeric_words: bool,
    /// Drop first columns before analysing
    pub ignore_first_columns: usize,
}

impl LogFilters {
    pub fn new() -> Self {
        let filters = Vec::new();
        let words_hash = HashMap::new();

        LogFilters {
            filters,
            words_hash,
            max_allowed_new_alternatives: 0,
            // below must never land as word alternative
            denote_optional: ".".to_string(),
            ignore_numeric_words: true,
            ignore_first_columns: 2,
        }
    }

    pub fn save(&self, path: &Path) {
        let mut log_filters_str = String::new();
        log_filters_str += &self.max_allowed_new_alternatives.to_string();
        log_filters_str += "\n";
        log_filters_str += &self.denote_optional;
        log_filters_str += "\n";
        log_filters_str += &self.ignore_numeric_words.to_string();
        log_filters_str += "\n";
        log_filters_str += &self.ignore_first_columns.to_string();
        log_filters_str += "\n";
        log_filters_str += &self.to_string();

        let path_display = path.display();
        let mut file = match File::create(&path) {
            Err(why) => panic!("Couldn't create {}: {}", path_display, why.to_string()),
            Ok(file) => file,
        };
        match file.write_all(log_filters_str.as_bytes()) {
            Err(why) => panic!("Couldn't write to {}: {}", path_display, why.to_string()),
            Ok(_) => println!("Successfully wrote to {}", path_display),
        }
    }

    pub fn to_string(&self) -> String {
        let mut filters_string: String = String::new();
        for filter in &self.filters {
            // Vec<Vec<String>> -> Vec<String>
            let word_alternatives: Vec<String> = filter
                .iter()
                .map(|s| "[".to_string() + &s.join(",") + "]")
                .collect();
            filters_string += &word_alternatives.join(",");
            filters_string += ",\n";
        }
        filters_string.pop();
        filters_string.pop();

        filters_string
    }

    pub fn load(path: &Path) -> Self {
        let path_display = path.display();
        let mut file = match File::open(&path) {
            Err(why) => panic!("Couldn't open {}: {}", path_display, why.to_string()),
            Ok(file) => file,
        };
        let mut log_filters_str = String::new();
        file.read_to_string(&mut log_filters_str)
            .expect("Could not read from file!");
        let log_filters_lines: Vec<&str> = log_filters_str.split('\n').collect();

        let mut log_filters = LogFilters::load_parameters(&log_filters_lines);
        log_filters.from_str_lines(&log_filters_lines);

        log_filters
    }

    fn load_parameters(log_filters_lines: &[&str]) -> Self {
        if log_filters_lines.len() < 5 {
            panic!(
                "File is corrupted! At least 5 lines expected, found {}",
                log_filters_lines.len()
            )
        }

        let max_allowed_new_alternatives: usize =
            match log_filters_lines[0].to_string().parse::<usize>() {
                Err(why) => panic!(
                    "Couldn't parse 1st line of input to `usize`: {}, {}",
                    log_filters_lines[0],
                    why.to_string()
                ),
                Ok(value) => value,
            };

        let denote_optional: String;
        denote_optional = log_filters_lines[1].to_string();
        if denote_optional.is_empty() {
            panic!("2nd line of input cannot be empty!");
        }

        let ignore_numeric_words: bool = match log_filters_lines[2].to_string().parse::<bool>() {
            Err(why) => panic!(
                "Couldn't parse 3rd line of input to `bool`: {}, {}",
                log_filters_lines[2],
                why.to_string()
            ),
            Ok(value) => value,
        };

        let ignore_first_columns: usize = match log_filters_lines[3].to_string().parse::<usize>() {
            Err(why) => panic!(
                "Couldn't parse 4th line of input to `usize`: {}, {}",
                log_filters_lines[3],
                why.to_string()
            ),
            Ok(value) => value,
        };

        LogFilters {
            filters: Vec::new(),
            words_hash: HashMap::new(),
            max_allowed_new_alternatives,
            denote_optional,
            ignore_numeric_words,
            ignore_first_columns,
        }
    }

    fn from_str_lines(&mut self, log_filters_lines: &[&str]) {
        for line in log_filters_lines {
            if !line.contains('[') || !line.contains(']') {
                continue;
            }
            let mut alternatives = Vec::new();
            let mut include_in_hash = Vec::new();
            let alts_iter = line
                .split(|c| c == '[' || c == ']')
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty() && s != ",");
            for alternative in alts_iter {
                let words: Vec<String> = alternative
                    .split(',')
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                include_in_hash.extend(words.clone());
                alternatives.push(words);
            }
            self.filters.push(alternatives);
            let last_filter_index = self.filters.len() - 1;
            for word in include_in_hash {
                if word.is_empty() || word == self.denote_optional {
                    continue;
                }
                self.update_hash(&word, last_filter_index)
            }
        }
    }

    pub fn print(&self) {
        if !self.filters.is_empty() {
            for elem in &self.filters {
                println!("{:?}", elem);
            }
        } else {
            println!("No filters added yet");
        }
        println!();
        if !self.words_hash.is_empty() {
            let keys: &Vec<&String> = &self.words_hash.keys().collect();
            let mut keys = keys.clone();
            keys.sort();
            for key in keys {
                println!("{} : {:?}", key, &self.words_hash[key]);
            }
        } else {
            println!("No words with references to filters added yet");
        }
    }

    pub fn is_line_known(&self, log_line: &str) -> bool {
        let words = self.line_to_words(&log_line);
        if self.find_best_matching_filter_index(&words) == -1 {
            return false;
        }

        true
    }

    fn line_to_words(&self, log_line: &str) -> Vec<String> {
        let raw_words = LogFilters::line_split(log_line);
        let mut words = Vec::new();

        let mut i = 0;
        for word in raw_words {
            let word = word.to_string();
            if self.ignore_numeric_words && self.is_word_only_numeric(&word) {
                continue;
            }
            if i < self.ignore_first_columns {
                i += 1;
                continue;
            }
            words.push(word);
        }

        words
    }

    pub fn line_split(log_line: &str) -> Vec<String> {
        log_line
            .split(|c| {
                c == ' '
                    || c == '/'
                    || c == ','
                    || c == '.'
                    || c == ':'
                    || c == '"'
                    || c == '\''
                    || c == '('
                    || c == ')'
                    || c == '{'
                    || c == '}'
                    || c == '['
                    || c == ']'
            })
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn learn_line(&mut self, log_line: &str) {
        let words = self.line_to_words(&log_line);

        let matched_filter_index = self.find_best_matching_filter_index(&words);
        if matched_filter_index >= 0 {
            self.update_filter(&words, matched_filter_index as usize);
        } else {
            self.add_filter(words);
        }
    }

    fn is_word_only_numeric(&self, word: &str) -> bool {
        let chars_are_numeric: Vec<bool> = word
            .chars()
            .map(|c| c == '*' || c == '#' || c.is_numeric())
            .collect();

        !chars_are_numeric.contains(&false)
    }

    fn find_best_matching_filter_index(&self, words: &[String]) -> isize {
        if self.filters.is_empty() || words.is_empty() {
            return -1;
        }

        let mut best_matching_filter_index: isize = -1;
        let mut max_consequent_matches: usize = 0;
        let mut max_consequent_matches_indexes: Vec<usize> = Vec::new();
        for filter_index in self.get_filter_indexes_with_min_req_matches(words) {
            let max_cur_consequent_matches = self.count_consequent_matches(words, filter_index);
            if max_cur_consequent_matches > max_consequent_matches {
                max_consequent_matches = max_cur_consequent_matches;
                best_matching_filter_index = filter_index as isize;
                max_consequent_matches_indexes = Vec::new();
            } else if max_cur_consequent_matches == max_consequent_matches {
                max_consequent_matches_indexes.push(filter_index);
            }
        }
        if max_consequent_matches as isize
            >= words.len() as isize - self.max_allowed_new_alternatives as isize
        {
            if max_consequent_matches_indexes.len() > 1 {
                let mut matching_filters: String = String::new();
                for filter_index in max_consequent_matches_indexes {
                    matching_filters += &format!("{:?}, ", self.filters[filter_index]);
                }
                eprintln!(
                    "More than one matching filter found. Words: {:?}; Filters: {}",
                    &words, &matching_filters
                );
            }
            return best_matching_filter_index;
        }

        -1
    }

    // TODO: decompose below into smaller and simpler methods
    fn get_filter_indexes_with_min_req_matches(&self, words: &[String]) -> Vec<usize> {
        let mut filter_indexes_with_min_req_matches: Vec<usize> = Vec::new();
        let filters_with_words = self.get_sorted_filter_indexes_containing_words(words);
        let mut matches: usize = 0;
        let mut optional_alternatives: usize = 0;
        let mut prev_index: isize = -1;
        let mut last_inserted_index: isize = -1;
        for filter_index in filters_with_words {
            if last_inserted_index == filter_index as isize {
                continue;
            }
            if prev_index != filter_index as isize {
                matches = 1;
                prev_index = filter_index as isize;
                optional_alternatives = 0;
                for word_alternatives in &self.filters[filter_index] {
                    if word_alternatives.contains(&self.denote_optional) {
                        optional_alternatives += 1;
                    }
                }
            } else {
                matches += 1;
            }

            if matches as isize >= words.len() as isize - self.max_allowed_new_alternatives as isize
                && matches as isize
                    >= self.filters[filter_index].len() as isize
                        - self.max_allowed_new_alternatives as isize
                        - optional_alternatives as isize
            {
                matches = 0;
                filter_indexes_with_min_req_matches.push(filter_index);
                last_inserted_index = filter_index as isize;
            }
        }

        filter_indexes_with_min_req_matches
    }

    fn get_sorted_filter_indexes_containing_words(&self, words: &[String]) -> Vec<usize> {
        let mut filters_with_words: Vec<usize> = Vec::new();
        for word in words {
            if self.words_hash.get(word).is_some() {
                let vector_indexes = &self.words_hash[word];
                filters_with_words.extend(vector_indexes);
            }
        }
        filters_with_words.sort();

        filters_with_words
    }

    fn count_consequent_matches(&self, words: &[String], filter_index: usize) -> usize {
        if self.filters.len() <= filter_index || words.is_empty() {
            return 0;
        }
        let mut consequent_matches: usize = 0;
        let mut max_consequent_matches: usize = 0;
        let mut new_alternatives: usize = 0;

        let mut extra_allowed_new_alternatives: usize = 0;
        let filter_length = self.filters[filter_index].len();
        if filter_length < words.len() {
            extra_allowed_new_alternatives = words.len() - filter_length;
        }

        let mut last_matching_index: isize = -1;
        for word in words {
            let mathing_index = self.get_word_index_in_filter(
                word,
                filter_index,
                (last_matching_index + 1) as usize,
            );
            if mathing_index >= 0 && mathing_index > last_matching_index {
                last_matching_index = mathing_index;
                consequent_matches += 1;
                if consequent_matches > max_consequent_matches {
                    max_consequent_matches = consequent_matches;
                }
            } else {
                new_alternatives += 1;
                if new_alternatives
                    > self.max_allowed_new_alternatives + extra_allowed_new_alternatives
                {
                    return 0;
                }
            }
        }

        max_consequent_matches
    }

    fn get_word_index_in_filter(
        &self,
        word: &str,
        filter_index: usize,
        start_from_word: usize,
    ) -> isize {
        if word.is_empty() {
            return -1;
        }
        if self.words_hash.get(word).is_none() {
            return -1;
        }
        if !&self.words_hash[word].contains(&filter_index) {
            return -1;
        }
        let filter = self.filters.get(filter_index);
        if filter.is_none() {
            return -1;
        }
        let filter = filter.unwrap();
        if filter.is_empty() || filter.len() - 1 < start_from_word {
            return -1;
        }

        for (word_alternative_index, word_alternative) in
            filter.iter().enumerate().skip(start_from_word)
        {
            if word_alternative.contains(&word.to_owned()) {
                return word_alternative_index as isize;
            }
        }

        -1
    }

    // TODO: decompose below into smaller and simpler methods
    fn update_filter(&mut self, words: &[String], filter_index: usize) {
        let mut indexes = self.normalise_lengths_before_first_match(&words, filter_index, 0, 0);
        while indexes.0 >= 0 && indexes.1 >= 0 && words.len() > indexes.0 as usize {
            let new_indexes = self.normalise_lengths_before_first_match(
                &words,
                filter_index,
                indexes.0 as usize,
                indexes.1 as usize,
            );
            if new_indexes.0 == -1 || new_indexes.1 == -1 {
                break;
            }
            if new_indexes.0 != indexes.0 || new_indexes.1 != indexes.1 {
                indexes = new_indexes;
            } else {
                if indexes.0 == words.len() as isize - 1 {
                    break;
                }
                if indexes.1 == self.filters[filter_index].len() as isize - 1 {
                    break;
                }
                indexes.0 += 1;
                indexes.1 += 1;
            }
        }
        if indexes.0 >= 0 && indexes.1 >= 0 {
            let filter_length = { self.filters[filter_index].len() };
            if words.len() > filter_length && indexes.1 == filter_length as isize - 1 {
                for extra_word in 0..words.len() - filter_length {
                    {
                        let filter = &mut self.filters[filter_index];
                        filter.push(vec![
                            words[filter_length + extra_word].clone(),
                            self.denote_optional.clone(),
                        ]);
                    }
                    self.update_hash(&words[filter_length + extra_word].clone(), filter_index);
                }
            } else if indexes.0 < words.len() as isize {
                let mut reversed_words = words.to_owned();
                reversed_words.reverse();
                self.filters[filter_index].reverse();
                self.normalise_lengths_before_first_match(&reversed_words, filter_index, 0, 0);
                self.filters[filter_index].reverse();
            }
        }
    }

    // TODO: decompose below into smaller and simpler methods
    fn normalise_lengths_before_first_match(
        &mut self,
        words: &[String],
        filter_index: usize,
        word_start_index: usize,
        filter_start_index: usize,
    ) -> (isize, isize) {
        // returns first index after normalised filter slice
        let (first_word, first_filter) = self.get_indexes_of_earliest_matching_word(
            &words,
            filter_index,
            word_start_index,
            filter_start_index,
        );
        if first_word < 0 || first_filter < 0 {
            return (-1, -1);
        }
        let filters_offset = filter_start_index as isize - word_start_index as isize;
        if first_word + filters_offset > first_filter {
            let mut front_words = Vec::new();
            let mut updates: isize = 0;
            for word in &words[word_start_index..first_word as usize] {
                front_words.push(vec![word.clone(), self.denote_optional.clone()]);
                updates += 1;
            }
            // TODO: check if below can be done in more elegant way
            {
                let first_filter = first_filter as usize;
                let filter = &mut self.filters[filter_index];
                filter.splice(first_filter..first_filter, front_words);
            }
            for word in &words[word_start_index..first_word as usize] {
                self.update_hash(&word, filter_index);
            }

            (first_word, first_filter + updates)
        } else {
            {
                // Mark first filter columns as optional alternatives
                let filter = &mut self.filters[filter_index];
                for word_alternatives in filter
                    .iter_mut()
                    .take(
                        (filter_start_index as isize + first_filter - first_word - filters_offset)
                            as usize,
                    )
                    .skip(filter_start_index)
                {
                    if !word_alternatives.contains(&self.denote_optional) {
                        word_alternatives.push(self.denote_optional.clone());
                    }
                }
                // Add new alternatives if filter length before first match was longer than words index
                let mut word_index: usize = word_start_index;
                for word_alternatives in filter.iter_mut().take(first_filter as usize).skip(
                    (filter_start_index as isize + first_filter - first_word - filters_offset)
                        as usize,
                ) {
                    if !word_alternatives.contains(&words[word_index]) {
                        word_alternatives.push(words[word_index].clone());
                    }
                    word_index += 1;
                }
            }
            for word in words
                .iter()
                .take(first_word as usize)
                .skip(word_start_index)
            {
                self.update_hash(&word, filter_index);
            }

            (first_word, first_filter)
        }
    }

    fn get_indexes_of_earliest_matching_word(
        &self,
        words: &[String],
        filter_index: usize,
        word_start_index: usize,
        filter_start_index: usize,
    ) -> (isize, isize) {
        if words.len() as isize - 1 < word_start_index as isize
            || self.filters.get(filter_index).is_none()
        {
            return (-1, -1);
        }
        if self.filters[filter_index].len() as isize - 1 < filter_start_index as isize {
            return (-1, -1);
        }

        let filters_offset = filter_start_index as isize - word_start_index as isize;
        let mut first_matching_word: isize = -1;
        let mut first_matching_filter: isize = -1;
        for (word_index, word) in words.iter().enumerate().skip(word_start_index) {
            let matching_filter_index = self.get_word_index_in_filter(
                &word,
                filter_index,
                (word_start_index as isize + filters_offset) as usize,
            );
            if matching_filter_index >= 0
                && (first_matching_filter == -1 || matching_filter_index < first_matching_filter)
            {
                first_matching_filter = matching_filter_index;
                first_matching_word = word_index as isize;
            }
        }

        (first_matching_word, first_matching_filter)
    }

    fn add_filter(&mut self, words: Vec<String>) {
        let mut new_filter = Vec::new();
        let expected_index: usize = self.filters.len();

        for word in words {
            if !word.is_empty() {
                new_filter.push(vec![word]);
            }
        }
        if !new_filter.is_empty() {
            self.filters.push(new_filter.clone());
            for word_alternatives in new_filter {
                self.update_hash(&word_alternatives[0], expected_index);
            }
        }
    }

    fn update_hash(&mut self, word: &str, filter_index: usize) {
        if self.is_word_in_filter(word, filter_index) {
            self.words_hash
                .entry(word.to_owned())
                .or_insert(vec![filter_index]);
            let vector_indexes = self.words_hash.get_mut(word).unwrap();
            if !vector_indexes.contains(&filter_index) {
                vector_indexes.push(filter_index);
                vector_indexes.sort();
            }
        }
    }

    fn is_word_in_filter(&self, word: &str, filter_index: usize) -> bool {
        let filter = self.filters.get(filter_index);
        if filter.is_none() {
            return false;
        }

        let filter = filter.unwrap();
        for word_alternatives in filter {
            if word_alternatives.contains(&word.to_owned()) {
                return true;
            }
        }

        false
    }
}

#[cfg(feature = "tst_utils")]
pub mod tst_utils {
    use super::*;

    pub fn _words_vector_from_string(words: &str) -> Vec<String> {
        LogFilters::line_split(words)
    }

    pub fn _simple_filter_from_string(words: &str) -> Vec<Vec<String>> {
        let words_vec = LogFilters::line_split(words);

        let mut filter = Vec::new();
        for word in words_vec {
            filter.push(vec![word.to_string()]);
        }
        return filter;
    }

    pub fn _add_word_alternative(
        mut filter: Vec<Vec<String>>,
        index: usize,
        word: &str,
    ) -> Vec<Vec<String>> {
        if filter.get(index).is_some() {
            filter.get_mut(index).unwrap().push(word.to_string());
            return filter;
        } else {
            panic!(
                "Failed to create test data! Extending {:?} at {}",
                filter, index
            );
        }
    }

    pub fn _add_test_filter(test_filters: &mut LogFilters, filter: Vec<Vec<String>>) {
        let next_filter_index = test_filters.filters.len();
        for word_alternatives in &filter {
            for word in word_alternatives {
                if test_filters.words_hash.get(word).is_some() {
                    let filter_indexes = test_filters.words_hash.get_mut(word).unwrap();
                    if !filter_indexes.contains(&next_filter_index) {
                        filter_indexes.push(next_filter_index);
                    }
                } else {
                    test_filters
                        .words_hash
                        .insert(word.clone(), vec![next_filter_index]);
                }
            }
        }
        test_filters.filters.push(filter);
    }

    pub fn _init_test_data() -> LogFilters {
        let mut log_filters = LogFilters::new();
        let mut complex_filter = _simple_filter_from_string("aaa qqq ccc sss");
        complex_filter = _add_word_alternative(complex_filter, 1, "bbb");
        complex_filter = _add_word_alternative(complex_filter, 2, "rrr");
        complex_filter = _add_word_alternative(complex_filter, 3, "ddd");
        _add_test_filter(&mut log_filters, complex_filter);
        _add_test_filter(
            &mut log_filters,
            _simple_filter_from_string("eee fff ggg hhh x y z"),
        );
        _add_test_filter(
            &mut log_filters,
            _simple_filter_from_string("iii jjj kkk lll"),
        );
        _add_test_filter(
            &mut log_filters,
            _simple_filter_from_string("mmm nnn ooo ppp"),
        );
        complex_filter = _simple_filter_from_string("qqq rrr sss ttt");
        complex_filter = _add_word_alternative(complex_filter, 3, "aaa");
        _add_test_filter(&mut log_filters, complex_filter);
        _add_test_filter(
            &mut log_filters,
            _simple_filter_from_string("ttt aaa uuu bbb ccc ddd vvv"),
        );
        return log_filters;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_split() {
        // Test if string will be splitted correctly (single separators)
        let line_1 = "a b/c,d.e:f\"g\'h(i)j{k}l[m]n";
        let result = vec![
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n",
        ];
        assert_eq!(LogFilters::line_split(&line_1), result);

        // Test if string will be splitted correctly (multiple separators)
        let line_2 = " /,.a:\"\'()b{}[]";
        let result = vec!["a", "b"];
        assert_eq!(LogFilters::line_split(&line_2), result);

        // Empty string expected if line consisting of only separators
        let line_3 = " /,.:\"\'(){}[]";
        let result: Vec<String> = Vec::new();
        assert_eq!(LogFilters::line_split(&line_3), result);

        let line_4 = "";
        let result: Vec<String> = Vec::new();
        assert_eq!(LogFilters::line_split(&line_4), result);

        let line_5 = "LoremIpsum";
        let result = vec!["LoremIpsum"];
        assert_eq!(LogFilters::line_split(&line_5), result);
    }

    #[test]
    fn line_to_words() {
        let mut log_filters = LogFilters::new();
        log_filters.ignore_numeric_words = false;
        log_filters.ignore_first_columns = 0;

        // Test if string will be splitted correctly (single separators)
        let line_1 = "a b/c,d.e:f\"g\'h(i)j{k}l[m]n";
        let result = vec![
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n",
        ];
        assert_eq!(log_filters.line_to_words(&line_1), result);

        // Test if string will be splitted correctly (multiple separators)
        let line_2 = " /,.a:\"\'()b{}[]";
        let result = vec!["a", "b"];
        assert_eq!(log_filters.line_to_words(&line_2), result);

        // Empty string expected if line consisting of only separators
        let line_3 = " /,.:\"\'(){}[]";
        let result: Vec<String> = Vec::new();
        assert_eq!(log_filters.line_to_words(&line_3), result);

        let line_4 = "";
        let result: Vec<String> = Vec::new();
        assert_eq!(log_filters.line_to_words(&line_4), result);

        // Test if string will be splitted correctly (no separators)
        let line_5 = "LoremIpsum";
        let result = vec!["LoremIpsum"];
        assert_eq!(log_filters.line_to_words(&line_5), result);

        // Following tests for LogFilters::ignore_first_columns parameter set to `2`
        let mut log_filters = LogFilters::new();
        log_filters.ignore_numeric_words = false;
        log_filters.ignore_first_columns = 2;

        // Test if string will be splitted correctly (single separators)
        let line_1 = "a b/c,d.e:f\"g\'h(i)j{k}l[m]n";
        let result = vec!["c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n"];
        assert_eq!(log_filters.line_to_words(&line_1), result);

        // Test if string will be splitted correctly (multiple separators)
        let line_2 = " /,.a:\"\'()b{}[]c[]{}.,";
        let result = vec!["c"];
        assert_eq!(log_filters.line_to_words(&line_2), result);

        // Empty string expected if line consisting of only separators
        let line_3 = " /,.:\"\'(){}[]";
        let result: Vec<String> = Vec::new();
        assert_eq!(log_filters.line_to_words(&line_3), result);

        let line_4 = "";
        let result: Vec<String> = Vec::new();
        assert_eq!(log_filters.line_to_words(&line_4), result);

        // First two words should be removed, numeric word should stay
        let line_5 = "Lorem ipsum dolor sit amet, 123 consectetur adipiscing elit7";
        let result = vec![
            "dolor",
            "sit",
            "amet",
            "123",
            "consectetur",
            "adipiscing",
            "elit7",
        ];
        assert_eq!(log_filters.line_to_words(&line_5), result);

        // Test if numeric words will be ignored
        let mut log_filters = LogFilters::new();
        log_filters.ignore_numeric_words = true;
        log_filters.ignore_first_columns = 2;

        // First two words and numeric word should be removed
        let line_5 = "Lorem ipsum dolor sit amet, 123 consectetur adipiscing elit7";
        let result = vec!["dolor", "sit", "amet", "consectetur", "adipiscing", "elit7"];
        assert_eq!(log_filters.line_to_words(&line_5), result);
    }

    #[test]
    fn to_string() {
        // TODO: cover incorrect input
        let mut log_filters = LogFilters::new();
        assert_eq!(log_filters.to_string(), "");

        // One filter with no alternatives
        tst_utils::_add_test_filter(
            &mut log_filters,
            tst_utils::_simple_filter_from_string("aaa bbb ccc ddd"),
        );
        let filter_1: String = "[aaa],[bbb],[ccc],[ddd]".to_string();
        let result = filter_1.clone();
        assert_eq!(log_filters.to_string(), result);

        // Two filters with no alternatives
        tst_utils::_add_test_filter(
            &mut log_filters,
            tst_utils::_simple_filter_from_string("xxx yyy zzz"),
        );
        let filter_2: String = "[xxx],[yyy],[zzz]".to_string();
        let result = filter_1.clone() + ",\n" + &filter_2;
        assert_eq!(log_filters.to_string(), result);

        // Three filters, third filter with alternatives
        let mut complex_filter = tst_utils::_simple_filter_from_string("eee fff ggg hhh");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 1, "iii");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 1, "jjj");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 3, ".");
        tst_utils::_add_test_filter(&mut log_filters, complex_filter);
        let filter_3: String = "[eee],[fff,iii,jjj],[ggg],[hhh,.]".to_string();
        let result = filter_1.clone() + ",\n" + &filter_2 + ",\n" + &filter_3;
        assert_eq!(log_filters.to_string(), result);
    }

    #[test]
    fn load_parameters() {
        // TODO: cover incorrect input
        let log_filters_lines = vec!["2", ".", "true", "2", "0"];
        let log_filters = LogFilters::load_parameters(&log_filters_lines);
        assert_eq!(log_filters.max_allowed_new_alternatives, 2);
        assert_eq!(log_filters.denote_optional, ".");
        assert_eq!(log_filters.ignore_numeric_words, true);
        assert_eq!(log_filters.ignore_first_columns, 2);
    }

    #[test]
    fn from_str_lines() {
        // TODO: cover incorrect input
        // Filter with no alternatives
        let log_filters_lines = vec!["[a],[b],[c],[d],[e]"];
        let mut log_filters = LogFilters::new();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.ignore_numeric_words = true;
        log_filters.ignore_first_columns = 2;
        log_filters.from_str_lines(&log_filters_lines);
        assert_eq!(log_filters.filters.len(), 1);
        let expected = tst_utils::_simple_filter_from_string("a b c d e");
        assert_eq!(log_filters.filters[0], expected);
        assert_eq!(
            log_filters.words_hash.get(&"a".to_string()).unwrap(),
            &vec![0 as usize]
        );
        assert_eq!(
            log_filters.words_hash.get(&"b".to_string()).unwrap(),
            &vec![0 as usize]
        );
        assert_eq!(
            log_filters.words_hash.get(&"c".to_string()).unwrap(),
            &vec![0 as usize]
        );
        assert_eq!(
            log_filters.words_hash.get(&"d".to_string()).unwrap(),
            &vec![0 as usize]
        );
        assert_eq!(
            log_filters.words_hash.get(&"e".to_string()).unwrap(),
            &vec![0 as usize]
        );

        // Filter with alternatives
        let log_filters_lines = vec!["[a,b],[c],[d,e]"];
        let mut log_filters = LogFilters::new();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.ignore_numeric_words = true;
        log_filters.ignore_first_columns = 2;
        log_filters.from_str_lines(&log_filters_lines);
        assert_eq!(log_filters.filters.len(), 1);
        let mut expected = tst_utils::_simple_filter_from_string("a c d");
        expected = tst_utils::_add_word_alternative(expected, 0, "b");
        expected = tst_utils::_add_word_alternative(expected, 2, "e");
        assert_eq!(log_filters.filters[0], expected);
        assert_eq!(
            log_filters.words_hash.get(&"a".to_string()).unwrap(),
            &vec![0 as usize]
        );
        assert_eq!(
            log_filters.words_hash.get(&"b".to_string()).unwrap(),
            &vec![0 as usize]
        );
        assert_eq!(
            log_filters.words_hash.get(&"c".to_string()).unwrap(),
            &vec![0 as usize]
        );
        assert_eq!(
            log_filters.words_hash.get(&"d".to_string()).unwrap(),
            &vec![0 as usize]
        );
        assert_eq!(
            log_filters.words_hash.get(&"e".to_string()).unwrap(),
            &vec![0 as usize]
        );

        // Two filters
        let log_filters_lines = vec!["[a],[b],[c],[d],[e,f]", "[a,b],[c],[d,e,g]"];
        let mut log_filters = LogFilters::new();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.ignore_numeric_words = true;
        log_filters.ignore_first_columns = 2;
        log_filters.from_str_lines(&log_filters_lines);
        assert_eq!(log_filters.filters.len(), 2);
        let mut expected_1 = tst_utils::_simple_filter_from_string("a b c d e");
        expected_1 = tst_utils::_add_word_alternative(expected_1, 4, "f");
        let mut expected_2 = tst_utils::_simple_filter_from_string("a c d");
        expected_2 = tst_utils::_add_word_alternative(expected_2, 0, "b");
        expected_2 = tst_utils::_add_word_alternative(expected_2, 2, "e");
        expected_2 = tst_utils::_add_word_alternative(expected_2, 2, "g");
        assert_eq!(log_filters.filters[0], expected_1);
        assert_eq!(log_filters.filters[1], expected_2);
        assert_eq!(
            log_filters.words_hash.get(&"a".to_string()).unwrap(),
            &vec![0 as usize, 1 as usize]
        );
        assert_eq!(
            log_filters.words_hash.get(&"b".to_string()).unwrap(),
            &vec![0 as usize, 1 as usize]
        );
        assert_eq!(
            log_filters.words_hash.get(&"c".to_string()).unwrap(),
            &vec![0 as usize, 1 as usize]
        );
        assert_eq!(
            log_filters.words_hash.get(&"d".to_string()).unwrap(),
            &vec![0 as usize, 1 as usize]
        );
        assert_eq!(
            log_filters.words_hash.get(&"e".to_string()).unwrap(),
            &vec![0 as usize, 1 as usize]
        );
        assert_eq!(
            log_filters.words_hash.get(&"f".to_string()).unwrap(),
            &vec![0 as usize]
        );
        assert_eq!(
            log_filters.words_hash.get(&"g".to_string()).unwrap(),
            &vec![1 as usize]
        );
    }

    #[test]
    fn is_word_only_numeric() {
        let log_filters = LogFilters::new();
        assert_eq!(log_filters.is_word_only_numeric(&"asdf".to_string()), false);
        assert_eq!(log_filters.is_word_only_numeric(&"123a".to_string()), false);
        assert_eq!(log_filters.is_word_only_numeric(&"a123".to_string()), false);
        assert_eq!(log_filters.is_word_only_numeric(&"6789".to_string()), true);
        assert_eq!(log_filters.is_word_only_numeric(&"*6789".to_string()), true);
        assert_eq!(log_filters.is_word_only_numeric(&"#6789".to_string()), true);
        assert_eq!(
            log_filters.is_word_only_numeric(&"6789*6789".to_string()),
            true
        );
        assert_eq!(
            log_filters.is_word_only_numeric(&"6789#6789".to_string()),
            true
        );
        assert_eq!(log_filters.is_word_only_numeric(&"".to_string()), true);
    }

    #[test]
    fn find_best_matching_filter_index() {
        let log_filters = LogFilters::new();
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc ddd");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), -1);

        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        // Empty words vector should result in invalid index
        let words = vec![];
        assert_eq!(log_filters.find_best_matching_filter_index(&words), -1);
        // First full match should be returned
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc ddd");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), 0);
        // If words vector is shorter than filter then first fully matching filter should be returned
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), 0);
        let words = tst_utils::_words_vector_from_string("aaa bbb");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), -1);
        log_filters.max_allowed_new_alternatives = 2;
        assert_eq!(log_filters.find_best_matching_filter_index(&words), 0);
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("aaa");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), -1);
        log_filters.max_allowed_new_alternatives = 2;
        assert_eq!(log_filters.find_best_matching_filter_index(&words), -1);
        log_filters.max_allowed_new_alternatives = 3;
        assert_eq!(log_filters.find_best_matching_filter_index(&words), 0);
        // Test if 1 word alternative is allowed
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc xxx");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), 0);
        let words = tst_utils::_words_vector_from_string("aaa xxx ccc ddd");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), 0);
        // Two and more new alternatives should result in incorrect index
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("aaa bbb zzz xxx");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), -1);
        let words = tst_utils::_words_vector_from_string("aaa xxx zzz ddd");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), -1);
        // Test if words vector can be longer than existing filter
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc ddd eee");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), 0);
        log_filters.max_allowed_new_alternatives = 2;
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc ddd eee fff");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), 0);
        // Test if longer words vector will be allowed to contain 1 word alternative to existing word
        log_filters.max_allowed_new_alternatives = 2;
        let words = tst_utils::_words_vector_from_string("aaa xxx ccc ddd eee");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), 0);
        // Test if longer words vector will be allowed to contain 1 new word alternative
        log_filters.max_allowed_new_alternatives = 2;
        let words = tst_utils::_words_vector_from_string("aaa xxx bbb ccc ddd eee");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), 0);
        // Test if words vector and filter vector must contain words in the same order
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("ddd ccc bbb aaa");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), -1);
        let words = tst_utils::_words_vector_from_string("ccc bbb aaa");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), -1);
        // Test for shorter word
        log_filters.max_allowed_new_alternatives = 0;
        let words = tst_utils::_words_vector_from_string("bbb aaa");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), -1);
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("bbb aaa");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), -1);
        log_filters.max_allowed_new_alternatives = 3;
        let words = tst_utils::_words_vector_from_string("bbb aaa");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), 0);
        // Test situation where there are more optional alternatives than max_allowed_new_alternatives
        let mut log_filters = LogFilters::new();
        log_filters.max_allowed_new_alternatives = 0;
        let mut complex_filter =
            tst_utils::_simple_filter_from_string("eee fff ggg hhh iii jjj kkk lll");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 4, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 5, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 6, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 7, ".");
        tst_utils::_add_test_filter(&mut log_filters, complex_filter);
        let words = tst_utils::_words_vector_from_string("eee fff ggg hhh");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), 0);
        // Test situation where there are only optional alternatives
        let mut log_filters = LogFilters::new();
        log_filters.max_allowed_new_alternatives = 0;
        let mut complex_filter =
            tst_utils::_simple_filter_from_string("eee fff ggg hhh iii jjj kkk lll");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 0, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 1, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 2, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 3, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 4, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 5, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 6, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 7, ".");
        tst_utils::_add_test_filter(&mut log_filters, complex_filter);
        let words = tst_utils::_words_vector_from_string("mmm nnn ooo ppp");
        assert_eq!(log_filters.find_best_matching_filter_index(&words), -1);

        // TODO: more unit-tests to cover edge cases for max_allowed_new_alternatives
    }

    #[test]
    fn get_filter_indexes_with_min_req_matches() {
        // Test what happens if method was used on empty data structure
        let log_filters = LogFilters::new();
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&vec![]),
            vec![]
        );
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc ddd");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![]
        );

        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        assert_eq!(
            log_filters.get_sorted_filter_indexes_containing_words(&vec![]),
            vec![]
        );
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc ddd");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![0]
        );
        // Test when words length is less than self.min_req_consequent_matches
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("aaa bbb");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![]
        );
        log_filters.max_allowed_new_alternatives = 2;
        let words = tst_utils::_words_vector_from_string("aaa bbb");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![0]
        );
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("aaa");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![]
        );
        log_filters.max_allowed_new_alternatives = 2;
        let words = tst_utils::_words_vector_from_string("aaa");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![]
        );
        log_filters.max_allowed_new_alternatives = 3;
        let words = tst_utils::_words_vector_from_string("aaa");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![0, 4]
        );
        // But empty words vector is still not allowed
        log_filters.max_allowed_new_alternatives = 1;
        let words = vec![];
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![]
        );
        // One-word words vector will only match if at least one filter contains that word
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("xyz");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![]
        );
        // Test when new word alternatives are required
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("aaa lll ccc ddd");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![0]
        );
        // Test when new word alternative is required and words vector is shorter than filter
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("aaa lll ccc");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![]
        );
        log_filters.max_allowed_new_alternatives = 2;
        let words = tst_utils::_words_vector_from_string("aaa lll ccc");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![0]
        );
        // We are not counting consequent matches here, max_allowed_new_alternatives
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("aaa lll zzz ddd");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![]
        );
        log_filters.max_allowed_new_alternatives = 2;
        let words = tst_utils::_words_vector_from_string("aaa lll zzz ddd");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![0]
        );
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("aaa lll zzz yyy ddd");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![]
        );
        log_filters.max_allowed_new_alternatives = 3;
        let words = tst_utils::_words_vector_from_string("aaa lll zzz yyy ddd");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![0]
        );
        // We are not checking for correct words order here
        log_filters.max_allowed_new_alternatives = 1;
        let words = tst_utils::_words_vector_from_string("ddd lll zzz yyy aaa");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![]
        );
        log_filters.max_allowed_new_alternatives = 2;
        let words = tst_utils::_words_vector_from_string("ddd lll zzz yyy aaa");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![]
        );
        log_filters.max_allowed_new_alternatives = 3;
        let words = tst_utils::_words_vector_from_string("ddd lll zzz yyy aaa");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![0]
        );
        // Test situation where there are more optional alternatives than max_allowed_new_alternatives
        let mut log_filters = LogFilters::new();
        log_filters.max_allowed_new_alternatives = 0;
        let mut complex_filter =
            tst_utils::_simple_filter_from_string("eee fff ggg hhh iii jjj kkk lll");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 4, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 5, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 6, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 7, ".");
        tst_utils::_add_test_filter(&mut log_filters, complex_filter);
        let words = tst_utils::_words_vector_from_string("eee fff ggg hhh");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![0]
        );
        // Test situation where there are only optional alternatives
        let mut log_filters = LogFilters::new();
        log_filters.max_allowed_new_alternatives = 0;
        let mut complex_filter =
            tst_utils::_simple_filter_from_string("eee fff ggg hhh iii jjj kkk lll");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 0, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 1, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 2, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 3, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 4, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 5, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 6, ".");
        complex_filter = tst_utils::_add_word_alternative(complex_filter, 7, ".");
        tst_utils::_add_test_filter(&mut log_filters, complex_filter);
        let words = tst_utils::_words_vector_from_string("mmm nnn ooo ppp");
        assert_eq!(
            log_filters.get_filter_indexes_with_min_req_matches(&words),
            vec![]
        );

        // TODO: more unit-tests to cover edge cases for max_allowed_new_alternatives
    }

    #[test]
    fn get_sorted_filter_indexes_containing_words() {
        let log_filters = LogFilters::new();
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc ddd");
        assert_eq!(
            log_filters.get_sorted_filter_indexes_containing_words(&words),
            vec![]
        );
        assert_eq!(
            log_filters.get_sorted_filter_indexes_containing_words(&vec![]),
            vec![]
        );

        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        assert_eq!(
            log_filters.get_sorted_filter_indexes_containing_words(&vec![]),
            vec![]
        );
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc ddd");
        assert_eq!(
            log_filters.get_sorted_filter_indexes_containing_words(&words),
            vec![0, 0, 0, 0, 4, 5, 5, 5, 5]
        );
        let words = tst_utils::_words_vector_from_string("aaa xxx");
        assert_eq!(
            log_filters.get_sorted_filter_indexes_containing_words(&words),
            vec![0, 4, 5]
        );
        let words = tst_utils::_words_vector_from_string("xxx");
        assert_eq!(
            log_filters.get_sorted_filter_indexes_containing_words(&words),
            vec![]
        );
    }

    #[test]
    fn count_consequent_matches() {
        // Test what happens if method was used on empty data structure
        let mut log_filters = LogFilters::new();
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc ddd");
        assert_eq!(log_filters.count_consequent_matches(&words, 0), 0);
        assert_eq!(log_filters.count_consequent_matches(&words, 1), 0);
        assert_eq!(log_filters.count_consequent_matches(&vec![], 0), 0);
        log_filters.max_allowed_new_alternatives = 0;
        assert_eq!(log_filters.count_consequent_matches(&words, 0), 0);
        assert_eq!(log_filters.count_consequent_matches(&words, 1), 0);
        assert_eq!(log_filters.count_consequent_matches(&vec![], 0), 0);

        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        // Test for existing pattern
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc ddd");
        assert_eq!(log_filters.count_consequent_matches(&words, 0), 4);
        assert_eq!(log_filters.count_consequent_matches(&words, 1), 0);
        // Test out of bounds
        assert_eq!(
            log_filters.count_consequent_matches(&words, log_filters.filters.len()),
            0
        );
        // Test empty words vector
        assert_eq!(log_filters.count_consequent_matches(&vec![], 0), 0);
        // Test if words vector can be smaller than filter
        let words = tst_utils::_words_vector_from_string("iii jjj lll");
        assert_eq!(log_filters.count_consequent_matches(&words, 2), 3);
        let words = tst_utils::_words_vector_from_string("iii lll");
        assert_eq!(log_filters.count_consequent_matches(&words, 2), 2);
        let words = tst_utils::_words_vector_from_string("iii jjj");
        assert_eq!(log_filters.count_consequent_matches(&words, 2), 2);
        let words = tst_utils::_words_vector_from_string("jjj kkk");
        assert_eq!(log_filters.count_consequent_matches(&words, 2), 2);
        let words = tst_utils::_words_vector_from_string("iii");
        assert_eq!(log_filters.count_consequent_matches(&words, 2), 1);
        let words = tst_utils::_words_vector_from_string("jjj");
        assert_eq!(log_filters.count_consequent_matches(&words, 2), 1);
        // Test if word alternative will be matched
        let words = tst_utils::_words_vector_from_string("aaa");
        assert_eq!(log_filters.count_consequent_matches(&words, 4), 1);
        // Test if 1 word alternative is allowed
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc xxx");
        assert_eq!(log_filters.count_consequent_matches(&words, 0), 3);
        let words = tst_utils::_words_vector_from_string("aaa xxx ccc ddd");
        assert_eq!(log_filters.count_consequent_matches(&words, 0), 3);
        let words = tst_utils::_words_vector_from_string("aaa bbb zzz xxx");
        assert_eq!(log_filters.count_consequent_matches(&words, 0), 0);
        let words = tst_utils::_words_vector_from_string("aaa xxx zzz ddd");
        assert_eq!(log_filters.count_consequent_matches(&words, 0), 0);
        // Test if words vector can be longer than existing filter
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc ddd eee fff ggg hhh");
        assert_eq!(log_filters.count_consequent_matches(&words, 0), 4);
        // Test if longer words vector will be allowed to contain 1 word alternative to existing word
        let words = tst_utils::_words_vector_from_string("aaa xxx ccc ddd eee fff ggg hhh");
        assert_eq!(log_filters.count_consequent_matches(&words, 3), 0);
        // Test if longer words vector will be allowed to contain 1 new word alternative
        let words = tst_utils::_words_vector_from_string("aaa xxx bbb ccc ddd fff ggg hhh");
        assert_eq!(log_filters.count_consequent_matches(&words, 4), 0);
        // Test if words vector and filter vector must contain words in the same order
        let words = tst_utils::_words_vector_from_string("ddd ccc bbb aaa");
        assert_eq!(log_filters.count_consequent_matches(&words, 0), 0);
    }

    #[test]
    fn get_word_index_in_filter() {
        // Test what happens if method was used on empty data structure
        let log_filters = LogFilters::new();
        assert_eq!(
            log_filters.get_word_index_in_filter(&"aaa".to_string(), 0, 0),
            -1
        );
        assert_eq!(
            log_filters.get_word_index_in_filter(&"aaa".to_string(), 0, 100),
            -1
        );
        assert_eq!(
            log_filters.get_word_index_in_filter(&"aaa".to_string(), 100, 0),
            -1
        );
        assert_eq!(
            log_filters.get_word_index_in_filter(&"".to_string(), 0, 0),
            -1
        );

        let log_filters = tst_utils::_init_test_data();
        // Test if word will be matched when it should be
        assert_eq!(
            log_filters.get_word_index_in_filter(&"aaa".to_string(), 0, 0),
            0
        );
        assert_eq!(
            log_filters.get_word_index_in_filter(&"aaa".to_string(), 4, 0),
            3
        );
        assert_eq!(
            log_filters.get_word_index_in_filter(&"qqq".to_string(), 0, 0),
            1
        );
        assert_eq!(
            log_filters.get_word_index_in_filter(&"sss".to_string(), 0, 3),
            3
        );
        assert_eq!(
            log_filters.get_word_index_in_filter(&"ddd".to_string(), 0, 3),
            3
        );
        // Test if word will not be matched if starting index is higher than word index in filter
        assert_eq!(
            log_filters.get_word_index_in_filter(&"aaa".to_string(), 0, 1),
            -1
        );
        // Empty string should not be matched
        assert_eq!(
            log_filters.get_word_index_in_filter(&"".to_string(), 4, 0),
            -1
        );
        // Test when word does not exist in filter or filter does not exist
        assert_eq!(
            log_filters.get_word_index_in_filter(&"aaa".to_string(), 1, 0),
            -1
        );
        assert_eq!(
            log_filters.get_word_index_in_filter(&"aaa".to_string(), log_filters.filters.len(), 0),
            -1
        );
    }

    #[test]
    fn update_filter() {
        // Test empty data structure
        let mut log_filters = LogFilters::new();
        log_filters.update_filter(&vec![], 0);
        assert_eq!(log_filters.filters.len(), 0);

        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.denote_optional = ".".to_string();
        // Try to update based on empty words vector
        let filter_0_len = log_filters.filters[0].len();
        log_filters.update_filter(&vec![], 0);
        assert_eq!(log_filters.filters[0].len(), filter_0_len);
        // Try to update a filter that does not exist
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc xxx");
        let nonexisting_filter_index = log_filters.filters.len();
        log_filters.update_filter(&words, nonexisting_filter_index);
        // No update required
        let words = tst_utils::_words_vector_from_string("mmm nnn ooo ppp");
        log_filters.update_filter(&words, 3);
        let expected = tst_utils::_simple_filter_from_string("mmm nnn ooo ppp");
        assert_eq!(log_filters.filters.get(3).unwrap(), &expected);

        // One new (hence optional) word alternative added at the front of filter
        let words = tst_utils::_words_vector_from_string("foo qqq rrr sss ttt");
        log_filters.update_filter(&words, 4);
        let mut expected = tst_utils::_simple_filter_from_string("foo qqq rrr sss ttt");
        expected = tst_utils::_add_word_alternative(expected, 0, ".");
        expected = tst_utils::_add_word_alternative(expected, 4, "aaa");
        assert_eq!(log_filters.filters.get(4).unwrap(), &expected);
        assert_eq!(
            log_filters.words_hash.get(&"foo".to_string()).unwrap(),
            &vec![4]
        );
        // Two new (hence optional) word alternatives added at the front of filter
        let words = tst_utils::_words_vector_from_string("xyz qwe mmm nnn ooo ppp");
        log_filters.update_filter(&words, 3);
        let mut expected = tst_utils::_simple_filter_from_string("xyz qwe mmm nnn ooo ppp");
        expected = tst_utils::_add_word_alternative(expected, 0, ".");
        expected = tst_utils::_add_word_alternative(expected, 1, ".");
        assert_eq!(log_filters.filters.get(3).unwrap(), &expected);
        assert_eq!(
            log_filters.words_hash.get(&"xyz".to_string()).unwrap(),
            &vec![3]
        );
        assert_eq!(
            log_filters.words_hash.get(&"qwe".to_string()).unwrap(),
            &vec![3]
        );
        // One word turned to (optional) alternative as a result of words vector shorter than filter
        let words = tst_utils::_words_vector_from_string("fff ggg hhh x y z");
        log_filters.update_filter(&words, 1);
        let mut expected = tst_utils::_simple_filter_from_string("eee fff ggg hhh x y z");
        expected = tst_utils::_add_word_alternative(expected, 0, ".");
        assert_eq!(log_filters.filters.get(1).unwrap(), &expected);
        // Two words turned to (optional) alternatives as a result of words vector shorter than filter
        let words = tst_utils::_words_vector_from_string("kkk lll");
        log_filters.update_filter(&words, 2);
        let mut expected = tst_utils::_simple_filter_from_string("iii jjj kkk lll");
        expected = tst_utils::_add_word_alternative(expected, 0, ".");
        expected = tst_utils::_add_word_alternative(expected, 1, ".");
        assert_eq!(log_filters.filters.get(2).unwrap(), &expected);
        // One word turned to optional alternative and one new alternative added
        let words = tst_utils::_words_vector_from_string("bar ccc sss");
        log_filters.update_filter(&words, 0);
        let mut expected = tst_utils::_simple_filter_from_string("aaa qqq ccc sss");
        expected = tst_utils::_add_word_alternative(expected, 0, ".");
        expected = tst_utils::_add_word_alternative(expected, 1, "bbb");
        expected = tst_utils::_add_word_alternative(expected, 1, "bar");
        expected = tst_utils::_add_word_alternative(expected, 2, "rrr");
        expected = tst_utils::_add_word_alternative(expected, 3, "ddd");
        assert_eq!(log_filters.filters.get(0).unwrap(), &expected);

        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.denote_optional = ".".to_string();
        // Add alternative to one word in the middle
        let words = tst_utils::_words_vector_from_string("iii jjj foo lll");
        log_filters.update_filter(&words, 2);
        let mut expected = tst_utils::_simple_filter_from_string("iii jjj kkk lll");
        expected = tst_utils::_add_word_alternative(expected, 2, "foo");
        assert_eq!(log_filters.filters.get(2).unwrap(), &expected);
        assert_eq!(
            log_filters.words_hash.get(&"foo".to_string()).unwrap(),
            &vec![2]
        );
        // Add alternatives to consequent two words in the middle
        let words = tst_utils::_words_vector_from_string("ttt aaa xyz qwe ccc ddd vvv");
        log_filters.update_filter(&words, 5);
        let mut expected = tst_utils::_simple_filter_from_string("ttt aaa uuu bbb ccc ddd vvv");
        expected = tst_utils::_add_word_alternative(expected, 2, "xyz");
        expected = tst_utils::_add_word_alternative(expected, 3, "qwe");
        assert_eq!(log_filters.filters.get(5).unwrap(), &expected);
        assert_eq!(
            log_filters.words_hash.get(&"xyz".to_string()).unwrap(),
            &vec![5]
        );
        assert_eq!(
            log_filters.words_hash.get(&"qwe".to_string()).unwrap(),
            &vec![5]
        );
        // Add alternatives to two non-consequent words in the middle
        let words = tst_utils::_words_vector_from_string("eee fff bar hhh x baz z");
        log_filters.update_filter(&words, 1);
        let mut expected = tst_utils::_simple_filter_from_string("eee fff ggg hhh x y z");
        expected = tst_utils::_add_word_alternative(expected, 2, "bar");
        expected = tst_utils::_add_word_alternative(expected, 5, "baz");
        assert_eq!(log_filters.filters.get(1).unwrap(), &expected);
        assert_eq!(
            log_filters.words_hash.get(&"bar".to_string()).unwrap(),
            &vec![1]
        );
        assert_eq!(
            log_filters.words_hash.get(&"baz".to_string()).unwrap(),
            &vec![1]
        );

        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.denote_optional = ".".to_string();
        // Turn one word in the middle to optional alternative
        let words = tst_utils::_words_vector_from_string("ttt aaa bbb ccc ddd vvv");
        log_filters.update_filter(&words, 5);
        let mut expected = tst_utils::_simple_filter_from_string("ttt aaa uuu bbb ccc ddd vvv");
        expected = tst_utils::_add_word_alternative(expected, 2, ".");
        assert_eq!(log_filters.filters.get(5).unwrap(), &expected);
        // Turn two non-consequent words in the middle to optional alternatives
        let words = tst_utils::_words_vector_from_string("eee ggg x y z");
        log_filters.update_filter(&words, 1);
        let mut expected = tst_utils::_simple_filter_from_string("eee fff ggg hhh x y z");
        expected = tst_utils::_add_word_alternative(expected, 1, ".");
        expected = tst_utils::_add_word_alternative(expected, 3, ".");
        assert_eq!(log_filters.filters.get(1).unwrap(), &expected);
        // Turn one word in the middle to optional alternative
        let words = tst_utils::_words_vector_from_string("iii jjj lll");
        log_filters.update_filter(&words, 2);
        let mut expected = tst_utils::_simple_filter_from_string("iii jjj kkk lll");
        expected = tst_utils::_add_word_alternative(expected, 2, ".");
        assert_eq!(log_filters.filters.get(2).unwrap(), &expected);

        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.denote_optional = ".".to_string();
        // Last word not matching
        let words = tst_utils::_words_vector_from_string("ttt aaa uuu bbb ccc ddd xyz");
        log_filters.update_filter(&words, 5);
        let mut expected = tst_utils::_simple_filter_from_string("ttt aaa uuu bbb ccc ddd vvv");
        expected = tst_utils::_add_word_alternative(expected, 6, "xyz");
        assert_eq!(log_filters.filters.get(5).unwrap(), &expected);
        assert_eq!(
            log_filters.words_hash.get(&"xyz".to_string()).unwrap(),
            &vec![5]
        );

        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.denote_optional = ".".to_string();
        // Words vector shorter by one word
        let words = tst_utils::_words_vector_from_string("ttt aaa uuu bbb ccc ddd");
        log_filters.update_filter(&words, 5);
        let mut expected = tst_utils::_simple_filter_from_string("ttt aaa uuu bbb ccc ddd vvv");
        expected = tst_utils::_add_word_alternative(expected, 6, ".");
        assert_eq!(log_filters.filters.get(5).unwrap(), &expected);

        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.denote_optional = ".".to_string();
        // Words vector longer by one word
        let words = tst_utils::_words_vector_from_string("ttt aaa uuu bbb ccc ddd vvv xyz");
        log_filters.update_filter(&words, 5);
        let mut expected = tst_utils::_simple_filter_from_string("ttt aaa uuu bbb ccc ddd vvv xyz");
        expected = tst_utils::_add_word_alternative(expected, 7, ".");
        assert_eq!(log_filters.filters.get(5).unwrap(), &expected);
        assert_eq!(
            log_filters.words_hash.get(&"xyz".to_string()).unwrap(),
            &vec![5]
        );
    }

    #[test]
    fn normalise_lengths_before_first_match() {
        // Test empty data structure
        let mut log_filters = LogFilters::new();
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&vec![], 0, 0, 0),
            (-1, -1)
        );
        assert_eq!(log_filters.filters.len(), 0);
        assert_eq!(log_filters.words_hash.len(), 0);
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc xxx");
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&words, 0, 0, 0),
            (-1, -1)
        );
        assert_eq!(log_filters.filters.len(), 0);
        assert_eq!(log_filters.words_hash.len(), 0);

        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.denote_optional = ".".to_string();
        // Try to update based on empty words vector
        let filter_0_len = log_filters.filters[0].len();
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&vec![], 0, 0, 0),
            (-1, -1)
        );
        assert_eq!(log_filters.filters[0].len(), filter_0_len);
        // Try to update a filter that does not exist
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc xxx");
        let nonexisting_filter_index = log_filters.filters.len();
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(
                &words,
                nonexisting_filter_index,
                0,
                0
            ),
            (-1, -1)
        );
        // No update required
        let words = tst_utils::_words_vector_from_string("mmm nnn ooo ppp");
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&words, 3, 0, 0),
            (0, 0)
        );
        let expected = tst_utils::_simple_filter_from_string("mmm nnn ooo ppp");
        assert_eq!(log_filters.filters.get(3).unwrap(), &expected);

        // One new (hence optional) word alternative added at the front of filter
        let words = tst_utils::_words_vector_from_string("foo qqq rrr sss ttt");
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&words, 4, 0, 0),
            (1, 1)
        );
        let mut expected = tst_utils::_simple_filter_from_string("foo qqq rrr sss ttt");
        expected = tst_utils::_add_word_alternative(expected, 0, ".");
        expected = tst_utils::_add_word_alternative(expected, 4, "aaa");
        assert_eq!(log_filters.filters.get(4).unwrap(), &expected);
        assert_eq!(
            log_filters.words_hash.get(&"foo".to_string()).unwrap(),
            &vec![4]
        );
        // Two new (hence optional) word alternatives resulting from passed word vector
        let words = tst_utils::_words_vector_from_string("xyz qwe mmm nnn ooo ppp");
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&words, 3, 0, 0),
            (2, 2)
        );
        let mut expected = tst_utils::_simple_filter_from_string("xyz qwe mmm nnn ooo ppp");
        expected = tst_utils::_add_word_alternative(expected, 0, ".");
        expected = tst_utils::_add_word_alternative(expected, 1, ".");
        assert_eq!(log_filters.filters.get(3).unwrap(), &expected);
        assert_eq!(
            log_filters.words_hash.get(&"xyz".to_string()).unwrap(),
            &vec![3]
        );
        assert_eq!(
            log_filters.words_hash.get(&"qwe".to_string()).unwrap(),
            &vec![3]
        );
        // One word turned to (optional) alternative as a result of words vector shorter than filter
        let words = tst_utils::_words_vector_from_string("fff ggg hhh x y z");
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&words, 1, 0, 0),
            (0, 1)
        );
        let mut expected = tst_utils::_simple_filter_from_string("eee fff ggg hhh x y z");
        expected = tst_utils::_add_word_alternative(expected, 0, ".");
        assert_eq!(log_filters.filters.get(1).unwrap(), &expected);
        // Two words turned to (optional) alternatives as a resulting of words vector shorter than filter
        let words = tst_utils::_words_vector_from_string("kkk lll");
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&words, 2, 0, 0),
            (0, 2)
        );
        let mut expected = tst_utils::_simple_filter_from_string("iii jjj kkk lll");
        expected = tst_utils::_add_word_alternative(expected, 0, ".");
        expected = tst_utils::_add_word_alternative(expected, 1, ".");
        assert_eq!(log_filters.filters.get(2).unwrap(), &expected);
        // One word turned to optional alternative and one new alternative added to second word
        let words = tst_utils::_words_vector_from_string("bar ccc sss");
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&words, 0, 0, 0),
            (1, 2)
        );
        let mut expected = tst_utils::_simple_filter_from_string("aaa qqq ccc sss");
        expected = tst_utils::_add_word_alternative(expected, 0, ".");
        expected = tst_utils::_add_word_alternative(expected, 1, "bbb");
        expected = tst_utils::_add_word_alternative(expected, 1, "bar");
        expected = tst_utils::_add_word_alternative(expected, 2, "rrr");
        expected = tst_utils::_add_word_alternative(expected, 3, "ddd");
        assert_eq!(log_filters.filters.get(0).unwrap(), &expected);

        // Tests covering when both filter and words vector do not start from column 0 and both are different indexes
        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        // Test empty words vector on valid filters
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&vec![], 5, 3, 2),
            (-1, -1)
        );
        let expected = tst_utils::_simple_filter_from_string("ttt aaa uuu bbb ccc ddd vvv");
        assert_eq!(log_filters.filters.get(5).unwrap(), &expected);

        // both filter and words vector match first word
        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.denote_optional = ".".to_string();
        //     0   1   2  |  3   4   5   6
        // w: ttt aaa kkk | uuu ccc ddd vvv
        // f:     ttt aaa | uuu bbb ccc ddd vvv
        //         0   1  |  2   3   4   5   6
        // r:     ttt aaa | uuu bbb ccc ddd vvv
        let words = tst_utils::_words_vector_from_string("ttt aaa kkk uuu ccc ddd vvv");
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&words, 5, 3, 2),
            (3, 2)
        );
        let expected = tst_utils::_simple_filter_from_string("ttt aaa uuu bbb ccc ddd vvv");
        assert_eq!(log_filters.filters.get(5).unwrap(), &expected);

        // first filter's alternative matches second word
        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.denote_optional = ".".to_string();
        //     0   1   2  |  3   4   5   6   7   8
        // w: ttt aaa uuu | xyz uuu bbb ccc ddd vvv
        // f:     ttt aaa | uuu bbb ccc ddd vvv
        //         0   1  |  2   3   4   5   6
        // r:     ttt aaa | xyz uuu bbb ccc ddd vvv
        //                   .
        let words = tst_utils::_words_vector_from_string("ttt aaa uuu xyz uuu bbb ccc ddd vvv");
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&words, 5, 3, 2),
            (4, 3)
        );
        let mut expected = tst_utils::_simple_filter_from_string("ttt aaa xyz uuu bbb ccc ddd vvv");
        expected = tst_utils::_add_word_alternative(expected, 2, ".");
        assert_eq!(log_filters.filters.get(5).unwrap(), &expected);
        assert_eq!(
            log_filters.words_hash.get(&"xyz".to_string()).unwrap(),
            &vec![5]
        );

        // second filter's alternative matches second word
        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.denote_optional = ".".to_string();
        //     0   1   2  |  3   4   5   6   7
        // w: ttt aaa uuu | xyz bbb ccc ddd vvv
        // f:     ttt aaa | uuu bbb ccc ddd vvv
        //         0   1  |  2   3   4   5   6
        // r:     ttt aaa | uuu bbb ccc ddd vvv
        //                  xyz
        let words = tst_utils::_words_vector_from_string("ttt aaa uuu xyz bbb ccc ddd vvv");
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&words, 5, 3, 2),
            (4, 3)
        );
        let mut expected = tst_utils::_simple_filter_from_string("ttt aaa uuu bbb ccc ddd vvv");
        expected = tst_utils::_add_word_alternative(expected, 2, "xyz");
        assert_eq!(log_filters.filters.get(5).unwrap(), &expected);
        assert_eq!(
            log_filters.words_hash.get(&"xyz".to_string()).unwrap(),
            &vec![5]
        );

        // words missing first alternative and second alternative with new option
        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.denote_optional = ".".to_string();
        //     0   1   2  |  3   4   5   6
        // w: ttt aaa fff | xyz ccc ddd vvv
        // f:     ttt aaa | uuu bbb ccc ddd vvv
        //         0   1  |  2   3   4   5   6
        // r:     ttt aaa | uuu bbb ccc ddd vvv
        //                   .  xyz
        let words = tst_utils::_words_vector_from_string("ttt aaa fff xyz ccc ddd vvv");
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&words, 5, 3, 2),
            (4, 4)
        );
        let mut expected = tst_utils::_simple_filter_from_string("ttt aaa uuu bbb ccc ddd vvv");
        expected = tst_utils::_add_word_alternative(expected, 2, ".");
        expected = tst_utils::_add_word_alternative(expected, 3, "xyz");
        assert_eq!(log_filters.filters.get(5).unwrap(), &expected);
        assert_eq!(
            log_filters.words_hash.get(&"xyz".to_string()).unwrap(),
            &vec![5]
        );

        // no matches
        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.denote_optional = ".".to_string();
        let words = tst_utils::_words_vector_from_string("xyz foo bar baz");
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&words, 5, 3, 2),
            (-1, -1)
        );
        let expected = tst_utils::_simple_filter_from_string("ttt aaa uuu bbb ccc ddd vvv");
        assert_eq!(log_filters.filters.get(5).unwrap(), &expected);

        // first word matching last filter alternative with earlier match available
        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        log_filters.denote_optional = ".".to_string();
        //     0   1   2  |  3   4   5   6   7   8
        // w: aaa bbb ccc | lll ddd eee fff ggg hhh
        // f:     aaa bbb | ccc ddd eee fff ggg hhh
        //                                      lll
        //         0   1  |  2   3   4   5   6   7
        // r:     aaa bbb | ccc ddd eee fff ggg hhh
        //                  lll                 lll
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc lll ddd eee fff ggg hhh");
        let new_filter = tst_utils::_simple_filter_from_string("aaa bbb ccc ddd eee fff ggg hhh");
        let new_filter = tst_utils::_add_word_alternative(new_filter, 7, "lll");
        tst_utils::_add_test_filter(&mut log_filters, new_filter);
        assert_eq!(
            log_filters.normalise_lengths_before_first_match(&words, 6, 3, 2),
            (4, 3)
        );
        let mut expected = tst_utils::_simple_filter_from_string("aaa bbb ccc ddd eee fff ggg hhh");
        expected = tst_utils::_add_word_alternative(expected, 2, "lll");
        expected = tst_utils::_add_word_alternative(expected, 7, "lll");
        assert_eq!(log_filters.filters.get(6).unwrap(), &expected);
    }

    #[test]
    fn get_indexes_of_earliest_matching_word() {
        let mut log_filters = LogFilters::new();
        // Test empty words vector on empty filters
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&vec![], 0, 0, 0),
            (-1, -1)
        );
        // Test valid words vector on empty filters
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc ddd");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 0, 0, 0),
            (-1, -1)
        );
        // Test valid words vector on empty filter
        log_filters.filters.push(vec![]);
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 0, 0, 0),
            (-1, -1)
        );

        // Tests covering when both filter and words vector start from column 0
        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        // Test empty words vector on valid filters
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&vec![], 0, 0, 0),
            (-1, -1)
        );
        // both filter and words vector match first word
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc ddd");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 0, 0, 0),
            (0, 0)
        );
        // first filter's alternative matches second word
        let words = tst_utils::_words_vector_from_string("xyz aaa ccc ddd");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 0, 0, 0),
            (1, 0)
        );
        // second filter's alternative matches second word
        let words = tst_utils::_words_vector_from_string("xyz bbb ccc ddd");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 0, 0, 0),
            (1, 1)
        );
        // first word matching last filter alternative with earlier match available
        let words = tst_utils::_words_vector_from_string("sss aaa ccc ddd");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 0, 0, 0),
            (1, 0)
        );
        // words missing first alternative and second alternative with new option
        let words = tst_utils::_words_vector_from_string("bar ccc sss");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 0, 0, 0),
            (1, 2)
        );
        // no matches
        let words = tst_utils::_words_vector_from_string("xyz");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 0, 0, 0),
            (-1, -1)
        );

        // Tests covering when both filter and words vector do not start from column 0 but both are the same indexes
        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        // Test empty words vector on valid filters
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&vec![], 0, 2, 2),
            (-1, -1)
        );
        // both filter and words vector match first word
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc ddd");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 0, 2, 2),
            (2, 2)
        );
        // first filter's alternative matches second word
        let words = tst_utils::_words_vector_from_string("aaa bbb xyz ccc");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 0, 2, 2),
            (3, 2)
        );
        // second filter's alternative matches second word
        let words = tst_utils::_words_vector_from_string("aaa bbb xyz ddd");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 0, 2, 2),
            (3, 3)
        );
        // words missing first alternative and second alternative with new option
        let words = tst_utils::_words_vector_from_string("aaa bar ddd");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 0, 1, 1),
            (2, 3)
        );
        // no matches
        let words = tst_utils::_words_vector_from_string("xyz foo bar baz");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 0, 2, 2),
            (-1, -1)
        );
        // first word matching last filter alternative with earlier match available
        let words = tst_utils::_words_vector_from_string("aaa bbb lll ddd eee fff ggg hhh");
        let new_filter = tst_utils::_simple_filter_from_string("aaa bbb ccc ddd eee fff ggg hhh");
        let new_filter = tst_utils::_add_word_alternative(new_filter, 7, "lll");
        tst_utils::_add_test_filter(&mut log_filters, new_filter);
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 6, 2, 2),
            (3, 3)
        );

        // Tests covering when both filter and words vector do not start from column 0 and both are different indexes
        let mut log_filters = tst_utils::_init_test_data();
        log_filters.max_allowed_new_alternatives = 1;
        // Test empty words vector on valid filters
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&vec![], 5, 3, 2),
            (-1, -1)
        );
        // both filter and words vector match first word
        //     0   1   2  |  3   4   5   6
        // w: ttt aaa kkk | uuu ccc ddd vvv
        // f:     ttt aaa | uuu bbb ccc ddd vvv
        //         0   1  |  2   3   4   5   6
        let words = tst_utils::_words_vector_from_string("ttt aaa kkk uuu ccc ddd vvv");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 5, 3, 2),
            (3, 2)
        );
        // first filter's alternative matches second word
        //     0   1   2  |  3   4   5   6   7   8
        // w: ttt aaa uuu | xyz uuu bbb ccc ddd vvv
        // f:     ttt aaa | uuu bbb ccc ddd vvv
        //         0   1  |  2   3   4   5   6
        let words = tst_utils::_words_vector_from_string("ttt aaa uuu xyz uuu bbb ccc ddd vvv");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 5, 3, 2),
            (4, 2)
        );
        // second filter's alternative matches second word
        //     0   1   2  |  3   4   5   6   7
        // w: ttt aaa uuu | fff bbb ccc ddd vvv
        // f:     ttt aaa | uuu bbb ccc ddd vvv
        //         0   1  |  2   3   4   5   6
        let words = tst_utils::_words_vector_from_string("ttt aaa uuu fff bbb ccc ddd vvv");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 5, 3, 2),
            (4, 3)
        );
        // words missing first alternative and second alternative with new option
        //     0   1   2  |  3   4   5   6
        // w: ttt aaa fff | xyz ccc ddd vvv
        // f:     ttt aaa | uuu bbb ccc ddd vvv
        //         0   1  |  2   3   4   5   6
        let words = tst_utils::_words_vector_from_string("ttt aaa fff xyz ccc ddd vvv");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 5, 3, 2),
            (4, 4)
        );
        // no matches
        let words = tst_utils::_words_vector_from_string("xyz foo bar baz");
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 5, 3, 2),
            (-1, -1)
        );
        // first word matching last filter alternative with earlier match available
        //     0   1   2  |  3   4   5   6   7   8
        // w: aaa bbb ccc | lll ddd eee fff ggg hhh
        // f:     aaa bbb | ccc ddd eee fff ggg hhh
        //                                      lll
        //         0   1  |  2   3   4   5   6   7
        let words = tst_utils::_words_vector_from_string("aaa bbb ccc lll ddd eee fff ggg hhh");
        let new_filter = tst_utils::_simple_filter_from_string("aaa bbb ccc ddd eee fff ggg hhh");
        let new_filter = tst_utils::_add_word_alternative(new_filter, 7, "lll");
        tst_utils::_add_test_filter(&mut log_filters, new_filter);
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 6, 3, 2),
            (4, 3)
        );
        // same filter, matching last word
        //     0   1   2   3   4   5   6   7  |  8
        // w: aaa bbb ccc lll ddd eee fff ggg | hhh
        // f:     aaa bbb ccc ddd eee fff ggg | hhh
        //                                    | lll
        //         0   1   2   3   4   5   6     7
        assert_eq!(
            log_filters.get_indexes_of_earliest_matching_word(&words, 6, 8, 7),
            (8, 7)
        );
    }

    #[test]
    fn add_filter() {
        // Test what happens if method was used on empty data structure
        let mut log_filters = LogFilters::new();
        log_filters.add_filter(tst_utils::_words_vector_from_string("aaa bbb ccc"));
        assert_eq!(
            log_filters.words_hash.get(&"aaa".to_string()).unwrap(),
            &vec![0]
        );
        assert_eq!(
            log_filters.words_hash.get(&"bbb".to_string()).unwrap(),
            &vec![0]
        );
        assert_eq!(
            log_filters.words_hash.get(&"ccc".to_string()).unwrap(),
            &vec![0]
        );
        assert_eq!(
            log_filters.filters.get(0).unwrap(),
            &tst_utils::_simple_filter_from_string("aaa bbb ccc")
        );
        // add_filter does not check if filter already exists
        log_filters.add_filter(tst_utils::_words_vector_from_string("aaa bbb ccc"));
        assert_eq!(
            log_filters.words_hash.get(&"aaa".to_string()).unwrap(),
            &vec![0, 1]
        );
        assert_eq!(
            log_filters.words_hash.get(&"bbb".to_string()).unwrap(),
            &vec![0, 1]
        );
        assert_eq!(
            log_filters.words_hash.get(&"ccc".to_string()).unwrap(),
            &vec![0, 1]
        );
        assert_eq!(
            log_filters.filters.get(1).unwrap(),
            &tst_utils::_simple_filter_from_string("aaa bbb ccc")
        );
    }

    #[test]
    fn update_hash() {
        // Test what happens if method was used on empty data structure
        let mut log_filters = LogFilters::new();
        let word = "xxx".to_string();
        log_filters.update_hash(&word, 0);
        assert_eq!(log_filters.words_hash.get(&word).is_some(), false);

        let mut log_filters = tst_utils::_init_test_data();
        // Trying to add a word not found in any filter
        let word = "xyz".to_string();
        log_filters.update_hash(&word, 0);
        assert_eq!(log_filters.words_hash.get(&word).is_some(), false);
        // Trying to add already existing word should change nothing
        let word = "aaa".to_string();
        assert_eq!(log_filters.words_hash.get(&word).unwrap(), &vec![0, 4, 5]);
        log_filters.update_hash(&word, 0);
        assert_eq!(log_filters.words_hash.get(&word).unwrap(), &vec![0, 4, 5]);
        // Adding new word to hash just after new filter was added
        let word = "xyz".to_string();
        log_filters
            .filters
            .push(tst_utils::_simple_filter_from_string(&word));
        let last_index: usize = log_filters.filters.len() - 1;
        assert_eq!(log_filters.words_hash.get(&word).is_some(), false);
        log_filters.update_hash(&word, last_index);
        assert_eq!(
            log_filters.words_hash.get(&word).unwrap(),
            &vec![last_index]
        );
        // Adding new word to hash when extending existing filter
        let word = "iii".to_string();
        log_filters.filters[0].push(vec![word.clone()]);
        assert_eq!(log_filters.words_hash.get(&word).unwrap(), &vec![2]);
        log_filters.update_hash(&word, 0);
        assert_eq!(log_filters.words_hash.get(&word).unwrap(), &vec![0, 2]);
    }

    #[test]
    fn is_word_in_filter() {
        let log_filters = tst_utils::_init_test_data();
        assert_eq!(log_filters.is_word_in_filter(&"aaa".to_string(), 0), true);
        assert_eq!(log_filters.is_word_in_filter(&"aaa".to_string(), 4), true);
        assert_eq!(log_filters.is_word_in_filter(&"hhh".to_string(), 1), true);
        assert_eq!(log_filters.is_word_in_filter(&"aaa".to_string(), 1), false);
        assert_eq!(log_filters.is_word_in_filter(&"xxx".to_string(), 2), false);
        assert_eq!(
            log_filters.is_word_in_filter(&"xxx".to_string(), log_filters.filters.len()),
            false
        );
        assert_eq!(log_filters.is_word_in_filter(&"".to_string(), 0), false);
    }
}
