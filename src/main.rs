use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::from_utf8,
};

const ECHO_PREFIX: &str = "/echo/";

fn get_response(path: &str) -> String {
    if path == "/" {
        return String::from("HTTP/1.1 200 OK\r\n\r\n");
    }

    if path.starts_with(ECHO_PREFIX) {
        let param = &path[ECHO_PREFIX.len()..];
        return format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            param.len(),
            param
        );
    }

    String::from("HTTP/1.1 404 Not Found\r\n\r\n")
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 4096];

    match stream.read(&mut buffer) {
        Ok(n) => {
            if let Ok(request) = from_utf8(&buffer[..n]) {
                if let Some(request_line) = request.lines().next() {
                    let parts: Vec<&str> = request_line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let path = parts[1];

                        let response = get_response(path);

                        if let Err(e) = stream.write(response.as_bytes()) {
                            eprintln!("Failed to send response: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to read from connection: {}", e),
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("received request");
                handle_connection(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
