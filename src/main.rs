mod http_parser;
use http_parser::{HttpRequest, HttpResponse, StatusCode};
use std::{
    env,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::from_utf8,
};
use thread_pool_server::ThreadPool;

const ECHO_PREFIX: &str = "/echo/";
const FILE_PREFIX: &str = "/files/";

fn get_response(request: &HttpRequest) -> HttpResponse {
    let mut response = match request.path.as_str() {
        "/" => HttpResponse::ok(),

        "/user-agent" => match request.headers.get("user-agent") {
            Some(user_agent) => HttpResponse::text(user_agent.to_string()),
            None => HttpResponse::bad_request(),
        },

        path if path.starts_with(ECHO_PREFIX) => {
            let param = &path[ECHO_PREFIX.len()..];
            HttpResponse::text(param.to_string())
        }

        path if path.starts_with(FILE_PREFIX) && request.method == "POST" => {
            let file_name = &path[FILE_PREFIX.len()..];
            let env_args: Vec<String> = env::args().collect();
            let default_dir = String::from("./tmp/");
            let dir = env_args.get(2).unwrap_or(&default_dir);
            let file_path = format!("{}{}", dir, file_name);

            let file_content = match request.body.as_ref() {
                Some(content) => content,
                None => &String::from(""),
            };

            match std::fs::write(file_path, file_content) {
                Ok(_) => HttpResponse::created(),
                Err(_) => HttpResponse::internal_server_error(),
            }
        }

        path if path.starts_with(FILE_PREFIX) => {
            let file_name = &path[FILE_PREFIX.len()..];
            let env_args: Vec<String> = env::args().collect();
            let default_dir = String::from("/tmp/");
            let dir = env_args.get(2).unwrap_or(&default_dir);
            let file_path = format!("{}{}", dir, file_name);

            match std::fs::read(file_path) {
                Ok(content) => {
                    let mut response = HttpResponse::with_body(StatusCode::Ok, content);
                    response.set_content_type("application/octet-stream");
                    response
                }
                Err(_) => HttpResponse::not_found(),
            }
        }

        _ => HttpResponse::not_found(),
    };

    if request.headers.get("accept-encoding") == Some(&"gzip".to_string()) {
        response.add_header("Content-Encoding", "gzip");
    }

    response
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 4096];

    match stream.read(&mut buffer) {
        Ok(n) => {
            if let Ok(request_str) = from_utf8(&buffer[..n]) {
                match HttpRequest::parse(request_str) {
                    Ok(request) => {
                        let response = get_response(&request);
                        if let Err(e) = stream.write(&response.to_bytes()) {
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
