extern crate mio;
extern crate regex;

pub mod http;
mod event_loop;
mod app_server;

use std::io::prelude::*;
use mio::tcp::*;
use regex::Regex;
use http::{Request, Response, RequestBuilder};
use app_server::*;

pub trait Handler : Send + 'static {
    fn process(&mut self, request: Request, response: &mut Response);
    fn duplicate(&self) -> Box<Handler>;
}

struct HandlerRule(Regex, Box<Handler>);

struct HandlerApp {
    handlers: Vec<HandlerRule>,
    builder: RequestBuilder,
}
impl HandlerApp {
    fn new(handlers: Vec<HandlerRule>) -> HandlerApp {
        return HandlerApp {
            handlers: handlers,
            builder: RequestBuilder::new(),
        };
    }
}
impl App for HandlerApp {
    fn handle(&mut self, stream: &mut TcpStream) {
        let mut data = Vec::new();
        let _ = stream.read_to_end(&mut data);
        if let Some(r) = self.builder.read(&data) {
            let resp = &mut Response::new(stream);
            for &mut HandlerRule(ref regex, ref mut handler) in &mut self.handlers {
                if regex.is_match(&r.uri) {
                    handler.process(r, resp);
                    break;
                }
            }
            resp.set_not_found().send();
        }
    }
    fn duplicate(&self) -> Box<App> {
        let mut handlers = Vec::new();
        for &HandlerRule(ref r, ref h) in &self.handlers {
            handlers.push(HandlerRule(r.clone(), h.duplicate()));
        }
        Box::new(HandlerApp {
            handlers: handlers,
            builder: RequestBuilder::new(),
        })
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
        let app_server = AppServer::new(&self.host,
                                        self.num_workers,
                                        Box::new(HandlerApp::new(self.handlers)));
        app_server.run();
    }
}
