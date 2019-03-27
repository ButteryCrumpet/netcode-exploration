
mod server;
mod connection;
mod packet;
use server::Server;

fn main() {
    let addr = "127.0.0.1:12345".parse().unwrap();
    
    tokio::run(Server::new(addr, 1500, 2).expect("bind fail"));
}
