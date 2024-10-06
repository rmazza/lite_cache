use std::slice::Iter;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const DELIMITER: &str = "\r\n";
const OK: &str = "OK";

#[derive(Debug, PartialEq)]
enum RequestError {
    InvalidRequest(String),
}

#[tokio::main]
async fn main() {
    // Bind the listener to port 6379
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Listening on port 6379...");

    loop {
        // Accept incoming connections
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("Accepted connection from: {:?}", addr);

        tokio::spawn(async move {
            let mut buffer = [0; 1024];

            // Read data from the socket
            match socket.read(&mut buffer).await {
                Ok(n) if n == 0 => return, // connection closed
                Ok(n) => {
                    // Convert the buffer into a string 
                    let message = String::from_utf8_lossy(&buffer[0..n]);
                    println!("{}", message);
                    // Process the message
                    let _response = process_message(&message);
                    // let _response = "+OK\r\n";
                

                    if let Err(e) = socket.write_all(&buffer[0..n]).await {
                        println!("Failed to write to socket; err = {:?}", e);
                    }
                }
                Err(e) => {
                    println!("Failed to read from socket; err = {:?}", e);
                }
            }
        });
    }
}

fn simple_string(value: &str) -> String {
    base_message('+', value, DELIMITER)
}

fn error_message(value: &str) -> String {
    base_message('-',value, DELIMITER)
}

fn base_message(first_char: char, value: &str, delim: &str) -> String {
    format!("{}{}{}", first_char, value, delim)
}

fn parse_length(encoded: &str) -> usize {
    encoded[1..].parse::<usize>().unwrap_or_default()
}
// "*2\r\n$4\r\necho\r\n$11\r\nhello world\r\n”
// "*2\r\n$3\r\nget\r\n$3\r\nkey\r\n”

fn process_message(main_message: &str) -> Result<String, RequestError> {
    if !main_message.starts_with('*') {
        return Err(RequestError::InvalidRequest(error_message("Invalid message format")));
    }
    let split_message: Vec<&str> = main_message.split(DELIMITER).collect();
    let mut message_iter: Iter<'_, &str> = split_message.iter();
    
    let length: usize =  parse_length(message_iter.next().unwrap());

    if length != ((split_message.len() - 1) / 2) {
        return Err(RequestError::InvalidRequest(error_message("Invalid array length")));
    }
    
    let command = split_pair(&mut message_iter)?;

    match command.to_lowercase().as_str() {
        "ping" => Ok(simple_string("PONG")),
        "echo" => {
            let message_to_echo = split_pair( &mut message_iter)?;
            Ok(simple_string(&message_to_echo))
        },
        _ => Err(RequestError::InvalidRequest(format!("Command {} not found", command).to_string()))
    }   
}

fn split_pair(split_message_iter: &mut Iter<'_, &str>) -> Result<String, RequestError> {
    let command_length: usize = parse_length(split_message_iter.next().unwrap());
    let command: &str = split_message_iter.next().unwrap();

    if command_length != command.len() {
        return Err(RequestError::InvalidRequest(error_message("Invalid bulk string length")))
    }
    Ok(String::from(command))
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn simple_string_tests() {
        assert_eq!(simple_string("OK"), "+OK\r\n");
        assert_eq!(simple_string("Hello World"), "+Hello World\r\n");
    }

    #[test]
    fn error_message_tests() {
        assert_eq!(error_message("Error"), "-Error\r\n");
        assert_eq!(error_message("Error message"), "-Error message\r\n");
    }

    #[test]
    fn parse_array_length_tests() {
        assert_eq!(parse_length("*4"), 4);
        assert_eq!(parse_length("*15"), 15);
        assert_eq!(parse_length("*100"), 100);
        assert_eq!(parse_length("*-1"), 0);
    }

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

    }
}