use cached::proc_macro::cached;
use regex::Regex;

pub mod rules;
pub mod task_parser;
pub mod task_watch;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseResult<T> {
    Parsed(T),
    NeedMoreLines,
    Unreachable,
    None,
}

#[cached]
pub fn create_regex(pattern: &'static str) -> Regex {
    Regex::new(pattern).unwrap()
}
