use mio::tcp::*;
use std::collections::HashMap;
use event_loop::*;

pub trait App : Send + 'static {
    fn handle(&mut self, stream: &mut TcpStream);
    fn duplicate(&self) -> Box<App>;
}

struct AppWithStream {
    app: Box<App>,
    stream: TcpStream,
}
impl AppWithStream {
    fn new(app: Box<App>, stream: TcpStream) -> AppWithStream {
        AppWithStream {
            app: app,
            stream: stream,
        }
    }
    fn handle(&mut self) {
        self.app.handle(&mut self.stream);
    }
}

struct AppEventHandler {
    app: Box<App>,
    clients: HashMap<usize, AppWithStream>,
}
impl AppEventHandler {
    fn new(app: Box<App>) -> AppEventHandler {
        return AppEventHandler {
            app: app,
            clients: HashMap::new(),
        };
    }
}
impl EventHandler for AppEventHandler {
    fn new_client(&mut self, id: usize, stream: TcpStream) {
        self.clients.insert(id, AppWithStream::new(self.app.duplicate(), stream));
    }
    fn client_ready(&mut self, id: usize) {
        match self.clients.get_mut(&id) {
            None => panic!("client no {} can't be found in clients map!", id),
            Some(ref mut client) => {
                client.handle();
            }
        }
    }
    fn duplicate(&self) -> Box<EventHandler> {
        return Box::new(AppEventHandler::new(self.app.duplicate()));
    }
}

pub struct AppServer {
    host: String,
    num_workers: usize,
    app: Box<App>,
}
impl AppServer {
    pub fn new(host: &str, num_workers: usize, app: Box<App>) -> AppServer {
        return AppServer {
            host: host.to_string(),
            num_workers: num_workers,
            app: app,
        };
    }

    pub fn run(self) {
        let l = EventLoop::new(&self.host,
                               self.num_workers,
                               Box::new(AppEventHandler::new(self.app)));
        l.run();
    }
}
