extern crate rand;

use std::net::SocketAddr;
use std::collections::HashMap;
use laminar::{Socket, SocketEvent, Packet};
use crate::protocol::*;
use std::time::{Duration, Instant};
use crossbeam_channel::Sender;
use std::thread;
use rand::prelude::*;

fn check_gold_coins(coins_found: u32, world_map: &Vec<Vec<u8>>, packet_sender: &Sender<Packet>, gold_coins: &mut Vec<(f32, f32)>, positions : &HashMap<SocketAddr, Position>, nicknames: &HashMap<SocketAddr, String>, points: &mut HashMap<SocketAddr, u8>) -> u32 {
    let mut new_coins_found = coins_found;
    let mut who = None;
    for (key, value) in positions {
        for i in 0..gold_coins.len() {
            if (value.x as i32, value.y as i32) == (gold_coins[i].0 as i32, gold_coins[i].1 as i32) {
                let (new_x, new_y) = random_position(world_map);
                gold_coins[i].0 = new_x;
                gold_coins[i].1 = new_y;
                who = Some(key);
                let who_points = points.get(key).unwrap();
                points.insert(*key, who_points + 1);
            }
        }
    }
    if let Some(winner_endpoint) = who {
        new_coins_found += 1;
        if new_coins_found > 2 {
            let final_winner_endpoint = points.iter().max_by_key(|entry | entry.1).unwrap();
            let winner = nicknames.get(final_winner_endpoint.0).unwrap();
            for (key, _) in positions {
                let textures_message = ServerMessage::MessageText(format!("winner: {}", winner), Duration::from_secs(10));
                let message_ser = bincode::serialize(&textures_message).unwrap();
                packet_sender.send(Packet::reliable_unordered(key.clone(), message_ser)).unwrap();
                points.insert(*key, 0);
            }
            new_coins_found = 0;
        }
        else {
            for (key, _) in positions {
                let textures_message = ServerMessage::MessageText(format!("coin {}: {}", new_coins_found, nicknames.get(winner_endpoint).unwrap()), Duration::from_secs(10));
                let message_ser = bincode::serialize(&textures_message).unwrap();
                packet_sender.send(Packet::reliable_unordered(key.clone(), message_ser)).unwrap();
            }
        }
        for (key, _) in positions {
            let message = ServerMessage::MessageGoldCoins(gold_coins.clone());
            let message_ser = bincode::serialize(&message).unwrap();
            packet_sender.send(Packet::reliable_unordered(key.clone(), message_ser)).unwrap();
        }
    }
    new_coins_found
}

fn random_position(world_map: &Vec<Vec<u8>>) -> (f32, f32) {
    let mut rng = rand::thread_rng();
    loop {
        let y = rng.gen_range(0, 23);
        let x = rng.gen_range(0, 23);
        if world_map[x][y] == 0 {
            let coin_position = (x as f32 + 0.5, y as f32 + 0.5);
            println!("coin position{:?}", coin_position);
            return coin_position;
        }
    }
}


pub fn server(address: String) {
    let mut coins_found = 0;
    let world_map : Vec<Vec<u8>> =
        vec![
        vec![8,8,8,8,8,8,8,8,8,8,8,4,4,6,4,4,6,4,6,4,4,4,6,4],
        vec![8,0,0,0,0,0,0,0,0,0,8,4,0,0,0,0,0,0,0,0,0,0,0,4],
        vec![8,0,3,3,0,0,0,0,0,8,8,4,0,0,0,0,0,0,0,0,0,0,0,6],
        vec![8,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,6],
        vec![8,0,3,3,0,0,0,0,0,8,8,4,0,0,0,0,0,0,0,0,0,0,0,4],
        vec![8,0,0,0,0,0,0,0,0,0,8,4,0,0,0,0,0,6,6,6,0,6,4,6],
        vec![8,8,8,8,0,8,8,8,8,8,8,4,4,4,4,4,4,6,0,0,0,0,0,6],
        vec![7,7,7,7,0,7,7,7,7,0,8,0,8,0,8,0,8,4,0,4,0,6,0,6],
        vec![7,7,0,0,0,0,0,0,7,8,0,8,0,8,0,8,8,6,0,0,0,0,0,6],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,8,6,0,0,0,0,0,4],
        vec![7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,8,6,0,6,0,6,0,6],
        vec![7,7,0,0,0,0,0,0,7,8,0,8,0,8,0,8,8,6,4,6,0,6,6,6],
        vec![7,7,7,7,0,7,7,7,7,8,8,4,0,6,8,4,8,3,3,3,0,3,3,3],
        vec![2,2,2,2,0,2,2,2,2,4,6,4,0,0,6,0,6,3,0,0,0,0,0,3],
        vec![2,2,0,0,0,0,0,2,2,4,0,0,0,0,0,0,4,3,0,0,0,0,0,3],
        vec![2,0,0,0,0,0,0,0,2,4,0,0,0,0,0,0,4,3,0,0,0,0,0,3],
        vec![1,0,0,0,0,0,0,0,1,4,4,4,4,4,6,0,6,3,3,0,0,0,3,3],
        vec![2,0,0,0,0,0,0,0,2,2,2,1,2,2,2,6,6,0,0,5,0,5,5,5],
        vec![2,2,0,0,0,0,0,2,2,2,0,0,0,2,2,5,5,0,5,0,0,0,5,5],
        vec![2,0,0,0,0,0,0,0,2,0,0,0,0,0,2,5,0,5,0,5,0,5,0,5],
        vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,5],
        vec![2,0,0,0,0,0,0,0,2,0,0,0,0,0,2,5,0,5,0,5,0,5,0,5],
        vec![2,2,0,0,0,0,0,2,2,2,0,0,0,2,2,5,5,5,5,0,0,0,5,5],
        vec![2,2,2,2,1,2,2,2,2,2,2,1,2,2,2,5,5,5,5,5,5,5,5,5]
            ];
        let sprites = 
            vec![
            vec![20.5, 11.5, 10.0], //green light in front of playerstart
            //green lights in every room
            vec![18.5,4.5, 10.0],
            vec![10.0,4.5, 10.0],
            vec![10.0,12.5,10.0],
            vec![3.5, 6.5, 10.0],
            vec![3.5, 20.5,10.0],
            vec![3.5, 14.5,10.0],
            vec![14.5,20.5,10.0],

            //row of pillars in front of wall: fisheye test
            vec![18.5, 10.5, 9.0],
            vec![18.5, 11.5, 9.0],
            vec![18.5, 12.5, 9.0],

            //some barrels around the map
            vec![21.5, 1.5, 8.0],
            vec![15.5, 1.5, 8.0],
            vec![16.0, 1.8, 8.0],
            vec![16.2, 1.2, 8.0],
            vec![3.5,  2.5, 8.0],
            vec![9.5, 15.5, 8.0],
            vec![10.0, 15.1,8.0],
            vec![10.5, 15.8,8.0],
            ];

    let mut gold_coins = vec![
        random_position(&world_map)
    ];
    let textures_url = "https://srv-file10.gofile.io/download/GrF7ZN/wolfenstein_textures.zip";
    // Creates the socket
    let mut socket = Socket::bind(address).unwrap();
    let packet_sender = socket.get_packet_sender();
    let event_receiver = socket.get_event_receiver();
    // Starts the socket, which will start a poll mechanism to receive and send messages.
    let _thread = thread::spawn(move || socket.start_polling());

    let mut positions = HashMap::new();
    let mut last_seen = HashMap::new();
    let mut nicknames = HashMap::new();
    let mut points = HashMap::new();

    loop {
        // Waits until a socket event occurs
        let result = event_receiver.recv();

        match result {
            Ok(socket_event) => {
                match  socket_event {
                    SocketEvent::Packet(packet) => {
                        let endpoint: SocketAddr = packet.addr();
                        let received_data: &[u8] = packet.payload();
                        let message = bincode::deserialize::<ClientMessage>(received_data).unwrap();
                        match message {
                            ClientMessage::MessagePosition(pos) => {
                                last_seen.insert(endpoint, Instant::now());
                                positions.insert(endpoint, pos);

                                let now = Instant::now();
                                println!("positions {:?}", positions);
                                let mut to_remove = vec![];
                                for (key, _) in &positions {
                                    if now - last_seen[key] > Duration::from_secs(20) {
                                        to_remove.push(key.clone());
                                    }
                                }
                                for key in to_remove {
                                    positions.remove(&key);
                                    last_seen.remove(&key);
                                }
                                let mut positions_clone = HashMap::new();
                                for (key, value) in &positions {
                                    if key != &endpoint {
                                        positions_clone.insert(key.clone(), value.clone());
                                    }
                                }
                                coins_found = check_gold_coins(coins_found, &world_map, &packet_sender, &mut gold_coins, &positions, &nicknames, &mut points);
                                let positions_message = ServerMessage::MessagePositions(positions_clone);
                                let pos_ser = bincode::serialize(&positions_message).unwrap();
                                packet_sender.send(Packet::reliable_unordered(endpoint, pos_ser)).unwrap();
                            },
                            ClientMessage::MessageHello(nickname) => {
                                nicknames.insert(endpoint, nickname);
                                points.insert(endpoint, 0);
                                // let (x, y) = random_position(&world_map);
                                let (x, y) = (20.5, 12.0);
                                let map_message = ServerMessage::MessageTeleport(Position { x: x, y: y, dir_x: -1.0, dir_y: 0.0, speed: 0.0 });
                                let message_ser = bincode::serialize(&map_message).unwrap();
                                packet_sender.send(Packet::reliable_unordered(endpoint, message_ser)).unwrap();
                                let map_message = ServerMessage::MessageWorldMap(world_map.clone());
                                let message_ser = bincode::serialize(&map_message).unwrap();
                                packet_sender.send(Packet::reliable_unordered(endpoint, message_ser)).unwrap();
                                let sprites_message = ServerMessage::MessageSprites(sprites.clone());
                                let message_ser = bincode::serialize(&sprites_message).unwrap();
                                packet_sender.send(Packet::reliable_unordered(endpoint, message_ser)).unwrap();
                                let textures_message = ServerMessage::MessageTexturesZip(String::from(textures_url));
                                let message_ser = bincode::serialize(&textures_message).unwrap();
                                packet_sender.send(Packet::reliable_unordered(endpoint, message_ser)).unwrap();
                                let textures_message = ServerMessage::MessageGoldCoins(gold_coins.clone());
                                let message_ser = bincode::serialize(&textures_message).unwrap();
                                packet_sender.send(Packet::reliable_unordered(endpoint, message_ser)).unwrap();
                                let textures_message = ServerMessage::MessageText(String::from("Hello !"), Duration::from_secs(10));
                                let message_ser = bincode::serialize(&textures_message).unwrap();
                                packet_sender.send(Packet::reliable_unordered(endpoint, message_ser)).unwrap();
                            }
                        }
                    },
                   SocketEvent::Connect(_) => { /* a client connected */ },
                    SocketEvent::Timeout(_) => { /* a client timed out */},
                }
            }
            Err(e) => {
                println!("Something went wrong when receiving, error: {:?}", e);
            }
        }
    }
}

