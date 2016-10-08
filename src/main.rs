extern crate webserver;

use webserver::{WebServer, Handler};
use webserver::http::{Request, Response, Status};
use std::thread::*;
use std::time::*;

struct MainHandler;
impl Handler for MainHandler {
    fn process(&mut self, r: Request, resp: &mut Response) {
        resp.set_status(Status::ok());
        resp.set_header("Content-Type", "text/html");
        resp.set_body(&format!("<h1>Sylvain Server</h1>You requested: {}", r.uri));
        resp.flush();
    }
    fn duplicate(&self) -> Box<Handler> {
        return Box::new(MainHandler);
    }
}

struct SecretHandler;
impl Handler for SecretHandler {
    fn process(&mut self, r: Request, resp: &mut Response) {
        resp.set_status(Status::ok());
        resp.set_header("Content-Type", "text/html");
        resp.set_body(&format!("<h1>Secret page!!</h1>This is very {}", r.uri));
        resp.flush();
    }
    fn duplicate(&self) -> Box<Handler> {
        return Box::new(SecretHandler);
    }
}

struct LongHandler;
impl Handler for LongHandler {
    fn process(&mut self, r: Request, resp: &mut Response) {
        resp.set_status(Status::ok());
        resp.set_header("Content-Type", "text/html");
        resp.set_body(&format!("<h1>Long page!!</h1>This is very {}", r.uri));
        resp.set_header("Content-Length", &format!("{}", 100000 + 38));
        resp.flush();
        for _ in 0..10000 {
            resp.write_str("sunny<3<br/>");
            sleep(Duration::from_millis(100));
        }
    }
    fn duplicate(&self) -> Box<Handler> {
        return Box::new(LongHandler);
    }
}

fn main() {
    let mut server = WebServer::new("127.0.0.1:8080", 4);
    server.add_handler("/secret", SecretHandler);
    server.add_handler("/", MainHandler);
    server.add_handler("/long", LongHandler);
    server.run();
}
