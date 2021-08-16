use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::thread::{JoinHandle, spawn};
use std::io::stdin;
use std::sync::{Arc, Mutex};
use std::vec::Vec;

static mut KNOWN_STREAMS: std::vec::Vec::<SocketAddr> = Vec::new();

fn start_server() -> (JoinHandle<()>, String) {
    println!("Open a port");
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).expect("Did not enter a correct string");
    let server_addr = String::from("127.0.0.1:") + &buffer[0..4];
    (spawn(move || {
        let listener = TcpListener::bind(&server_addr[..]).unwrap();
        println!("server started");
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            if handle(stream) { break; }
        }
    }), buffer[0..4].to_string())
}

fn strip(msg: &String, to_strip: &str) -> String {
    msg.strip_prefix(to_strip).unwrap().to_string()
}

fn handle(mut stream: TcpStream) -> bool {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let content = get_msg(buffer);
    let peer_addr = stream.peer_addr().unwrap();
    println!("msg from {}", peer_addr);
    if content.starts_with("i ") {
        propagate("p ", strip(&content, "i "));
    } else if content.starts_with("p ") {
        println!("{}", strip(&content, "p "));
    } else if content.starts_with("connect") {
        println!("receiving connect");
        let addr = strip(&content, "connect ");
        println!("{}", addr);
        send("connection_req".to_owned(), &addr.parse::<SocketAddr>().unwrap());
    } else if content.starts_with("connection_req") {
        println!("receiving connection_req");
        unsafe {
            KNOWN_STREAMS.push(peer_addr);
        }
        propagate("connection", peer_addr.to_string());
    } else if content.starts_with("connection") {
        println!("receiving connection");
        let addr = strip(&content, "connection");
        println!("receiving connection {}", addr);
    }
    content.eq(&String::from("q"))
}

/// todo:
/// Propagate a command with another strategy
fn propagate(command: &str, content: String) {
    unsafe {
        for known_stream in KNOWN_STREAMS.clone() {
            send(command.to_owned() + &content[..], &known_stream);
        }
    }
}

fn run_client(port: String) -> std::io::Result<()> {
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Did not enter a correct string");
        let local_srv = &SocketAddr::from(([127, 0, 0, 1],
            port[..4].parse::<u16>().unwrap()));
        if input.starts_with("connect") {
            send(input, local_srv);
        }
    }
}

fn send(content: String, addr: &SocketAddr) -> bool {
    let mut stream = TcpStream::connect_timeout(addr, std::time::Duration::new(30,0)).expect("wow");
    stream.write(content.as_bytes());
    content.eq(&String::from("q\n"))
}

fn main() {
    let (jh, port) = start_server();
    println!("port {}", port);
    run_client(port).expect("Failed running client");
    jh.join().expect("Failed joining server thread");
}

fn get_msg(bytes: [u8; 1024]) -> String {
    let mut ret = String::new();
    for &c in bytes.iter() {
        if c == b'\0' || c == b'\n' { break; }
        ret += &String::from_utf8(vec![c]).unwrap()[..];
    }
    ret
}
