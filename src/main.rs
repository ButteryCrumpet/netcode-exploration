use std::time;
use std::thread;

mod server;
mod connection;
mod packet;
mod client;

use server::Server;
use client::Client;

fn main() {
    let client_addr = "0.0.0.0:12345".parse().unwrap();
    let server_addr = "127.0.0.1:12346".parse().unwrap();
    
    let j1 = thread::spawn(move || {
        tokio::run(Server::new(server_addr, 1504, 1).expect("bind fail"));
    });

    let j2 = thread::spawn(move || {
        let mut client = Client::new(client_addr, server_addr);
        client.connect();

        client.send(b"hi").unwrap();
        let start = time::Instant::now();
        loop {

            if let Ok(data) = client.recv() {
                thread::sleep(time::Duration::from_millis(33));
                client.send(&data).unwrap();
            }

            if time::Instant::now() - start > time::Duration::from_secs(20) {
                break;
            }
        }

    });

    j1.join().unwrap();
    j2.join().unwrap();
}
