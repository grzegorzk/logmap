extern crate logmap;

#[test]
fn no_alternatives_allowed() {
    let mut log_filters = logmap::logmap::LogFilters::new();
    log_filters.max_allowed_new_alternatives = 1;
    log_filters.min_req_consequent_matches = 3;
    log_filters.ignore_numeric_words = true;
    log_filters.ignore_first_columns = 2;
    
    log_filters.learn_line("Sep 16 20:17:04 AM kernel: wlp2s0: authenticated");
    let expected = "[kernel],[wlp2s0],[authenticated]";
    assert_eq!(log_filters.to_string(), expected);
    println!("{}", log_filters.to_string());
}

#[test]
fn one_alternative_allowed() {
}

#[test]
fn three_alternatives_allowed() {
}
