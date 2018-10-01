extern crate logmap;

#[test]
fn no_alts_include_num_no_cols_skipped() {
    let mut log_filters = logmap::logmap::LogFilters::new();
    log_filters.max_allowed_new_alternatives = 0;
    log_filters.ignore_numeric_words = false;
    log_filters.ignore_first_columns = 0;

    log_filters.learn_line("Sep 26 09:13:15 anonymous_hostname systemd-logind[572]: Removed session c524.");
    log_filters.learn_line("Sep 27 19:27:53 anonymous_hostname systemd-logind[572]: Removed session c525.");
    log_filters.learn_line("Sep 28 13:41:26 anonymous_hostname systemd-logind[572]: Removed session c526.");

    let mut expected: String = "[Sep],[26],[09],[13],[15],[anonymous_hostname],[systemd-logind],[572],[Removed],[session],[c524],".to_string();
                 expected += "\n[Sep],[27],[19],[27],[53],[anonymous_hostname],[systemd-logind],[572],[Removed],[session],[c525],";
                 expected += "\n[Sep],[28],[13],[41],[26],[anonymous_hostname],[systemd-logind],[572],[Removed],[session],[c526]";

    assert_eq!(log_filters.to_string(), expected);
}

#[test]
fn no_alts_no_nums_no_cols_skipped() {
    let mut log_filters = logmap::logmap::LogFilters::new();
    log_filters.max_allowed_new_alternatives = 0;
    log_filters.ignore_numeric_words = true;
    log_filters.ignore_first_columns = 0;

    log_filters.learn_line("Sep 26 09:13:15 anonymous_hostname systemd-logind[572]: Removed session c524.");
    log_filters.learn_line("Sep 27 19:27:53 anonymous_hostname systemd-logind[572]: Removed session c525.");
    log_filters.learn_line("Sep 28 13:41:26 anonymous_hostname systemd-logind[572]: Removed session c526.");

    let mut expected: String = "[Sep],[anonymous_hostname],[systemd-logind],[Removed],[session],[c524],".to_string();
                 expected += "\n[Sep],[anonymous_hostname],[systemd-logind],[Removed],[session],[c525],";
                 expected += "\n[Sep],[anonymous_hostname],[systemd-logind],[Removed],[session],[c526]";

    assert_eq!(log_filters.to_string(), expected);
}

#[test]
fn no_alts_no_nums_one_col_skipped() {
    let mut log_filters = logmap::logmap::LogFilters::new();
    log_filters.max_allowed_new_alternatives = 0;
    log_filters.ignore_numeric_words = true;
    log_filters.ignore_first_columns = 1;

    log_filters.learn_line("Sep 26 09:13:15 anonymous_hostname systemd-logind[572]: Removed session c524.");
    log_filters.learn_line("Sep 27 19:27:53 anonymous_hostname systemd-logind[572]: Removed session c525.");
    log_filters.learn_line("Sep 28 13:41:26 anonymous_hostname systemd-logind[572]: Removed session c526.");

    let mut expected: String = "[anonymous_hostname],[systemd-logind],[Removed],[session],[c524],".to_string();
                 expected += "\n[anonymous_hostname],[systemd-logind],[Removed],[session],[c525],";
                 expected += "\n[anonymous_hostname],[systemd-logind],[Removed],[session],[c526]";

    assert_eq!(log_filters.to_string(), expected);
}

#[test]
fn one_alt_no_nums_one_col_skipped() {
    let mut log_filters = logmap::logmap::LogFilters::new();
    log_filters.max_allowed_new_alternatives = 1;
    log_filters.ignore_numeric_words = true;
    log_filters.ignore_first_columns = 1;

    log_filters.learn_line("Sep 26 09:13:15 anonymous_hostname systemd-logind[572]: Removed session c524.");
    log_filters.learn_line("Sep 27 19:27:53 anonymous_hostname systemd-logind[572]: Removed session c525.");
    log_filters.learn_line("Sep 28 13:41:26 anonymous_hostname systemd-logind[572]: Removed session c526.");

    let expected: String = "[anonymous_hostname],[systemd-logind],[Removed],[session],[c524,c525,c526]".to_string();

    assert_eq!(log_filters.to_string(), expected);
}

#[test]
fn one_alt_no_nums_one_col_skipped_followed_by_short_line() {
    let mut log_filters = logmap::logmap::LogFilters::new();
    log_filters.max_allowed_new_alternatives = 1;
    log_filters.ignore_numeric_words = true;
    log_filters.ignore_first_columns = 1;

    log_filters.learn_line("Sep 26 09:13:15 anonymous_hostname systemd-logind[572]: Removed session c524.");
    log_filters.learn_line("Sep 27 19:27:53 anonymous_hostname systemd-logind[572]: Removed session c525.");
    log_filters.learn_line("Sep 28 13:41:26 anonymous_hostname systemd-logind[572]: Removed session c526.");
    log_filters.learn_line("Sep 28 13:41:26 anonymous_hostname");

    let mut expected: String = "[anonymous_hostname],[systemd-logind],[Removed],[session],[c524,c525,c526],".to_string();
                 expected += "\n[anonymous_hostname]";

    assert_eq!(log_filters.to_string(), expected);
}
