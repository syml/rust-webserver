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

pub struct Response {
    version: String,
    status: Status,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Response {
    pub fn new() -> Response {
        return Response {
            version: "HTTP/1.1".to_string(),
            status: Status::ok(),
            headers: HashMap::new(),
            body: Vec::new(),
        };
    }

    pub fn not_found() -> Response {
        let mut r = Response::new();
        r.set_status(Status::not_found());
        r.set_header("Content-Type", "text/html");
        r.set_body("<html><h1>404 Not found</h1></html>");
        return r;
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }
    pub fn set_header(&mut self, header_name: &str, value: &str) {
        self.headers.insert(header_name.to_string(), value.to_string());
    }

    pub fn set_body_bytes(&mut self, body: &[u8]) {
        self.body.extend_from_slice(body);
        let value = self.body.len().to_string();
        self.set_header("Content-Length", &value);
    }

    pub fn set_body(&mut self, body: &str) {
        self.set_body_bytes(body.as_bytes());
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
}

#[derive(Debug, Clone)]
enum Method {
    Get,
}

#[derive(Debug, Clone)]
pub struct Request {
    version: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    pub uri: String,
    method: Method,
}

impl Request {
    fn new() -> Request {
        return Request {
            version: String::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            uri: String::new(),
            method: Method::Get,
        };
    }
    fn set_version(&mut self, version: &str) {
        self.version = version.to_string();
    }
    fn set_uri(&mut self, uri: &str) {
        self.uri = uri.to_string();
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

pub struct Connection {
    socket: TcpStream,
    data: Vec<u8>,
    parsed: usize,
    body_size: usize,
    request: Request,
    state: State,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        return Connection {
            state: State::ParseRequestLine,
            socket: socket,
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
                                    self.request.set_uri(parts[1]);
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
                                                    Err(e) => {
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

    pub fn read(&mut self) -> Option<Request> {
        self.socket.read_to_end(&mut self.data);
        return self.parse_request();
    }

    pub fn write(&mut self, response: Response) {
        self.socket.write(&response.as_bytes());
    }
}
