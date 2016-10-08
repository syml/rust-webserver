use mio::*;
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
    conns: HashMap<usize, AppWithStream>,
}
impl AppEventHandler {
    fn new(app: Box<App>) -> AppEventHandler {
        return AppEventHandler {
            app: app,
            conns: HashMap::new(),
        };
    }
}
impl EventHandler for AppEventHandler {
    fn new_conn(&mut self, id: usize, stream: TcpStream) {
        self.conns.insert(id, AppWithStream::new(self.app.duplicate(), stream));
    }
    fn conn_event(&mut self, id: usize, event: Ready) {
        if event.is_readable() {
            match self.conns.get_mut(&id) {
                None => panic!("conn no {} can't be found in conns map!", id),
                Some(ref mut conn) => {
                    conn.handle();
                }
            }
        }
        if event.is_error() || event.is_hup() {
            self.conns.remove(&id);
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
