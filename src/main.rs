extern crate webserver;

use webserver::WebServer;

fn main() {
    let server = WebServer::new("127.0.0.1:8080");
    server.join();
}
