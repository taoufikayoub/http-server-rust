use std::collections::HashMap;

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
