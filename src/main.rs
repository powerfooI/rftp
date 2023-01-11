mod arg_parser;
use epoll;
use std::{
    io::{prelude::*, BufReader, Write},
    net::{TcpListener, TcpStream},
    os::fd::{AsRawFd, RawFd},
};

const READ_FLAGS: i32 = epoll::Events::EPOLLONESHOT | epoll::Events::EPOLLIN;
const WRITE_FLAGS: i32 = epoll::Events::EPOLLONESHOT | epoll::Events::EPOLLOUT;

fn handle_client(mut stream: TcpStream) -> Result<(), std::io::Error> {
    let buf_reader = BufReader::new(&mut stream);

    let buf: Vec<_> = buf_reader
        .lines()
        .map(|l| l.unwrap())
        .take_while(|l| !l.is_empty())
        .collect();
    println!("{buf:#?}");

    stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n")
}

/// create read event
fn epoll_read_event(key: u64) -> epoll::Event {
    epoll::Event {
        events: READ_FLAGS,
        data: key,
    }
}

#[derive(Debug)]
pub struct RequestContext {
    pub stream: TcpStream,
    pub content_length: usize,
    pub buf: Vec<u8>,
}

impl RequestContext {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buf: Vec::new(),
            content_length: 0,
        }
    }
}

fn add_interest(epfd: RawFd, fd: RawFd, event: epoll::Event) -> io::Result<()> {
    epoll::ctl(epfd, epoll::ControlOptions::EPOLL_CTL_ADD, fd, event);
    Ok(())
}

fn modify_interest(epfd: RawFd, fd: RawFd, event: epoll::Event) -> io::Result<()> {
    epoll::ctl(epfd, epoll::ControlOptions::EPOLL_CTL_MOD, fd, event);
    Ok(())
}

fn delete_interest(epfd: RawFd, fd: RawFd, event: epoll::Event) -> io::Result<()> {
    epoll::ctl(epfd, epoll::ControlOptions::EPOLL_CTL_DEL, fd, event);
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args = arg_parser::Args::parse_args();
    println!("{args:?}");

    let listener = TcpListener::bind("127.0.0.1:8180")?;
    listener.set_nonblocking(true)?;

    let epoll_fd = epoll::create(true).expect("failed to create epoll fd");

    let mut ep_key = 100;

    add_interest(epfd, fd, epoll_read_event(ep_key))?;

    let mut events: Vec<epoll::Event> = Vec::with_capacity(1024);
    let mut request_contexts: HashMap<u64, RequestContext> = HashMap::new();

    loop {
        events.clear();
        let req = match epoll::wait(epoll_fd, 1000, events.as_mut_ptr()) {
            Ok(v) => v,
            Err(e) => panic!("error during epoll wait: ", e),
        };
        unsafe {
            events.set_len(req);
        }
        println!("requests in flight: {}", request_contexts.len());
        for ev in &events {
            match ev.data {
                100 => {
                    match listener.accept() {
                        Ok((stream, addr)) => {
                            stream.set_nonblocking(true)?;
                            println!("new client: {}", addr);
                            ep_key += 1;
                            add_interest(epfd, stream.as_raw_fd(), epoll_read_event(ep_key))?;
                            request_contexts.insert(ep_key, RequestContext::new(stream));
                        }
                        Err(e) => eprintln!("failed to accept socket: {}", e),
                    };
                    modify_interest(epfd, listener.as_raw_fd(), epoll_read_event(100))?;
                },
                key => {
                    let mut to_delete = None;
                    if let Some(context) = request_contexts.get_mut(&key) {
                        let events: u32 = ev.events;
                        match events {
                            v if v as i32 & epoll::Events::EPOLLIN == epoll::Events::EPOLLIN => {
                                println!("read event");
                            }
                            v if v as i32 & epoll::Events::EPOLLOUT == epoll::Events::EPOLLOUT => {
                                println!("write event");
                                to_delete = Some(key);
                            }
                            v => println!("unexpected events: {}", v),
                        };
                    }
                    if let Some(key) = to_delete {
                        request_contexts.remove(&key);
                    }
                }
            };
        }
    }

    // accept connections and process them serially
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        if let Err(e) = handle_client(stream) {
            println!("{e}");
        }
    }
    Ok(())
}
