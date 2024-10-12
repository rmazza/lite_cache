use std::iter::Peekable;
use std::slice::Iter;

use crate::RequestError;
use crate::DELIMITER;

pub struct UtilityStruct;

impl UtilityStruct {
    pub fn split_pair(split_message_iter: &mut Peekable<Iter<'_, &str>>) -> Result<String, RequestError> {
        let command_length: usize = UtilityStruct::parse_length(split_message_iter.next().unwrap());
        let command: &str = split_message_iter.next().unwrap();
    
        if command_length != command.len() {
            return Err(RequestError::InvalidRequest(UtilityStruct::error_message("Invalid bulk string length")))
        }
        Ok(String::from(command))
    }

    pub fn null() -> String {
        format!("_{}", DELIMITER)
    }
    
    pub fn parse_length(encoded: &str) -> usize {
        encoded[1..].parse::<usize>().unwrap_or_default()
    }

    pub fn simple_string(value: &str) -> String {
        UtilityStruct::base_message('+', value, DELIMITER)
    }
    
    pub fn error_message(value: &str) -> String {
        UtilityStruct::base_message('-', value, DELIMITER)
    }
    
    fn base_message(first_char: char, value: &str, delim: &str) -> String {
        format!("{}{}{}", first_char, value, delim)
    }
}

#[cfg(test)]
mod utility_tests {
    use super::*;

    #[test]
    fn simple_string_tests() {
        assert_eq!(UtilityStruct::simple_string("OK"), "+OK\r\n");
        assert_eq!(UtilityStruct::simple_string("Hello World"), "+Hello World\r\n");
    }

    #[test]
    fn error_message_tests() {
        assert_eq!( UtilityStruct::error_message("Error"), "-Error\r\n");
        assert_eq!( UtilityStruct::error_message("Error message"), "-Error message\r\n");
    }

    #[test]
    fn parse_array_length_tests() {
        assert_eq!(UtilityStruct::parse_length("*4"), 4);
        assert_eq!(UtilityStruct::parse_length("*15"), 15);
        assert_eq!(UtilityStruct::parse_length("*100"), 100);
        assert_eq!(UtilityStruct::parse_length("*-1"), 0);
    }
}