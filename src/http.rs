#![allow(dead_code)]
//! http but very stupid
//! 
//! gives zero security guarantees

use std::{collections::HashMap, fmt::Display, str::FromStr};

#[derive(Debug)]
pub enum ParsingError {
    UnknownMethod,
    IndecipherableRequestLine,
    UnsupportedProtocol,
    IllegalHeader,
    UnparseableContentLength,
    ContentTooShort,
}

#[derive(Debug)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::HEAD => "HEAD",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::CONNECT => "CONNECT",
            Method::OPTIONS => "OPTIONS",
            Method::TRACE => "TRACE",
            Method::PATCH => "PATCH",
        }
    }
}

impl FromStr for Method {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_uppercase().as_str() {
            "GET" => Method::GET,
            "HEAD" => Method::HEAD,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "CONNECT" => Method::CONNECT,
            "OPTIONS" => Method::OPTIONS,
            "TRACE" => Method::TRACE,
            "PATCH" => Method::PATCH,
            _ => return Err(ParsingError::UnknownMethod)
        })
    }
}

#[derive(Debug)]
pub struct ResponseCode(usize);

impl ResponseCode {
    /// Sources:
    /// - https://developer.mozilla.org/en-US/docs/Web/HTTP/Status
    /// - https://status.js.org/
    fn get_name(&self) -> String {
        match self.0 {
            100 => "Continue",
            101 => "Switching Protocols",
            102 => "Processing",
            103 => "Early Hints",
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            203 => "Non-Authoritative Information",
            204 => "No Content",
            205 => "Reset Content",
            206 => "Partial Content",
            207 => "Multi-Status",
            208 => "Already Reported",
            218 => "This is fine",
            226 => "IM Used",
            300 => "Multiple Choices",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            304 => "Not Modified",
            305 => "Use Proxy",
            306 => "Switch Proxy",
            307 => "Temporary Redirect",
            308 => "Permanent Redirect",
            400 => "Bad Request",
            401 => "Unauthorized",
            402 => "Payment Required",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            406 => "Not Acceptable",
            407 => "Proxy Authentication Required",
            408 => "Request Timeout",
            409 => "Conflict",
            410 => "Gone",
            411 => "Length Required",
            412 => "Precondition Failed",
            413 => "Content Too Large",
            414 => "URI Too Long",
            415 => "Unsupported Media Type",
            416 => "Range Not Satisfiable",
            417 => "Expectation Failed",
            418 => "I'm a teapot",
            421 => "Misdirected Request",
            422 => "Unprocessable Content",
            423 => "Locked",
            424 => "Failed Dependency",
            425 => "Too Early",
            426 => "Upgrade Required",
            428 => "Precondition Required",
            429 => "Too Many Requests",
            431 => "Request Header Fields Too Large",
            440 => "Login Time-out",
            444 => "Connection Closed Without Response",
            449 => "Retry With",
            450 => "Blocked by Windows Parental Controls",
            451 => "Unavailable For Legal Reasons",
            494 => "Request Header Too Large",
            495 => "SSL Certificate Error",
            496 => "SSL Certificate Required",
            497 => "HTTP Request Sent to HTTPS Port",
            499 => "Client Closed Request",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            505 => "HTTP Version Not Supported",
            506 => "Variant Also Negotiates",
            507 => "Insufficient Storage",
            508 => "Loop Detected",
            509 => "Bandwidth Limit Exceeded",
            510 => "Not Extended",
            511 => "Network Authentication Required",
            520 => "Unknown Error",
            521 => "Web Server Is Down",
            522 => "Connection Timed Out",
            523 => "Origin Is Unreachable",
            524 => "A Timeout Occurred",
            525 => "SSL Handshake Failed",
            526 => "Invalid SSL Certificate",
            527 => "Railgun Listener to Origin Error",
            530 => "Origin DNS Error",
            598 => "Network Read Timeout Error",
            _ => ""
        }.to_string()
    }
}

impl Display for ResponseCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self} {}", self.get_name())
    }
}

pub enum MessageKind {
    Request {
        method: Method,
        path: String,
    },
    Response {
        code: ResponseCode,
    }
}

pub struct Message {
    pub kind: MessageKind,
    pub headers: HashMap<String, String>,
    content: Option<Box<[u8]>>,
}

impl Message {
    pub fn new_request(method: Method, path: impl ToString) -> Self {
        Message {
            kind: MessageKind::Request {
                method,
                path: path.to_string()
            },
            headers: HashMap::new(),
            content: None,
        }
    }

    pub fn new_response(code: ResponseCode) -> Self {
        Message {
            kind: MessageKind::Response { code },
            headers: HashMap::new(),
            content: None,
        }
    }

    pub fn set_header(&mut self, header: impl ToString, value: impl ToString) -> Option<String> {
        self.headers.insert(header.to_string(), value.to_string())
    }

    pub fn remove_header(&mut self, header: impl AsRef<str>) -> Option<String> {
        self.headers.remove(header.as_ref())
    }

    pub fn set_content(&mut self, content: Option<Box<[u8]>>) {
        if let Some(content) = &content {
            self.headers.insert("Content-Length".to_string(), content.len().to_string());
        } else {
            self.headers.remove("Content-Length");
        }
        self.content = content;
    }

    pub fn get_content(&self) -> Option<&[u8]> {
        self.content.as_deref()
    }

    pub fn get_content_mut(&mut self) -> Option<&mut [u8]> {
        self.content.as_deref_mut()
    }

    /// Serialize the message
    pub fn serialize(&self) -> Box<[u8]> {
        let mut bytes = format!("{}", self).into_bytes();
        if let Some(content) = &self.content {
            bytes.extend_from_slice(&content);
            // For good measure
            bytes.push(b'\r');
            bytes.push(b'\n');
        }
        bytes.into_boxed_slice()
    }

    pub fn parse(data: impl IntoIterator<Item = u8>) -> Result<Self, ParsingError> {
        println!("Trace: data into iter");
        let mut data = data.into_iter();
        let request_line: Vec<u8> = data
            .by_ref()
            .skip_while(|x| x.is_ascii_whitespace())
            .take_while(|&c| c != b'\r')
            .collect();
        println!("Trace: request line");
        // Don't try to fool me, I'm not going to allow \r without \n
        if !matches!(data.next(), Some(b'\n')) {
            return Err(ParsingError::IndecipherableRequestLine);
        }
        let request_line = String::from_utf8(request_line).map_err(|_| ParsingError::IndecipherableRequestLine)?;
        let mut request_line = request_line.split(' '); // HTTP spec says space and only space, no htab or other whitespace
        let first_word = request_line.next().ok_or(ParsingError::IndecipherableRequestLine)?;
        println!("Trace: first word");
        let kind = if first_word == "HTTP/1.1" {
            // It's a response
            let code = ResponseCode(
                request_line
                    .next()
                    .ok_or(ParsingError::IndecipherableRequestLine)?
                    .parse()
                    .map_err(|_| ParsingError::IndecipherableRequestLine)?
                );
            MessageKind::Response { code }
        } else {
            // It's a request
            let method: Method = first_word.parse()?;
            let path = request_line.next().ok_or(ParsingError::IndecipherableRequestLine)?.to_string();
            if !matches!(request_line.next(), Some("HTTP/1.1")) {
                return Err(ParsingError::UnsupportedProtocol);
            }
            if !matches!(request_line.next(), None) {
                return Err(ParsingError::IndecipherableRequestLine);
            }
            MessageKind::Request { method, path }
        };
        println!("Trace: headers");
        let mut headers: HashMap<String, String> = HashMap::new();
        loop {
            let header: Vec<u8> = data.by_ref().take_while(|&c| c != b'\r').collect();
            if !matches!(data.next(), Some(b'\n')) {
                return Err(ParsingError::IllegalHeader); // No random \r in headers
            }
            if header.len() == 0 {
                break;
            }
            let header = String::from_utf8(header).map_err(|_| ParsingError::IllegalHeader)?;
            let (key, value) = header.split_once(":").ok_or(ParsingError::IllegalHeader)?;
            let key = key.to_ascii_lowercase();
            let value = value.trim_start_matches([' ', '\t']); // SP and HTAB are the only characters that get trimmed here according to the spec
            if headers.contains_key(&key) {
                let old_value = headers.get_mut(&key).unwrap();
                old_value.push_str(", ");
                old_value.push_str(value);
            } else {
                headers.insert(key, value.to_string());
            }
        }
        println!("Trace: content");
        let content = if let Some(content_length) = headers.get("content-length") {
            let content_length = content_length.parse().map_err(|_| ParsingError::UnparseableContentLength)?;
            let content: Box<[u8]> = data.take(content_length).collect();
            if content.len() != content_length {
                return Err(ParsingError::ContentTooShort);
            }
            Some(content)
        } else {
            None
        };
        Ok(Message {
            kind,
            headers,
            content
        })
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            MessageKind::Request { method, path } => {
                writeln!(f, "{} {} HTTP/1.1\r", method.as_str(), path)?;
            },
            MessageKind::Response { code } => {
                writeln!(f, "HTTP/1.1 {}\r", code)?;
            }
        }
        for (k, v) in &self.headers {
            writeln!(f, "{k}: {v}\r")?;
        }
        writeln!(f, "\r")?;
        Ok(())
    }
}
