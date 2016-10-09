extern crate mio;
extern crate regex;

pub mod http;
mod event_loop;
mod app_server;
pub mod http_file;
pub mod handlers;
pub mod handler_lib;

use app_server::*;
use handler_lib::*;

pub struct WebServer {
    host: String,
    handlers: Vec<HandlerRoute>,
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
        self.handlers.push(HandlerRoute(format!("^{}$", pattern), Box::new(handler)));
    }

    pub fn run(self) {
        let app_server = AppServer::new(&self.host,
                                        self.num_workers,
                                        Box::new(HandlerApp::new(self.handlers)));
        app_server.run();
    }
}
