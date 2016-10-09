use mio::*;
use mio::tcp::*;
use std::thread;
use std::sync::mpsc::*;
use std::net::SocketAddr;
use std::str::FromStr;

const SERVER: Token = Token(0);

pub trait EventHandler : Send + 'static {
    fn new_conn(&mut self, id: usize, conn: TcpStream);
    fn conn_event(&mut self, id: usize, event: Ready);
    fn duplicate(&self) -> Box<EventHandler>;
}

#[derive(Debug)]
enum Msg {
    NewConn(usize, TcpStream),
    ConnEvent(usize, Ready),
}

pub struct EventLoop {
    host: String,
    num_workers: usize,
    event_handler: Box<EventHandler>,
}

impl EventLoop {
    pub fn new(host: &str, num_workers: usize, event_handler: Box<EventHandler>) -> EventLoop {
        return EventLoop {
            host: host.to_string(),
            num_workers: num_workers,
            event_handler: event_handler,
        };
    }

    fn process_events(channel: Receiver<Msg>, mut event_handler: Box<EventHandler>) {
        loop {
            let msg = channel.recv().unwrap();
            match msg {
                Msg::NewConn(id, conn) => {
                    event_handler.new_conn(id, conn);
                }
                Msg::ConnEvent(id, event) => {
                    event_handler.conn_event(id, event);
                }
            }
        }
    }

    pub fn run(self) {
        let poll = Poll::new().unwrap();
        let server = TcpListener::bind(&SocketAddr::from_str(&self.host).unwrap()).unwrap();
        poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).unwrap();
        let mut events = Events::with_capacity(1024);
        let mut next_conn: usize = 1;
        let mut workers = Vec::new();
        // Create worker threads.
        for _ in 0..self.num_workers {
            let (tx, rx) = channel();
            let worker_handler = self.event_handler.duplicate();
            thread::spawn(move || {
                Self::process_events(rx, worker_handler);
            });
            workers.push(tx);
        }
        loop {
            match poll.poll(&mut events, None) {
                Err(e) => panic!("Error during poll(): {}", e),
                Ok(_) => {}
            }
            for event in events.iter() {
                match event.token() {
                    SERVER => {
                        match server.accept() {
                            Ok((stream, _)) => {
                                match poll.register(&stream,
                                                    Token(next_conn),
                                                    Ready::all(),
                                                    PollOpt::edge()) {
                                    Err(e) => panic!("Error during register(): {}", e),
                                    Ok(_) => {
                                        workers[next_conn % self.num_workers]
                                            .send(Msg::NewConn(next_conn, stream))
                                            .unwrap();
                                        next_conn += 1;
                                    }
                                }
                            }
                            Err(e) => panic!("Error during accept() : {}", e),
                        }
                    }
                    Token(id) => {
                        workers[id % self.num_workers]
                            .send(Msg::ConnEvent(id, event.kind()))
                            .unwrap();
                    }
                }
            }
        }
    }
}
