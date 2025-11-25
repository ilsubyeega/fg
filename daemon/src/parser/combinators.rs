//! Reusable nom parser combinators for Fall Guys log parsing.
//!
//! This module provides common parsing primitives that are shared across
//! multiple parsing rules. Using these combinators ensures consistency
//! and reduces code duplication.
//!
//! # Overview
//!
//! The combinators are organized into categories:
//!
//! - **Basic parsers**: [`identifier`], [`parse_isize`], [`parse_bool`]
//! - **Network parsers**: [`ip_address`], [`port_number`]
//! - **Navigation**: [`skip_to_tag`], [`take_until_eol`]
//! - **Log-specific**: [`log_tag`], [`key_equals_value`], [`key_colon_value`]
//!
//! # Example
//!
//! ```rust
//! use fg_daemon::parser::combinators::{identifier, parse_isize, skip_to_tag};
//! use nom::IResult;
//!
//! fn parse_player_id(input: &str) -> IResult<&str, isize> {
//!     let (input, _) = skip_to_tag("playerID = ")(input)?;
//!     parse_isize(input)
//! }
//! ```

use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{char, digit1},
    combinator::{map_res, opt, recognize},
    multi::separated_list1,
    sequence::{delimited, preceded},
};

// =============================================================================
// Basic Parsers
// =============================================================================

/// Parses an identifier consisting of alphanumeric characters, underscores,
/// dots, and hyphens.
///
/// This is commonly used for parsing:
/// - State names: `FGClient.StateMainMenu`
/// - Game mode IDs: `classic_solo_main_show`
/// - Round IDs: `round_tunnel_40`
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::combinators::identifier;
///
/// let (remaining, id) = identifier("FGClient.StateMainMenu rest").unwrap();
/// assert_eq!(id, "FGClient.StateMainMenu");
/// assert_eq!(remaining, " rest");
/// ```
pub fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '.' || c == '-')(input)
}

/// Parses a sequence of digits into an `isize`.
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::combinators::parse_isize;
///
/// let (_, num) = parse_isize("42 remaining").unwrap();
/// assert_eq!(num, 42);
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The input doesn't start with digits
/// - The number is too large for `isize`
pub fn parse_isize(input: &str) -> IResult<&str, isize> {
    map_res(digit1, |s: &str| s.parse::<isize>()).parse(input)
}

/// Parses "True" or "False" into a boolean.
///
/// Note: This is case-sensitive and matches the format used in Fall Guys logs.
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::combinators::parse_bool;
///
/// let (_, val) = parse_bool("True rest").unwrap();
/// assert!(val);
///
/// let (_, val) = parse_bool("False rest").unwrap();
/// assert!(!val);
/// ```
pub fn parse_bool(input: &str) -> IResult<&str, bool> {
    let (input, value) = nom::branch::alt((tag("True"), tag("False"))).parse(input)?;
    Ok((input, value == "True"))
}

/// Parses digits that may contain comma separators (e.g., "1,234").
///
/// This is commonly used for latency values in network metrics.
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::combinators::parse_isize_with_commas;
///
/// let (_, num) = parse_isize_with_commas("1,234 ms").unwrap();
/// assert_eq!(num, 1234);
/// ```
pub fn parse_isize_with_commas(input: &str) -> IResult<&str, isize> {
    let (input, digits_str) = take_while1(|c: char| c.is_ascii_digit() || c == ',')(input)?;
    let clean: String = digits_str.chars().filter(|c| *c != ',').collect();
    let value = clean.parse().unwrap_or(-1);
    Ok((input, value))
}

// =============================================================================
// Network Parsers
// =============================================================================

/// Parses an IPv4 address (digits separated by dots).
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::combinators::ip_address;
///
/// let (_, ip) = ip_address("192.168.1.1:8080").unwrap();
/// assert_eq!(ip, "192.168.1.1");
/// ```
pub fn ip_address(input: &str) -> IResult<&str, &str> {
    recognize(separated_list1(char('.'), digit1)).parse(input)
}

/// Parses a port number (sequence of digits).
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::combinators::port_number;
///
/// let (_, port) = port_number("8080 rest").unwrap();
/// assert_eq!(port, "8080");
/// ```
pub fn port_number(input: &str) -> IResult<&str, &str> {
    digit1(input)
}

/// Parses an IP address with optional port.
///
/// Returns `(ip_address, Option<port>)`.
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::combinators::ip_with_optional_port;
///
/// let (_, (ip, port)) = ip_with_optional_port("192.168.1.1:8080").unwrap();
/// assert_eq!(ip, "192.168.1.1");
/// assert_eq!(port, Some("8080"));
///
/// let (_, (ip, port)) = ip_with_optional_port("192.168.1.1 rest").unwrap();
/// assert_eq!(ip, "192.168.1.1");
/// assert_eq!(port, None);
/// ```
pub fn ip_with_optional_port(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    let (input, ip) = ip_address(input)?;
    let (input, port) = opt(preceded(char(':'), port_number)).parse(input)?;
    Ok((input, (ip, port)))
}

// =============================================================================
// Navigation Parsers
// =============================================================================

/// Creates a parser that skips to and consumes a specific tag.
///
/// This is useful for finding a specific marker in a log line and
/// positioning the parser right after it.
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::combinators::skip_to_tag;
///
/// let parser = skip_to_tag("playerID = ");
/// let (remaining, _) = parser("some prefix playerID = 42").unwrap();
/// assert_eq!(remaining, "42");
/// ```
pub fn skip_to_tag<'a>(target: &'static str) -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
    move |input: &'a str| {
        let (remaining, _) = take_until(target)(input)?;
        tag(target)(remaining)
    }
}

/// Takes characters until the end of line or end of input.
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::combinators::take_until_eol;
///
/// let (remaining, line) = take_until_eol("first line\nsecond line").unwrap();
/// assert_eq!(line, "first line");
/// ```
pub fn take_until_eol(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c != '\n' && c != '\r')(input)
}

// =============================================================================
// Log-Specific Parsers
// =============================================================================

/// Parses a bracketed log tag like `[GameStateMachine]`.
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::combinators::log_tag;
///
/// let (remaining, tag_content) = log_tag("[GameStateMachine] message").unwrap();
/// assert_eq!(tag_content, "GameStateMachine");
/// assert_eq!(remaining, " message");
/// ```
pub fn log_tag(input: &str) -> IResult<&str, &str> {
    delimited(
        char('['),
        take_while1(|c: char| c != ']'),
        char(']'),
    ).parse(input)
}

/// Parses a "key = value" pattern and returns the value.
///
/// # Arguments
///
/// * `key` - The key to look for
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::combinators::key_equals_value;
///
/// let parser = key_equals_value("playerID");
/// let (_, value) = parser("playerID = 42, rest").unwrap();
/// assert_eq!(value, "42,");
/// ```
pub fn key_equals_value<'a>(key: &'static str) -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
    move |input: &'a str| {
        let (input, _) = tag(key)(input)?;
        let (input, _) = tag(" = ")(input)?;
        take_while1(|c: char| !c.is_whitespace())(input)
    }
}

/// Parses a "key: value" pattern and returns the value.
///
/// # Arguments
///
/// * `key` - The key to look for
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::combinators::key_colon_value;
///
/// let parser = key_colon_value("Status");
/// let (_, value) = parser("Status: Active rest").unwrap();
/// assert_eq!(value, "Active");
/// ```
pub fn key_colon_value<'a>(key: &'static str) -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
    move |input: &'a str| {
        let (input, _) = tag(key)(input)?;
        let (input, _) = tag(": ")(input)?;
        take_while1(|c: char| !c.is_whitespace())(input)
    }
}

/// Parses content inside parentheses.
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::combinators::parenthesized;
///
/// let (remaining, content) = parenthesized("(pc_steam) rest").unwrap();
/// assert_eq!(content, "pc_steam");
/// assert_eq!(remaining, " rest");
/// ```
pub fn parenthesized(input: &str) -> IResult<&str, &str> {
    delimited(
        char('('),
        take_while1(|c: char| c != ')'),
        char(')'),
    ).parse(input)
}

/// Parses a JSON-style key-value pair: `"key": "value"` or `"key": value`.
///
/// Handles both quoted and unquoted values.
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::combinators::json_key_value;
///
/// let (_, (key, value)) = json_key_value(r#"  "state": "Connecting""#).unwrap();
/// assert_eq!(key, "state");
/// assert_eq!(value, "Connecting");
/// ```
pub fn json_key_value(input: &str) -> IResult<&str, (&str, &str)> {
    let (input, _) = take_until("\"")(input)?;
    let (input, key) = delimited(
        char('"'),
        take_while1(|c: char| c != '"'),
        char('"'),
    ).parse(input)?;
    let (input, _) = tag(": ")(input)?;
    
    // Value can be quoted or unquoted
    let (input, value) = nom::branch::alt((
        delimited(char('"'), take_while(|c: char| c != '"'), char('"')),
        take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '-' || c == '.'),
    )).parse(input)?;
    
    Ok((input, (key, value)))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier() {
        assert_eq!(identifier("FGClient.StateMainMenu").unwrap().1, "FGClient.StateMainMenu");
        assert_eq!(identifier("round_tunnel_40").unwrap().1, "round_tunnel_40");
        assert_eq!(identifier("some-id-123").unwrap().1, "some-id-123");
    }

    #[test]
    fn test_parse_isize() {
        assert_eq!(parse_isize("42").unwrap().1, 42);
        assert_eq!(parse_isize("0").unwrap().1, 0);
        assert_eq!(parse_isize("12345 rest").unwrap().1, 12345);
    }

    #[test]
    fn test_parse_bool() {
        assert!(parse_bool("True").unwrap().1);
        assert!(!parse_bool("False").unwrap().1);
    }

    #[test]
    fn test_ip_address() {
        assert_eq!(ip_address("192.168.1.1").unwrap().1, "192.168.1.1");
        assert_eq!(ip_address("10.0.0.1:8080").unwrap().1, "10.0.0.1");
    }

    #[test]
    fn test_ip_with_optional_port() {
        let (_, (ip, port)) = ip_with_optional_port("192.168.1.1:8080").unwrap();
        assert_eq!(ip, "192.168.1.1");
        assert_eq!(port, Some("8080"));

        let (_, (ip, port)) = ip_with_optional_port("192.168.1.1").unwrap();
        assert_eq!(ip, "192.168.1.1");
        assert_eq!(port, None);
    }

    #[test]
    fn test_skip_to_tag() {
        let parser = skip_to_tag("ID=");
        let (remaining, _) = parser("prefix ID=42").unwrap();
        assert_eq!(remaining, "42");
    }

    #[test]
    fn test_log_tag() {
        let (remaining, tag_content) = log_tag("[GameStateMachine] message").unwrap();
        assert_eq!(tag_content, "GameStateMachine");
        assert_eq!(remaining, " message");
    }

    #[test]
    fn test_json_key_value() {
        let (_, (key, value)) = json_key_value(r#"  "state": "Connecting""#).unwrap();
        assert_eq!(key, "state");
        assert_eq!(value, "Connecting");

        let (_, (key, value)) = json_key_value(r#"  "count": 42"#).unwrap();
        assert_eq!(key, "count");
        assert_eq!(value, "42");
    }

    #[test]
    fn test_parenthesized() {
        let (_, content) = parenthesized("(pc_steam)").unwrap();
        assert_eq!(content, "pc_steam");
    }
}
