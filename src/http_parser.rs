use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::io::Write;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub enum HttpParseError {
    InvalidRequestLine,
    InvalidHeader,
    MalformedRequest,
}

impl HttpRequest {
    pub fn parse(request: &str) -> Result<HttpRequest, HttpParseError> {
        let mut lines = request.lines();

        let request_line = lines.next().ok_or(HttpParseError::MalformedRequest)?;
        let mut parts = request_line.split_whitespace();

        let method = parts
            .next()
            .ok_or(HttpParseError::InvalidRequestLine)?
            .to_string();
        let path = parts
            .next()
            .ok_or(HttpParseError::InvalidRequestLine)?
            .to_string();
        let version = parts
            .next()
            .ok_or(HttpParseError::InvalidRequestLine)?
            .to_string();

        let mut headers = HashMap::new();
        let mut body = None;
        let mut found_empty_line = false;

        for line in lines {
            if line.is_empty() {
                found_empty_line = true;
                continue;
            }

            if found_empty_line {
                body = Some(line.to_string());
                break;
            }

            let mut header_parts = line.splitn(2, ':');
            let header_name = header_parts
                .next()
                .ok_or(HttpParseError::InvalidHeader)?
                .trim()
                .to_lowercase()
                .to_string();
            let header_value = header_parts
                .next()
                .ok_or(HttpParseError::InvalidHeader)?
                .trim()
                .to_string();

            headers.insert(header_name, header_value);
        }

        Ok(HttpRequest {
            method,
            path,
            version,
            headers,
            body,
        })
    }
}

#[derive(Debug)]
pub enum StatusCode {
    Ok = 200,
    Created = 201,
    BadRequest = 400,
    NotFound = 404,
    Forbidden = 403,
    InternalServerError = 500,
}

impl StatusCode {
    fn as_str(&self) -> &'static str {
        match self {
            StatusCode::Ok => "200 OK",
            StatusCode::Created => "201 Created",
            StatusCode::BadRequest => "400 Bad Request",
            StatusCode::NotFound => "404 Not Found",
            StatusCode::Forbidden => "403 Forbidden",
            StatusCode::InternalServerError => "500 Internal Server Error",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ContentEncoding {
    Identity,
    Gzip,
}

#[derive(Debug)]
pub struct HttpResponse {
    status: StatusCode,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
    encoding: ContentEncoding,
}

impl HttpResponse {
    pub fn new(status: StatusCode) -> Self {
        HttpResponse {
            status,
            headers: HashMap::new(),
            body: None,
            encoding: ContentEncoding::Identity,
        }
    }

    pub fn with_body(status: StatusCode, body: Vec<u8>) -> Self {
        let mut response = HttpResponse::new(status);
        response.set_body(body);
        response
    }

    pub fn with_string_body(status: StatusCode, body: String) -> Self {
        Self::with_body(status, body.into_bytes())
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = Some(body);
    }

    pub fn set_content_type(&mut self, content_type: &str) {
        self.headers
            .insert("Content-Type".to_string(), content_type.to_string());
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub fn set_accepted_encoding(&mut self, accept_encoding: Option<&str>) {
        if let Some(encodings) = accept_encoding {
            if encodings.split(',').any(|e| e.trim() == "gzip") {
                self.encoding = ContentEncoding::Gzip;
                self.add_header("Content-Encoding", "gzip");
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let body_bytes = match (self.encoding, &self.body) {
            (ContentEncoding::Gzip, Some(body)) => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(body).unwrap();
                encoder.finish().unwrap()
            }
            (_, body_opt) => body_opt.clone().unwrap_or_default(),
        };

        let mut response = format!("HTTP/1.1 {}\r\n", self.status.as_str());

        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }

        response.push_str(&format!("Content-Length: {}\r\n", body_bytes.len()));
        response.push_str("\r\n");

        let mut response_bytes = response.into_bytes();
        response_bytes.extend(body_bytes);
        response_bytes
    }
}

// Convenience methods for common responses
impl HttpResponse {
    pub fn ok() -> Self {
        HttpResponse::new(StatusCode::Ok)
    }

    pub fn created() -> Self {
        HttpResponse::new(StatusCode::Created)
    }

    pub fn not_found() -> Self {
        HttpResponse::new(StatusCode::NotFound)
    }

    pub fn bad_request() -> Self {
        HttpResponse::new(StatusCode::BadRequest)
    }

    pub fn internal_server_error() -> Self {
        HttpResponse::new(StatusCode::InternalServerError)
    }

    pub fn text(content: String) -> Self {
        let mut response = HttpResponse::with_string_body(StatusCode::Ok, content);
        response.set_content_type("text/plain");
        response
    }

    pub fn json<T: serde::Serialize>(content: &T) -> Result<Self, serde_json::Error> {
        let body = serde_json::to_string(content)?;
        let mut response = HttpResponse::with_string_body(StatusCode::Ok, body);
        response.set_content_type("application/json");
        Ok(response)
    }
}
