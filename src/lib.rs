extern crate threadpool;
extern crate mio;
extern crate regex;

pub mod http;

use std::io::prelude::*;
use threadpool::ThreadPool;
use mio::*;
use mio::tcp::*;
use std::collections::HashMap;
use regex::Regex;
use http::{Request, Response, Status, Connection};

const SERVER: Token = Token(0);

type Handler = Box<Fn(Request) -> Response>;

pub struct WebServer {
    host: String,
    handlers: Vec<(Regex, Handler)>,
}

impl WebServer {
    pub fn new(host: &str) -> WebServer {
        return WebServer {
            host: host.to_string(),
            handlers: Vec::new(),
        };
    }

    pub fn add_handler<T>(&mut self, pattern: &str, handler: T)
        where T: Fn(Request) -> Response + 'static
    {
        self.handlers.push((Regex::new(&format!("^{}$", pattern)).unwrap(),
                            Box::new(handler)));
    }

    pub fn run(self) {
        let mut poll = Poll::new().unwrap();
        let addr = self.host.parse().unwrap();
        let server = TcpListener::bind(&addr).unwrap();
        poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).unwrap();
        let mut events = Events::with_capacity(1024);
        let mut clients: HashMap<usize, Connection> = HashMap::new();
        let mut next_client: usize = 1;
        loop {
            let poll_error = poll.poll(&mut events, None);
            match poll_error {
                Err(e) => panic!("Error during poll(): {}", e),
                Ok(_) => {}
            }
            for event in events.iter() {
                match event.token() {
                    SERVER => {
                        let stream_ok = server.accept();
                        match stream_ok {
                            Ok((stream, _)) => {
                                match poll.register(&stream,
                                                    Token(next_client),
                                                    Ready::readable(),
                                                    PollOpt::edge()) {
                                    Err(e) => panic!("Error during register(): {}", e),
                                    _ => {}
                                }
                                clients.insert(next_client, Connection::new(stream));
                                next_client += 1;
                            }
                            Err(e) => panic!("Error during accept() : {}", e),
                        }
                    }
                    Token(ref id) => {
                        match clients.get_mut(id) {
                            None => panic!("client no {} can't be found in clients map!", id),
                            Some(ref mut client) => {
                                match client.read() {
                                    None => {}
                                    Some(r) => {
                                        for &(ref regex, ref handler) in &self.handlers {
                                            if regex.is_match(&r.uri) {
                                                let resp = handler(r);
                                                client.write(resp);
                                                break;
                                            }
                                        }
                                        client.write(Response::not_found());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
