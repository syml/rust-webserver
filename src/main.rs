extern crate webserver;

use webserver::WebServer;
use webserver::http::{Request, Response, Status};

fn main() {
    let mut server = WebServer::new("127.0.0.1:8080");
    server.add_handler("/secret", |r: Request| -> Response {
        let mut resp = Response::new();
        resp.set_status(Status::ok());
        resp.set_header("Content-Type", "text/html");
        resp.set_body(&format!("<h1>Secret!</h1>"));
        return resp;
    });
    server.add_handler("/.*", |r: Request| -> Response {
        let mut resp = Response::new();
        resp.set_status(Status::ok());
        resp.set_header("Content-Type", "text/html");
        resp.set_body(&format!("<h1>Sylvain Server</h1>You requested: {}", r.uri));
        return resp;
    });
    server.run();
}
