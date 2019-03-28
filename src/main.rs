
use std::thread;

mod server;
mod connection;
mod packet;
use server::Server;

fn main() {
    let addr = "127.0.0.1:12345".parse().unwrap();
    let addr2 = "127.0.0.1:12346".parse().unwrap();
    
    let j1 = thread::spawn(move || {
        tokio::run(Server::new(addr2, 1500, 1).expect("bind fail"));
    });
    
    println!("1 server down 1 to go");

    let j2 = thread::spawn(move || {
        tokio::run(Server::new(addr, 1500, 1).expect("bind fail"));
    });

    j1.join().unwrap();
    j2.join().unwrap();

    println!("Up and running!");   
}
