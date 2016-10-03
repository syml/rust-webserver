use std::io::prelude::*;
use std::collections::HashMap;
use mio::tcp::*;

struct Status {
    code: u32,
    desc: String,
}

impl Status {
    fn ok() -> Status {
        return Status {
            code: 200,
            desc: "OK".to_string(),
        };
    }
}

struct Response {
    version: String,
    status: Status,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Response {
    fn new() -> Response {
        return Response {
            version: "HTTP/1.1".to_string(),
            status: Status::ok(),
            headers: HashMap::new(),
            body: Vec::new(),
        };
    }

    fn set_status(&mut self, status: Status) {
        self.status = status;
    }
    fn set_header(&mut self, header_name: &str, value: &str) {
        self.headers.insert(header_name.to_string(), value.to_string());
    }

    fn set_body_bytes(&mut self, body: &[u8]) {
        self.body.extend_from_slice(body);
        let value = self.body.len().to_string();
        self.set_header("Content-Length", &value);
    }

    fn set_body(&mut self, body: &str) {
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
        b.extend_from_slice("\r\n".as_bytes());
        return b;
    }
}

struct Request {
    version: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    path: String,
}

impl Request {

}

#[derive(PartialEq)]
enum State {
    ParseRequestLine,
    ParseHeaders,
    ParseBody,
}

pub struct Connection {
    socket: TcpStream,
    data: Vec<u8>,
    parsed: usize,
    //request: Request,
    response: Response,
    state: State,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        return Connection {
            state: State::ParseRequestLine,
            socket: socket,
            data: Vec::new(),
            parsed: 0,
            response: Response::new(),
        };
    }

    pub fn process(&mut self) {
        self.socket.read_to_end(&mut self.data);
        if self.state == State::ParseRequestLine {
        }
        self.response.set_status(Status::ok());
        self.response.set_header("Content-Type", "text/html");
        self.response.set_body("<h1>Yiiiiihaaaaa</h1>");
        self.socket.write(&self.response.as_bytes());
    }
}
