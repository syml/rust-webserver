extern crate webserver;

use webserver::{WebServer, Handler};
use webserver::http::{Request, Response, Status};
use std::thread::*;
use std::time::*;
use webserver::http_file::*;

struct FileSystemHandler {
    path: String,
    fs: FileSystem,
}
impl FileSystemHandler {
    fn new(path: &str) -> FileSystemHandler {
        FileSystemHandler {
            path: path.to_string(),
            fs: FileSystem::new(path),
        }
    }
}
impl Handler for FileSystemHandler {
    fn process(&mut self, req: Request, resp: &mut Response) {
        self.fs.serve(&req.uri, resp);
    }
    fn duplicate(&self) -> Box<Handler> {
        return Box::new(FileSystemHandler::new(&self.path));
    }
}

struct FileHandler {
    path: String,
    fs: FileSystem,
}
impl FileHandler {
    fn new(path: &str) -> FileHandler {
        FileHandler {
            path: path.to_string(),
            fs: FileSystem::new(path),
        }
    }
}
impl Handler for FileHandler {
    fn process(&mut self, _: Request, resp: &mut Response) {
        self.fs.serve("", resp);
    }
    fn duplicate(&self) -> Box<Handler> {
        return Box::new(FileHandler::new(&self.path));
    }
}

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
