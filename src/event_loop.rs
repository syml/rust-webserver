use std::io::prelude::*;
use mio::*;
use mio::tcp::*;
use std::thread;
use std::sync::mpsc::*;
use std::net::SocketAddr;
use std::str::FromStr;

const SERVER: Token = Token(0);

pub trait EventHandler : Send + 'static {
    fn new_client(&mut self, id: usize, client: TcpStream);
    fn client_ready(&mut self, id: usize);
    fn duplicate(&self) -> Box<EventHandler>;
}

#[derive(Debug)]
enum Msg {
    NewClient(usize, TcpStream),
    ClientReady(usize),
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
                Msg::NewClient(id, client) => {
                    event_handler.new_client(id, client);
                }
                Msg::ClientReady(id) => {
                    event_handler.client_ready(id);
                }
            }
        }
    }

    pub fn run(self) {
        let mut poll = Poll::new().unwrap();
        let server = TcpListener::bind(&SocketAddr::from_str(&self.host).unwrap()).unwrap();
        poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).unwrap();
        let mut events = Events::with_capacity(1024);
        let mut next_client: usize = 1;
        let mut workers = Vec::new();
        // Create worker threads.
        for i in 0..self.num_workers {
            let (tx, rx) = channel();
            let worker_handler = self.event_handler.duplicate();
            let worker = thread::spawn(move || {
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
                                                    Token(next_client),
                                                    Ready::readable(),
                                                    PollOpt::edge()) {
                                    Err(e) => panic!("Error during register(): {}", e),
                                    _ => {}
                                }
                                workers[next_client % self.num_workers]
                                    .send(Msg::NewClient(next_client, stream));
                                next_client += 1;
                            }
                            Err(e) => panic!("Error during accept() : {}", e),
                        }
                    }
                    Token(id) => {
                        workers[id % self.num_workers].send(Msg::ClientReady(id));
                    }
                }
            }
        }
    }
}
