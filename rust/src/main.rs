use std::collections::HashMap;
use std::{env, thread};
use std::borrow::Cow;
use std::io::prelude::*;
use std::io::Split;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::time::Duration;

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
fn server_listener(server_address: &str) {
    let listener = TcpListener::bind(server_address).expect("Binding failed...");
    println!("Server listening on {}", server_address);

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

fn handle_client_connection(mut client_stream: TcpStream) {
    println!("[*] Received connection request from {:?}", client_stream);

    let mut buffer = [0; 1024];
    client_stream.write("Please enter [dest]\n".as_bytes()).unwrap();

    match client_stream.read(&mut buffer) {
        Ok(_) => {
            let message = String::from_utf8_lossy(&buffer[..]);
            dest_connection(client_stream, &message[..]);
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", client_stream.peer_addr().unwrap());
            client_stream.shutdown(Shutdown::Both).expect("Shutdown failed...");
        }
    } {}
}

fn dest_connection(mut client_stream: TcpStream, dest_addr: &str) {
    if let Ok(dest_stream) = TcpStream::connect(dest_addr) {
        println!("Connected to dest addr {}", dest_addr);
        handle_dest_connection(client_stream, dest_stream);
    } else {
        println!("Couldn't connect to dest addr {}", dest_addr);
    }
}

fn handle_dest_connection(mut client_stream: TcpStream, mut dest_stream: TcpStream) {
    let mut dest_buffer = [0; 1024];
    let mut client_buffer = [0; 1024];

    //request something from dest_server
    client_stream.write("Enter message\n".as_bytes()).unwrap();


    let mut clone_dest = dest_stream.try_clone().unwrap();
    let mut clone_client = client_stream.try_clone().unwrap();

    thread::spawn(|| {
        'client: while match client_stream.read(&mut client_buffer) {
            Ok(_size) => {
                if client_buffer == "end\n".as_bytes() {
                    break 'client;
                }
                dest_stream.write(&mut client_buffer).unwrap();
                thread::sleep(Duration::from_millis(1));
                true
            },
            Err(e) => {
                println!("Error {} occurred...", e);
                break 'client;
            }
        } {}
    });

    'dest: while match clone_dest.read(&mut dest_buffer) {
        Ok(_size) => {
            if dest_buffer == "end\n".as_bytes() {
                break 'dest;
            }
            clone_client.write(&mut dest_buffer).unwrap();
            thread::sleep(Duration::from_millis(1));
            true
        },
        Err(e) => {
            println!("Error {} occurred...", e);
            break 'dest;
        }
    } {}
}