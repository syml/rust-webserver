use std::io::prelude::*;
use mio::tcp::*;
use http::*;
use app_server::*;
pub use regex::Regex;

pub trait Handler : Send + 'static {
    fn process(&mut self, request: Request, response: &mut Response);
    fn duplicate(&self) -> Box<Handler>;
}

struct HandlerRule(Regex, Box<Handler>);
pub struct HandlerRoute(pub String, pub Box<Handler>);

pub struct HandlerApp {
    handlers: Vec<HandlerRule>,
    builder: RequestBuilder,
}
impl HandlerApp {
    pub fn new(handler_defs: Vec<HandlerRoute>) -> HandlerApp {
        let mut handlers = Vec::new();
        for &HandlerRoute(ref s, ref h) in &handler_defs {
            handlers.push(HandlerRule(Regex::new(&s).unwrap(), h.duplicate()));
        }
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
