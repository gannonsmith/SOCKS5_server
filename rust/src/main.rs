extern crate core;

use std::io::{Read, Write};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, TcpListener, TcpStream};
use std::{thread};

const SOCKS5VER: u8 = 0x05;
const RESERVED: u8 = 0x00;
const CMDCONNECT: u8 = 0x01;

#[allow(dead_code)]
#[repr(u8)]
enum Methods {
    NoAuthenticationRequired = 0x00,
    GSSAPI = 0x01,
    UsernamePassword = 0x02,
    NoAcceptableMethods = 0xFF,
}

#[allow(dead_code)]
#[repr(u8)]
enum ReplyCode {
    Succeeded = 0x00,
    Failure = 0x01,
    NotAllowed = 0x02,
    NetworkUnreachable = 0x03,
    HostUnreachable = 0x04,
    ConnectionRefused = 0x05,
    TTLExpired = 0x06,
    CommandUnsupported = 0x07,
    AddressUnsupported = 0x08,
}

#[allow(dead_code)]
mod addr_type {
    pub const IPV4: u8 = 0x01;
    pub const DOMAIN: u8 = 0x03;
    pub const IPV6: u8 = 0x04;
}


fn main() {
    println!("Hello, world!");


    /*
    Determine TCP or UDP
     */

    // pass address to our boys in the back
    let address = "127.0.0.1:8200";
    tcp_listener(address);
    println!("Shutting down server listener");
}

fn tcp_listener(address: &str) {

    // set up listener binded to port at address
    let listener = match TcpListener::bind(address) {
        Ok(listener) => listener,
        Err(e) => {
            println!("Could not bind to address specified: {:?}", e);
            return;
        }
    };
    println!("Server listening on {}", address);

    // listen for incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let addr = match stream.peer_addr() {
                    Ok(addy) => addy,
                    Err(e) => {
                        println!("Could not peer address: {:?}", e);
                        return;
                    }
                };
                println!("New connection: {}", addr);
                thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                println!("Could not receive connection: {:?}", e);
            }
        }
    }
}

fn handle_connection(mut client_stream: TcpStream) {
    let mut nmethods_buffer: [u8; 2] = [0; 2];

    println!("Received connection request from {:?}", client_stream);


    // read [Ver: 0x05] [CMD: 0x01] [RSV: 0x00] [ATYP] [DST.ADDR] [DST.PORT]
    if client_stream.read(&mut nmethods_buffer).is_err() {
        println!("Error with reading first two bytes in request.");
        return;
    };
    //implement authentication

    if nmethods_buffer[0] != SOCKS5VER {
        println!("Error with unsupported protocol.");
        return;
        //Err(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "Only socks5 protocol is supported!"));
    }


    let nmethods = nmethods_buffer[1];
    let mut methods_vec: Vec<u8> = vec![0; nmethods as usize];

    if client_stream.read(&mut methods_vec).is_err() {
        println!("Could not read client's stream");
        return;
    }

    //is 0x00 a method available
    let mut iter = methods_vec.iter();
    if iter.find(|&&x| x == (Methods::NoAuthenticationRequired as u8)).is_none() {
        println!("Does not support methods");
        return;
    }


    if client_stream.write(&[SOCKS5VER, Methods::NoAuthenticationRequired as u8]).is_err() {
        println!("Issue reading stream.");
        return;
    }

    let mut buffer: [u8; 4] = [0; 4];
    if client_stream.read(&mut buffer).is_err() {
        println!("Issue reading commands.");
        return;
    };
    println!("buffer: {:?}", &buffer);
    // check buffer[0..3]
    println!("protocol: {}", buffer[0]);
    if buffer[0] != SOCKS5VER {
        println!("Error with unsupported protocol.");
        return;
        //Err(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "Only socks5 protocol is supported!"));
    }

    //UDP ASSOCIATE and BIND are not supported.
    if buffer[1] != CMDCONNECT {
        println!("Error with unsupported commands: {}", buffer[1]);
        return;
        //Err(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, "Only connect cmd is supported!"));
    }

    //vp4 10 bytes
    //vp6 14 bytes
    //domain 256 bytes
    //4 bytes are used for version, cmd, rsv, and atyp. That leaves # - 4 sized arrays for each address type.


    let destination_stream_result:Result<TcpStream, ReplyCode> = match buffer[3] {
        addr_type::IPV4 => {
            let mut buffer_v4: [u8; 7] = [0; 7];
            if client_stream.read(&mut buffer_v4).is_err() {
                println!("Error reading IPV4 address.");
                return;
            };
            ipv4_connection(&mut buffer_v4)
        }
        addr_type::DOMAIN => {
            domain_name_connection(&mut client_stream)
        }
        addr_type::IPV6 => {
            let mut buffer_v6: [u8; 13] = [0; 13];
            if client_stream.read(&mut buffer_v6).is_err() {
                println!("Error reading IPV6 address.");
                return;
            };
            ipv6_connection(&mut buffer_v6)
        }
        _ => {
            println!("Failed connection.");
            return;
        }
    };

    let mut destination_stream = match destination_stream_result {
        Ok(stream) => stream,
        Err(e) => {
            match e {
                ReplyCode::NetworkUnreachable => {
                    println!("Interruption in stream: Network Unreachable");
                    return;
                }
                _ => {
                    println!("General Failure");
                    return;
                }
            }
        }
    };


    // connection succeeded
    // +----+-----+-------+------+----------+----------+
    // |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
    // +----+-----+-------+------+----------+----------+
    // | 1  |  1  | X'00' |  1   | Variable |    2     |
    // +----+-----+-------+------+----------+----------+

    if client_stream.write(&[SOCKS5VER, ReplyCode::Succeeded as u8, RESERVED, addr_type::IPV4, 0, 0, 0, 0, 0, 0]).is_err() {
        println!("Could not write to stream.");
        return;
    }

    let mut client_clone = match client_stream.try_clone() {
        Ok(clone) => clone,
        Err(e) => {
            println!("Could not clone client stream: {:?}", e);
            return;
        }
    };
    let mut destination_clone = match destination_stream.try_clone() {
        Ok(clone) => clone,
        Err(e) => {
            println!("Could not clone destination stream: {:?}", e);
            return;
        }
    };

    thread::spawn(move || {
        if std::io::copy(&mut client_clone, &mut destination_clone).is_err() {
            println!("Could not copy client stream to destination stream.");
            return;
        }
    });

    if std::io::copy(&mut destination_stream, &mut client_stream).is_err() {
        println!("Could not copy destination stream to client stream.");
        return;
    }
}

fn ipv4_connection(buffer: &mut [u8; 7]) -> Result<TcpStream, ReplyCode> {
    println!("Opening IPV4 Connection");

    //IPV4 requires u8 by 4, an empty vector is initialized using default and the IP address is sliced in from derefencing buffer for 4 u8s. The IPv4Addr is assigned.
    // let mut address_array: [u8; 4] = Default::default();
    // address_array.copy_from_slice(buffer[4]);

    let socket_v4 = SocketAddrV4::new(Ipv4Addr::new(buffer[0], buffer[1], buffer[2], buffer[3]), u16::from_be_bytes([buffer[4], buffer[5]]));
    TcpStream::connect(socket_v4).map_err(| _ | ReplyCode::NetworkUnreachable)
}
//result address type, socksreplycode
fn ipv6_connection(buffer: &mut [u8; 13]) -> Result<TcpStream, ReplyCode> {
    println!("Opening IPV6 Connection");

    let socket_v6 = SocketAddrV6::new(Ipv6Addr::new(buffer[0] as u16, buffer[1] as u16, buffer[2] as u16, buffer[3] as u16, buffer[4] as u16, buffer[5] as u16,
                                                    buffer[6] as u16, buffer[7] as u16), u16::from_be_bytes([buffer[8], buffer[9]]), 0, 0);
    TcpStream::connect(socket_v6).map_err(|_| ReplyCode::NetworkUnreachable)
}

fn domain_name_connection(stream: &mut TcpStream) -> Result<TcpStream, ReplyCode> {
    println!("Opening Domain Name Connection");

    let mut domain_buffer: [u8; 1] = [0; 1];
    let size = match stream.read(&mut domain_buffer) {
        Ok(len) => len,
        Err(_) => {
            return Err(ReplyCode::NetworkUnreachable);
        }
    };

    let length   = domain_buffer[0] as usize;
    println!("length : {}", length);

    let mut domain_vec: Vec<u8> = vec![0; length + 2];
    if stream.read(&mut domain_vec).is_err() {
        return Err(ReplyCode::NetworkUnreachable)
    }

    //No http because of DNS
    let mut address = String::from_utf8_lossy(&domain_vec[0..length]).to_string();
    if address == "www.osu.edu" { //
        return Err(ReplyCode::NetworkUnreachable);
    }
    if address == "msu.edu" {
        //Issue with security because of certificates.
        address = "umich.edu".to_string();
    }
    let port: u16 = u16::from_be_bytes([domain_vec[length], domain_vec[(length + 1)]]);

    println!("{:?} : {:?}", address, port);
    TcpStream::connect((address, port)).map_err( |_| ReplyCode::NetworkUnreachable)
}
