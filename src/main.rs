mod http_parser;
use http_parser::HttpRequest;
use std::{
    env,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::from_utf8,
};
use thread_pool_server::ThreadPool;

const ECHO_PREFIX: &str = "/echo/";
const FILE_PREFIX: &str = "/files/";

fn get_response_with_body_str(body: String, content_type: Option<&str>) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        content_type.unwrap_or("text/plain"),
        body.len(),
        body
    )
}

fn get_response(request: HttpRequest) -> String {
    match request.path.as_str() {
        "/" => String::from("HTTP/1.1 200 OK\r\n\r\n"),

        "/user-agent" => match request.headers.get("user-agent") {
            Some(user_agent) => get_response_with_body_str(user_agent.to_string(), None),
            None => String::from("HTTP/1.1 400 Bad Request\r\n\r\n"),
        },

        path if path.starts_with(ECHO_PREFIX) => {
            let param = &path[ECHO_PREFIX.len()..];
            get_response_with_body_str(param.to_string(), None)
        }

        path if path.starts_with(FILE_PREFIX) && request.method == "POST" => {
            let file_name = &path[FILE_PREFIX.len()..];
            let env_args: Vec<String> = env::args().collect();
            let default_dir = String::from("./tmp/");
            let dir = env_args.get(2).unwrap_or(&default_dir);
            let file_path = format!("{}{}", dir, file_name);
            let file_content = request.body.unwrap_or_default();
            match std::fs::write(file_path, file_content) {
                Ok(_) => String::from("HTTP/1.1 201 Created\r\n\r\n"),
                Err(_) => String::from("HTTP/1.1 500 Internal Server Error\r\n\r\n"),
            }
        }

        path if path.starts_with(FILE_PREFIX) => {
            let file_name = &path[FILE_PREFIX.len()..];
            let env_args: Vec<String> = env::args().collect();
            let default_dir = String::from("/tmp/");
            let dir = env_args.get(2).unwrap_or(&default_dir);
            let file_path = format!("{}{}", dir, file_name);
            println!("file_path: {}", file_path);
            let file_content = match std::fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(_) => return String::from("HTTP/1.1 404 Not Found\r\n\r\n"),
            };

            println!("file_content: {}", file_content);

            get_response_with_body_str(file_content, Some("application/octet-stream"))
        }

        _ => String::from("HTTP/1.1 404 Not Found\r\n\r\n"),
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 4096];

    match stream.read(&mut buffer) {
        Ok(n) => {
            if let Ok(request_str) = from_utf8(&buffer[..n]) {
                match HttpRequest::parse(request_str) {
                    Ok(request) => {
                        let response = get_response(request);
                        if let Err(e) = stream.write(response.as_bytes()) {
                            eprintln!("Failed to send response: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Failed to parse request: {:?}", e),
                }
            }
        }
        Err(e) => eprintln!("Failed to read from connection: {}", e),
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("received request");
                pool.execute(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
