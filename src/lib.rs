extern crate threadpool;
extern crate mio;
extern crate regex;

pub mod http;

use std::io::prelude::*;
use threadpool::ThreadPool;
use mio::*;
use mio::tcp::*;
use std::collections::HashMap;
use regex::Regex;
use http::{Request, Response, Status, Connection};
use std::sync::mpsc::*;
use std::thread;

const SERVER: Token = Token(0);

pub trait Handler : Send + 'static {
    fn process(&mut self, request: Request, connection: &mut Connection);
    fn duplicate(&self) -> Box<Handler>;
}

type HandlerRule = (Regex, Box<Handler>);

#[derive(Debug)]
enum Msg {
    NewClient(usize, TcpStream),
    ClientReady(usize),
}

pub struct WebServer {
    host: String,
    handlers: Vec<HandlerRule>,
    num_workers: usize,
}

impl WebServer {
    pub fn new(host: &str, num_workers: usize) -> WebServer {
        return WebServer {
            host: host.to_string(),
            handlers: Vec::new(),
            num_workers: num_workers,
        };
    }

    pub fn add_handler<T>(&mut self, pattern: &str, handler: T)
        where T: Handler
    {
        self.handlers.push((Regex::new(&format!("^{}$", pattern)).unwrap(),
                            Box::new(handler)));
    }

    fn process_clients(channel: Receiver<Msg>, mut handlers: Vec<HandlerRule>) {
        let mut clients: HashMap<usize, Connection> = HashMap::new();
        loop {
            let msg = channel.recv().unwrap();
            match msg {
                Msg::NewClient(id, client) => {
                    clients.insert(id, Connection::new(client));
                }
                Msg::ClientReady(id) => {
                    match clients.get_mut(&id) {
                        None => panic!("client no {} can't be found in clients map!", id),
                        Some(ref mut client) => {
                            match client.read() {
                                None => {}
                                Some(r) => {
                                    for &mut (ref regex, ref mut handler) in &mut handlers {
                                        if regex.is_match(&r.uri) {
                                            handler.process(r, client);
                                            break;
                                        }
                                    }
                                    client.write(Response::not_found());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn run(self) {
        let mut poll = Poll::new().unwrap();
        let addr = self.host.parse().unwrap();
        let server = TcpListener::bind(&addr).unwrap();
        poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).unwrap();
        let mut events = Events::with_capacity(1024);
        let mut next_client: usize = 1;
        let mut workers = Vec::new();
        // Create worker threads.
        for i in 0..self.num_workers {
            let (tx, rx) = channel();
            let mut handlers = Vec::new();
            for &(ref r, ref h) in &self.handlers {
                handlers.push((r.clone(), h.duplicate()));
            }
            let worker = thread::spawn(move || {
                Self::process_clients(rx, handlers);
            });
            workers.push(tx);
        }
        loop {
            let poll_error = poll.poll(&mut events, None);
            match poll_error {
                Err(e) => panic!("Error during poll(): {}", e),
                Ok(_) => {}
            }
            for event in events.iter() {
                match event.token() {
                    SERVER => {
                        let stream_ok = server.accept();
                        match stream_ok {
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
