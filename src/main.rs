extern crate webserver;

use webserver::{WebServer, Handler};
use webserver::http::{Request, Response, Status};

struct MainHandler;
impl Handler for MainHandler {
    fn process(&mut self, r: Request) -> Response {
        let mut resp = Response::new();
        resp.set_status(Status::ok());
        resp.set_header("Content-Type", "text/html");
        resp.set_body(&format!("<h1>Sylvain Server</h1>You requested: {}", r.uri));
        return resp;
    }
    fn duplicate(&self) -> Box<Handler> {
        return Box::new(MainHandler{});
    }
}

fn main() {
    let mut server = WebServer::new("127.0.0.1:8080", 4);
    server.add_handler("/*", MainHandler);
    server.run();
}
