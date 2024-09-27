use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const DELIMITER: &str = "\r\n";

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
                    let message: String = String::from_utf8_lossy(&buffer[0..n]).into_owned();
                    println!("{}", message);
                    // Process the message
                    // let response = process_message(message.to_string());
                    let _response = "+OK\r\n";
                

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


// "*2\r\n$4\r\necho\r\n$11\r\nhello world\r\n”
// "*2\r\n$3\r\nget\r\n$3\r\nkey\r\n”

fn process_message(main_message: String) -> Result<String, String> {
    if !main_message.starts_with('*') {
        return Err(error_message("Invalid message format"));
    }

    Ok(String::from("Test"))
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
    fn process_message_starts_with_tests() {
        assert_eq!(process_message("&t".to_string()).err().unwrap(), "-Invalid message format\r\n");
        assert_eq!(process_message("-t".to_string()).err().unwrap(), "-Invalid message format\r\n");
    }
}