use std::thread;
use std::time;

mod client;
mod connection;
mod message;
mod packet;
mod server;

use client::Client;
use server::Server;

fn main() {
    let client_addr = "0.0.0.0:12345".parse().unwrap();
    let server_addr = "127.0.0.1:12346".parse().unwrap();

    let j1 = thread::spawn(move || {
        tokio::run(Server::new(server_addr, 1504, 1).expect("bind fail"));
    });

    let j2 = thread::spawn(move || {
        let mut client = Client::new(client_addr);
        client.connect(server_addr);

        client.send(b"hi").unwrap();
        let start = time::Instant::now();
        let mut last_sent = time::Instant::now();
        let mut count: u128 = 0;
        loop {
            let ltime = time::Instant::now();

            if let Ok(data) = client.recv() {
                print!("\r{}", std::str::from_utf8(&data).unwrap());
            }

            if ltime - last_sent > time::Duration::from_millis(33) {
                client.queue_message(Vec::from(format!("This is message: {}", count)));
                count += 1;
                last_sent = ltime;
            }

            if let Ok(Some(_amt)) = client.send_next() {}

            if time::Instant::now() - start > time::Duration::from_secs(20) {
                break;
            }
        }
    });

    j1.join().unwrap();
    j2.join().unwrap();
}
