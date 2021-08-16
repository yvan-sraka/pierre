use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::thread::{JoinHandle, spawn};
use std::io::stdin;
use std::vec::Vec;
use std::time::Instant;
use std::fmt::Display;

fn strip(msg: &String, to_strip: &str) -> String { msg.strip_prefix(to_strip).unwrap().to_string() }
fn getp(cmd: String) -> String {
    let mut spaces = 0;
    for (index, &c) in cmd[..].as_bytes().iter().enumerate() {
        if c == b' ' {
            spaces += 1;
        }
        if spaces == 2 {
            return String::from(&cmd[..index]);
        }
    }
    cmd
}

fn start_server() -> (JoinHandle<()>, String) {
    println!("Open a port");
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).expect("Did not enter a correct string");
    let server_addr = String::from("127.0.0.1:") + &buffer[0..4];
    (spawn(move || {
        let listener = TcpListener::bind(&server_addr[..]).unwrap();
        println!("server started");
        let mut streams_index: Vec::<String> = Vec::new();
        let mut history: Vec::<String> = Vec::new();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            if handle(stream, &streams_index, &history) { break; }
        }
    }), buffer[0..4].to_string())
}

fn handle(mut stream: TcpStream,
        mut streams_index: &Vec::<String>,
        mut history: &Vec::<String>) -> bool {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let content = get_msg(buffer);
    let peer_addr = stream.peer_addr().unwrap();
    println!("msg from {}", peer_addr);
    if content.starts_with("i ") {
        let mut command = String::from("p ");
        command += &Instant::now().elapsed().as_secs().to_string()[..];
        command.push(' ');
        history.push(command);
        propagate(&command[..], strip(&content, "i "), streams_index);
    } else if content.starts_with("p ") {
        let command = getp(content);
        for h in history {
            if command.eq(h) {
                return false;
            }
        }
        let options = strip(&content, &command[..]);
        println!("{}", options);
        propagate(&content[..], options, streams_index);
    } else if content.starts_with("connect ") {
        let addr = strip(&content, "connect ");
        streams_index.push(addr);
        propagate("connection", String::new(), streams_index);
    } else if content.starts_with("connection") {
        println!("connection {}", peer_addr);
        streams_index.push(peer_addr.to_string());
    }
    content.eq(&String::from("q"))
}

/// todo:
/// Propagate a command with another strategy ?
fn propagate(command: &str, options: String, streams_index: &Vec::<String>) {
    for stream in streams_index {
        send(command.to_owned() + &options[..],
            &stream.parse::<SocketAddr>().unwrap());
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
