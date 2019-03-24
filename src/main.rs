
use std::io;
use std::net::SocketAddr;

use tokio::prelude::*;
use tokio::net::UdpSocket;

struct Server {
    socket: UdpSocket,
    buf: [u8; 1500]
}

impl Server {

    fn send(&mut self, amt: usize, addr: &SocketAddr) -> Poll<usize, io::Error> {
        let msg = [b"Ack - ", &self.buf[..amt]].concat();
        self.socket.poll_send_to(&msg, addr)
    }

    fn read(&mut self) -> Poll<(usize, SocketAddr), io::Error> {
        self.socket.poll_recv_from(&mut self.buf)
    }

}

impl Future for Server {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            let (msg_len, addr) = match self.read() {
                Ok(Async::Ready(t)) => t,
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(e) =>  {
                    println!("Ahh shit.. {}", e);
                    return Ok(Async::NotReady)
                },
            };

            self.send(msg_len, &addr).unwrap();

            println!("{}", String::from_utf8_lossy(&self.buf[0..msg_len]));
        }
    }
       
}

fn main() {
    let addr = "127.0.0.1:12345".parse().unwrap();
    let socket = UdpSocket::bind(&addr).unwrap();

    tokio::run(Server {
        socket: socket,
        buf: [0; 1500]
    });
}
