
use std::thread;

mod server;
mod connection;
mod packet;
use server::Server;

fn main() {
    let addr = "0.0.0.0:12345".parse().unwrap();
    let addr2 = "127.0.0.1:12346".parse().unwrap();
    
    let j1 = thread::spawn(move || {
        tokio::run(Server::new(addr2, 1500, 1).expect("bind fail"));
    });

    let j2 = thread::spawn(move || {
        tokio::run(Server::new(addr, 1500, 1).expect("bind fail"));
    });

    j1.join().unwrap();
    j2.join().unwrap();
}
