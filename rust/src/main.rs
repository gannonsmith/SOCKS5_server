use std::collections::HashMap;
use std::{env, thread};
use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};

struct Server {
    server: TcpListener,
    next_token: usize,
    clients: HashMap<Token, Client>
}

struct Client {
    stream: TcpStream,
    state: State,
    token: Token
}

// function arguments:
// cargo run
fn main() {

    // client connects to server
    server_listener("127.0.0.1:8000");

    // establish a tcp connection for you

    // proxies are able to transport udp packets across the connection
    // send udp packets to the server, which are then forwarded to recipient through a tcp connection
}

// conventionally uses port 1080
fn server_listener(address: &str) {
    let listener = TcpListener::bing(address).expect("Binding failed...");
    println!("Server listening on {}", address);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().expect("Connection failed..."));
                thread::spawn(move|| {
                    handle_client_connection(stream);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

fn handle_client_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.write("Please enter [dest_addr]\n".as_bytes()).unwrap();

    match stream.read(&mut buffer) {
        Ok(_) => {
            let dest_addr = String::from_utf8_lossy(&buffer[..]);
        },
        Err(_) => {
            println!("An error occured, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).expect("Shutdown failed...");
        }
    } {}
}

fn dest_connection(dest_addr: &str) {

}