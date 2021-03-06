use std::io::prelude::*;
use std::collections::HashMap;
use mio::tcp::*;
use std::str;

const CR: u8 = 13;
const LF: u8 = 10;

pub struct Status {
    code: u32,
    desc: String,
}

impl Status {
    pub fn ok() -> Status {
        return Status {
            code: 200,
            desc: "OK".to_string(),
        };
    }
    pub fn not_found() -> Status {
        return Status {
            code: 404,
            desc: "Not Found".to_string(),
        };
    }
}

pub struct Response<'a> {
    version: String,
    status: Status,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    stream: &'a mut TcpStream,
}

impl<'a> Response<'a> {
    pub fn new(stream: &'a mut TcpStream) -> Response<'a> {
        return Response {
            version: "HTTP/1.1".to_string(),
            status: Status::ok(),
            headers: HashMap::new(),
            body: Vec::new(),
            stream: stream,
        };
    }

    pub fn set_not_found(&mut self) -> &mut Response<'a> {
        self.set_status(Status::not_found())
            .set_header("Content-Type", "text/html")
            .set_body_str("<html><h1>404 Not found</h1></html>")
    }

    pub fn set_status(&mut self, status: Status) -> &mut Response<'a> {
        self.status = status;
        self
    }
    pub fn set_header(&mut self, header_name: &str, value: &str) -> &mut Response<'a> {
        self.headers.insert(header_name.to_string(), value.to_string());
        self
    }

    pub fn set_length(&mut self, length: u64) -> &mut Response<'a> {
        self.set_header("Content-Length", &format!("{}", length));
        self
    }

    pub fn set_headers(&mut self, headers: &[(&str, &str)]) -> &mut Response<'a> {
        for &(name, value) in headers {
            self.set_header(name, value);
        }
        self
    }

    pub fn set_body(&mut self, body: &[u8]) -> &mut Response<'a> {
        self.body.extend_from_slice(body);
        let value = self.body.len().to_string();
        self.set_header("Content-Length", &value);
        self
    }

    pub fn set_body_str(&mut self, body: &str) -> &mut Response<'a> {
        self.set_body(body.as_bytes());
        self
    }

    fn as_bytes(&self) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(format!("{} {} {}\r\n",
                                    self.version,
                                    self.status.code,
                                    self.status.desc)
                                .as_bytes());
        for (name, value) in &self.headers {
            b.extend_from_slice(format!("{}: {}\r\n", name, value).as_bytes());
        }
        b.extend_from_slice("\r\n".as_bytes());
        b.extend_from_slice(self.body.as_slice());
        return b;
    }

    pub fn send_data(&mut self, data: &[u8]) {
        let _ = self.stream.write(data);

    }
    pub fn send_str(&mut self, data: &str) {
        let _ = self.stream.write(data.as_bytes());
    }

    pub fn send(&mut self) {
        let bytes = self.as_bytes();
        let _ = self.stream.write(&bytes);
        self.headers.clear();
        self.body.clear();
    }
}

#[derive(Debug, Clone)]
enum Method {
    Get,
}

#[derive(Debug, Clone)]
pub struct Request {
    method: Method,
    pub uri: String,
    params: HashMap<String, String>,
    version: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Request {
    fn new() -> Request {
        return Request {
            method: Method::Get,
            uri: String::new(),
            params: HashMap::new(),
            version: String::new(),
            headers: HashMap::new(),
            body: Vec::new(),
        };
    }
    fn set_version(&mut self, version: &str) {
        self.version = version.to_string();
    }
    fn set_uri(&mut self, uri: &str) {
        self.uri = uri.to_string();
    }
    fn set_param(&mut self, name: &str, value: &str) {
        self.params.insert(name.to_string(), value.to_string());
    }
    fn parse_uri(&mut self, uri: &str) {
        match uri.find('?') {
            Some(idx) => {
                let params_str = &uri[idx + 1..];
                let base_uri = &uri[..idx];
                self.set_uri(base_uri);
                for param in params_str.split('&') {
                    let parts: Vec<&str> = param.split('=').collect();
                    if parts.len() == 2 {
                        self.set_param(parts[0], parts[1]);
                    }
                }
            }
            None => {
                self.set_uri(uri);
            }
        }
    }
    fn set_method(&mut self, method: Method) {
        self.method = method;
    }
    fn set_header(&mut self, header_name: &str, value: &str) {
        self.headers.insert(header_name.to_string(), value.to_string());
    }
    fn get_header(&self, header_name: &str) -> Option<String> {
        if let Some(s) = self.headers.get(header_name) {
            return Some(s.clone());
        }
        return None;
    }
    fn set_body(&mut self, body: &[u8]) {
        self.body = body.to_vec();
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum State {
    ParseRequestLine,
    ParseHeaders,
    ParseBody,
    Done,
    Error,
}

pub struct RequestBuilder {
    data: Vec<u8>,
    parsed: usize,
    body_size: usize,
    request: Request,
    state: State,
}

impl RequestBuilder {
    pub fn new() -> RequestBuilder {
        return RequestBuilder {
            state: State::ParseRequestLine,
            data: Vec::new(),
            parsed: 0,
            body_size: 0,
            request: Request::new(),
        };
    }

    fn get_line(&mut self) -> Option<Vec<u8>> {
        for i in self.parsed..self.data.len() {
            if i == self.parsed {
                match self.data[i] {
                    LF => {
                        let res = &self.data[self.parsed..i];
                        self.parsed = i + 1;
                        return Some(res.to_vec());
                    }
                    _ => {}
                }
            } else {
                match (self.data[i - 1], self.data[i]) {
                    (CR, LF) => {
                        let res = &self.data[self.parsed..i - 1];
                        self.parsed = i + 1;
                        return Some(res.to_vec());
                    }
                    (_, LF) => {
                        let res = &self.data[self.parsed..i];
                        self.parsed = i + 1;
                        return Some(res.to_vec());
                    }
                    _ => {}
                }
            }
        }
        return None;
    }

    fn parse_request(&mut self) -> Option<Request> {
        loop {
            let old_state = self.state;
            match old_state {
                State::ParseRequestLine => {
                    match self.get_line() {
                        None => return None,
                        Some(vec) => {
                            match str::from_utf8(&vec) {
                                Ok(s) => {
                                    let parts = s.split(" ").collect::<Vec<_>>();
                                    if parts.len() != 3 {
                                        println!("Invalid request: {}", s);
                                        self.state = State::Error;
                                        return None;
                                    }
                                    match parts[0] {
                                        "GET" => self.request.set_method(Method::Get),
                                        _ => {
                                            println!("Unsupported method {}", parts[0]);
                                            self.state = State::Error;
                                            return None;
                                        }
                                    }
                                    self.request.parse_uri(parts[1]);
                                    self.request.set_version(parts[2]);
                                    self.state = State::ParseHeaders;
                                }
                                Err(e) => {
                                    println!("Invalid utf8 request line: {}", e);
                                    self.state = State::Error;
                                    return None;
                                }
                            }
                        }
                    }
                }
                State::ParseHeaders => {
                    match self.get_line() {
                        None => return None,
                        Some(vec) => {
                            match str::from_utf8(&vec) {
                                Ok(s) => {
                                    match s {
                                        "" => {
                                            // We parsed the last header.
                                            if let Some(s) = self.request
                                                                 .get_header("Content-Length") {
                                                match s.parse() {
                                                    Ok(u) => {
                                                        if u > 0 {
                                                            self.body_size = u;
                                                            self.state = State::ParseBody;
                                                        }
                                                    }
                                                    _ => {
                                                        println!("Invalid Content-Length header \
                                                                  value: {}",
                                                                 s);
                                                        self.state = State::Error;
                                                    }
                                                }
                                            }
                                            self.state = State::Done;
                                        }
                                        _ => {
                                            match s.find(": ") {
                                                Some(idx) => {
                                                    let (name, value) = s.split_at(idx);
                                                    self.request.set_header(name, &value[2..]);
                                                }
                                                None => {
                                                    println!("Invalid header line: {}", s);
                                                    self.state = State::Error;
                                                    return None;
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("Invalid utf8 header line: {}", e);
                                    self.state = State::Error;
                                    return None;
                                }
                            }
                        }
                    }
                }
                State::ParseBody => {
                    if self.request.body.len() < self.parsed + self.body_size {
                        return None;
                    }
                    self.request
                        .set_body(&self.data[self.parsed..self.parsed + self.body_size]);
                    self.parsed += self.body_size;
                    self.state = State::Done;
                }
                State::Done => {
                    let parsed_request = self.request.clone();
                    self.request = Request::new();
                    self.data = (&self.data[self.parsed..]).to_vec();
                    self.parsed = 0;
                    self.state = State::ParseRequestLine;
                    return Some(parsed_request);
                }
                _ => panic!("Unknown state: {:?}", self.state),
            }
        }
    }

    pub fn read(&mut self, data: &[u8]) -> Option<Request> {
        self.data.extend_from_slice(data);
        return self.parse_request();
    }
}
