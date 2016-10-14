use std::io::prelude::*;
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
    fn shutdown(&self) {
        let _ = self.stream.shutdown(Shutdown::Both);
    }
    fn flush(&mut self) {
        let mut buf = Vec::new();
        let _ = self.stream.read_to_end(&mut buf);
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
        println!("Got new connection {}", id);
        self.conns.insert(id, AppWithStream::new(self.app.duplicate(), stream));
    }
    fn conn_event(&mut self, id: usize, event: Ready) {
        println!("Handling event!");
        if event.is_error() || event.is_hup() {
            if event.is_error() {
                println!("Error event on conn {}", id);
            } else {
                println!("Hangup event on conn {}", id)
            }
            match self.conns.remove(&id) {
                None => {
                    println!("WARNING: conn no {} can't be found in conns map for shutdown event!",
                             id)
                }
                Some(mut conn) => {
                    if event.is_readable() {
                        conn.flush();
                    }
                    conn.shutdown();
                    self.conns.remove(&id);
                    return;
                }
            }
        }
        if event.is_readable() {
            match self.conns.get_mut(&id) {
                None => {
                    println!("WARNING: conn no {} can't be found in conns map for read event!",
                             id)
                }
                Some(ref mut conn) => {
                    println!("Handling connection {}", id);
                    conn.handle();
                }
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
