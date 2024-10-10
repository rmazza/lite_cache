mod commands;
mod utils;

use std::slice::Iter;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use commands::set::SetCommand;
use utils::UtilityStruct;

pub const DELIMITER: &str = "\r\n";
const OK: &str = "OK";

static GLOBAL_MAP: Lazy<DashMap<String, String>> = Lazy::new(DashMap::new);

#[derive(Debug, PartialEq)]
pub enum RequestError {
    InvalidRequest(String),
    KeyNotFound(String),
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Listening on port 6379...");

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("Accepted connection from: {:?}", addr);

        tokio::spawn(async move {
            let mut buffer = [0; 1024];

            loop {
                match socket.read(&mut buffer).await {
                    Ok(0) => {
                        println!("Connection closed by client: {:?}", addr);
                        return;
                    }
                    Ok(n) => {
                        let message = String::from_utf8_lossy(&buffer[0..n]);

                        println!("Received message: {}", message);

                        let response = match process_message(&message) {
                            Ok(success_message) => success_message,
                            Err(e) => get_error_message(&e).to_string(),
                        };

                        println!("Sending response: {}", response);

                        if let Err(e) = socket.write_all(response.as_bytes()).await {
                            println!("Failed to write to socket; err = {:?}", e);
                            return; // Close connection on write error
                        }
                    }
                    Err(e) => {
                        println!("Failed to read from socket; err = {:?}", e);
                        return; // Close connection on read error
                    }
                }
            }
        });
    }
}

fn get_error_message(error: &RequestError) -> String {
    match error {
        RequestError::InvalidRequest(ref message) => format!("-{}\r\n", message),
        RequestError::KeyNotFound(ref key) => format!("-Error Key not found: {}\r\n", key),
    }
}

fn process_message(main_message: &str) -> Result<String, RequestError> {
    if !main_message.starts_with('*') {
        return Err(RequestError::InvalidRequest(UtilityStruct::error_message("Invalid message format")));
    }

    let split_message: Vec<&str> = main_message.split(DELIMITER).collect();
    let mut message_iter: Iter<'_, &str> = split_message.iter();

    let length: usize = UtilityStruct::parse_length(message_iter.next().unwrap());
    if length != ((split_message.len() - 1) / 2) {
        return Err(RequestError::InvalidRequest(UtilityStruct::error_message("Invalid array length")));
    }

    let command = UtilityStruct::split_pair(&mut message_iter)?;

    match command.to_lowercase().as_str() {
        "ping" => Ok(UtilityStruct::simple_string("PONG")),
        "echo" => {
            let message_to_echo = UtilityStruct::split_pair(&mut message_iter)?;
            Ok(UtilityStruct::simple_string(&message_to_echo))
        },
        "set" => {
            let set_command: SetCommand = SetCommand::new(UtilityStruct::split_pair(&mut message_iter)?, UtilityStruct::split_pair(&mut message_iter)?);
            GLOBAL_MAP.insert(set_command.key, set_command.value);
            Ok(UtilityStruct::simple_string(OK))
        },
        "get" => {
            let key = UtilityStruct::split_pair(&mut message_iter)?;
            if let Some(found_value) = GLOBAL_MAP.get(key.as_str()) {
                Ok(UtilityStruct::simple_string(&found_value))
            } else {
                Err(RequestError::KeyNotFound(key))
            }
        }, 
        "command" => Ok( UtilityStruct::simple_string(OK)),
        _ => Err(RequestError::InvalidRequest(UtilityStruct::error_message(&format!("Command {} not found", command)))),
    }
}

#[cfg(test)]
mod main_tests {
    use crate::*;

    #[test]
    fn process_message_starts_with_tests() {
        assert_eq!(process_message("&t"), Err(RequestError::InvalidRequest("-Invalid message format\r\n".to_string())));
        assert_eq!(process_message("-t"), Err(RequestError::InvalidRequest("-Invalid message format\r\n".to_string())));
    }

    #[test]
    fn process_message_invalid_array_length() {
        assert_eq!(process_message("*3\r\n$4\r\necho\r\n$11\r\nhello world\r\n"), Err(RequestError::InvalidRequest("-Invalid array length\r\n".to_string())));
        assert_eq!(process_message("*4\r\n$3\r\nget\r\n$3\r\nkey\r\n"), Err(RequestError::InvalidRequest("-Invalid array length\r\n".to_string())));
    }

    #[test]
    fn process_message_echo_test() {
        assert_eq!(process_message("*2\r\n$4\r\necho\r\n$11\r\nhello world\r\n"), Ok("+hello world\r\n".to_string()));
        assert_eq!(process_message("*2\r\n$4\r\necho\r\n$19\r\nhello !@#$%^& world\r\n"), Ok("+hello !@#$%^& world\r\n".to_string()));
    }

    #[test]
    fn process_message_ping_test() {
        assert_eq!(process_message("*1\r\n$4\r\nping\r\n"), Ok("+PONG\r\n".to_string()));
    }

    #[test]
    fn process_invalid_bulk_string() {
        assert_eq!(process_message("*2\r\n$3\r\necho\r\n$11\r\nhello world\r\n"), Err(RequestError::InvalidRequest("-Invalid bulk string length\r\n".to_string())))
    }

    #[test]
    fn process_message_command_not_found() {
        assert_eq!(process_message("*1\r\n$4\r\nzzzz\r\n"), Err(RequestError::InvalidRequest("-Command zzzz not found\r\n".to_string())))
    }

    #[test]
    fn process_message_set_get() {
        assert_eq!(process_message("*3\r\n$3\r\nset\r\n$7\r\ntestKey\r\n$9\r\ntestValue\r\n"), Ok("+OK\r\n".to_string()));
        assert_eq!(process_message("*2\r\n$3\r\nget\r\n$7\r\ntestKey\r\n"), Ok("+testValue\r\n".to_string()));
    }
}