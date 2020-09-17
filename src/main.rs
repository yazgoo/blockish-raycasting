mod protocol;
mod server;
mod client;
use crate::server::server;
use crate::client::client;
use std::env;
use std::thread;
#[macro_use] extern crate scan_fmt;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 5 {
        let server_address = args[2].clone();
        thread::spawn(move || {
            server(server_address, true);
        });
        client(args[2].clone(), args[3].clone(), args[4].clone());
    }
    else if args.len() == 4 {
        client(args[1].clone(), args[2].clone(), args[3].clone());
    }
    else if args.len() == 2 {
        server(args[1].clone(), false);
    }
    else {
        println!("usage");
        println!("        server: <server address>");
        println!("           e.g:  0.0.0.0:12345");
        println!("        client: <server address> <client address> <nickname>");
        println!("           e.g:  0.0.0.0:12345    0.0.0.0:12346    yazgoo");
        println!(" client+server: serve <server address> <client address> <nickname>");
        println!("           e.g: serve  0.0.0.0:12345    0.0.0.0:12346    yazgoo");
    }
}
