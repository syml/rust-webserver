extern crate mio;
extern crate regex;

pub mod http;
mod event_loop;

use mio::tcp::*;
use std::collections::HashMap;
use regex::Regex;
use http::{Request, Response, Connection};
use event_loop::*;

pub trait Handler : Send + 'static {
    fn process(&mut self, request: Request, response: &mut Response);
    fn duplicate(&self) -> Box<Handler>;
}

struct HandlerRule(Regex, Box<Handler>);

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
                        let resp = &mut Response::new(client);
                        for &mut HandlerRule(ref regex, ref mut handler) in &mut self.handlers {
                            if regex.is_match(&r.uri) {
                                handler.process(r, resp);
                                break;
                            }
                        }
                        resp.not_found();
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
        let event_loop = EventLoop::new(&self.host,
                                        self.num_workers,
                                        Box::new(WebServerEventHandler::new(self.handlers)));
        event_loop.run();
    }
}
