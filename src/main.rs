
use std::io;
use std::iter;
use std::collections::HashMap;
use std::net::SocketAddr;

use tokio::prelude::*;
use tokio::net::UdpSocket;

struct Server {
    socket: UdpSocket,
    buffer: Vec<u8>,
    connections: HashMap<SocketAddr, Connection>
}

impl Server {

    fn new (addr: SocketAddr, max_packet_size: usize) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(&addr)?;
        let buffer: Vec<u8> = iter::repeat(0).take(max_packet_size).collect();
        let connections = HashMap::new();

        Ok(Server {
            socket,
            buffer,
            connections
        })
    }

    fn read(&mut self) -> Poll<(Vec<u8>, SocketAddr), io::Error> {
        self.socket.poll_recv_from(&mut self.buffer)
            .map(|poll| {
                let (amt, addr) = match poll {
                    Async::Ready((amt, addr)) => (amt, addr),
                    Async::NotReady => return Async::NotReady
                };

                Async::Ready((self.buffer[..amt].to_vec(), addr))
            })
    }

    fn local_addr(&self) -> Result<SocketAddr, io::Error> {
        self.socket.local_addr()
    }

}

impl Future for Server {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            let (data, addr) = match self.read() {
                Ok(Async::Ready(t)) => t,
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(e) =>  {
                    println!("Ahh shit.. {}", e);
                    return Ok(Async::NotReady)
                },
            };

            if self.connections.contains_key(&addr) {
                for (_, conn) in self.connections.iter() {
                    self.socket.poll_send_to(&data, conn.remote_addr()).unwrap();
                }
            } else {
                if self.connections.len() < 4 {
                    self.connections.insert(addr, Connection { addr });
                }
            }

            println!("{}", String::from_utf8_lossy(&data));
        }
    }
       
}

struct Connection {
    addr: SocketAddr
}

impl Connection {
    fn remote_addr(&self) -> &SocketAddr {
        &self.addr
    }
}

fn main() {
    let addr = "127.0.0.1:12345".parse().unwrap();
    
    tokio::run(Server::new(addr, 1500).expect("bind fail"));
}
