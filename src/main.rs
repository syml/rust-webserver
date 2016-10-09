extern crate webserver;

use std::thread::*;
use std::time::*;
use webserver::handlers::*;
use webserver::handler_lib::*;
use webserver::http::*;
use webserver::*;

struct LongHandler;
impl Handler for LongHandler {
    fn process(&mut self, r: Request, resp: &mut Response) {
        resp.set_status(Status::ok())
            .set_header("Content-Type", "text/html")
            .set_body_str(&format!("<h1>Long page!!</h1>This is very {}", r.uri))
            .set_header("Content-Length", &format!("{}", 100000 + 38))
            .send();
        for _ in 0..10000 {
            resp.send_str("sunny<3<br/>");
            sleep(Duration::from_millis(100));
        }
    }
    fn duplicate(&self) -> Box<Handler> {
        return Box::new(LongHandler);
    }
}

fn main() {
    let mut server = WebServer::new("127.0.0.1:8080", 4);
    server.add_handler("/long", LongHandler);
    server.add_handler("/", FileHandler::new("http/index.html"));
    server.add_handler("/.*", FileSystemHandler::new("http"));
    server.run();
}
