extern crate webserver;

use webserver::handlers::*;
use webserver::*;

fn main() {
    let mut server = WebServer::new("127.0.0.1:8080", 4);
    server.add_handler("/", FileHandler::new("http/index.html"));
    server.add_handler("/.*", FileSystemHandler::new("http"));
    server.run();
}
