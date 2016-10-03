extern crate threadpool;
extern crate mio;

mod http;

use std::io::prelude::*;
use std::thread;
use std::thread::JoinHandle;
use threadpool::ThreadPool;
use std::sync::mpsc::*;
use mio::*;
use mio::tcp::*;
use std::collections::HashMap;

const SERVER: Token = Token(0);

enum Msg {
    Stop,
}

struct Context {
    host: String,
    channel: Receiver<Msg>,
}

pub struct WebServer {
    server_thread: JoinHandle<()>,
    channel: Sender<Msg>,
}

impl WebServer {
    pub fn new(host: &str) -> WebServer {
        let (tx, rx) = channel();
        let context = Context {
            host: host.to_string(),
            channel: rx,
        };
        let server_thread = thread::spawn(move || Self::server_main(context));
        return WebServer {
            server_thread: server_thread,
            channel: tx,
        };
    }

    pub fn stop(self) {
        self.channel.send(Msg::Stop);
        self.server_thread.join();
    }

    pub fn join(self) {
        self.server_thread.join();
    }

    fn server_main(context: Context) {
        let mut poll = Poll::new().unwrap();
        let addr = context.host.parse().unwrap();
        let server = TcpListener::bind(&addr).unwrap();
        poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).unwrap();
        let mut events = Events::with_capacity(1024);
        let mut clients: HashMap<usize, http::Connection> = HashMap::new();
        let mut next_client: usize = 1;
        loop {
            let msg = context.channel.try_recv();
            match msg {
                Ok(Msg::Stop) => break,
                Err(TryRecvError::Disconnected) => {
                    panic!("server_main thread disconnected from main thread!");
                }
                _ => {}
            }
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
                                clients.insert(next_client, http::Connection::new(stream));
                                next_client += 1;
                            }
                            Err(e) => panic!("Error during accept() : {}", e),
                        }
                    }
                    Token(ref id) => {
                        match clients.get_mut(id) {
                            None => panic!("client no {} can't be found in clients map!", id),
                            Some(ref mut client) => {
                                client.process();
                            }
                        }
                    }
                }
            }
        }
    }
}
