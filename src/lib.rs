extern crate mio;
extern crate regex;

pub mod http;
mod event_loop;

use std::io::prelude::*;
use mio::*;
use mio::tcp::*;
use std::collections::HashMap;
use regex::Regex;
use http::{Request, Response, Status, Connection};
use std::sync::mpsc::*;
use std::thread;
use event_loop::*;

const SERVER: Token = Token(0);

pub trait Handler : Send + 'static {
    fn process(&mut self, request: Request, connection: &mut Connection);
    fn duplicate(&self) -> Box<Handler>;
}

struct HandlerRule(Regex, Box<Handler>);

#[derive(Debug)]
enum Msg {
    NewClient(usize, TcpStream),
    ClientReady(usize),
}

struct WebServerEventHandler {
    clients: HashMap<usize, Connection>,
    handlers: Vec<HandlerRule>,
}
impl WebServerEventHandler {
    fn new(handlers: Vec<HandlerRule>) -> WebServerEventHandler {
        return WebServerEventHandler {
            clients: HashMap::new(),
            handlers: handlers,
        };
    }
}
impl EventHandler for WebServerEventHandler {
    fn new_client(&mut self, id: usize, client: TcpStream) {
        self.clients.insert(id, Connection::new(client));
    }
    fn client_ready(&mut self, id: usize) {
        match self.clients.get_mut(&id) {
            None => panic!("client no {} can't be found in clients map!", id),
            Some(ref mut client) => {
                match client.read() {
                    None => {}
                    Some(r) => {
                        for &mut HandlerRule(ref regex, ref mut handler) in &mut self.handlers {
                            if regex.is_match(&r.uri) {
                                handler.process(r, client);
                                break;
                            }
                        }
                        client.write(Response::not_found());
                    }
                }
            }
        }
    }
    fn duplicate(&self) -> Box<EventHandler> {
        let mut handlers = Vec::new();
        for &HandlerRule(ref r, ref h) in &self.handlers {
            handlers.push(HandlerRule(r.clone(), h.duplicate()));
        }
        return Box::new(WebServerEventHandler::new(handlers));
    }
}

pub struct WebServer {
    host: String,
    handlers: Vec<HandlerRule>,
    num_workers: usize,
}

impl WebServer {
    pub fn new(host: &str, num_workers: usize) -> WebServer {
        return WebServer {
            host: host.to_string(),
            handlers: Vec::new(),
            num_workers: num_workers,
        };
    }

    pub fn add_handler<T>(&mut self, pattern: &str, handler: T)
        where T: Handler
    {
        self.handlers.push(HandlerRule(Regex::new(&format!("^{}$", pattern)).unwrap(),
                                       Box::new(handler)));
    }

    pub fn run(self) {
        let event_loop = EventLoop::new("127.0.0.1:8080",
                                        4,
                                        Box::new(WebServerEventHandler::new(self.handlers)));
        event_loop.run();
    }
}
