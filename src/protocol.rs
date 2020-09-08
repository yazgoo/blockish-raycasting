use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Duration;
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub dir_x: f32,
    pub dir_y: f32,
    pub speed: f32
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    MessagePositions(HashMap<SocketAddr, Position>),
    MessageWorldMap(Vec<Vec<u8>>),
    MessageSprites(Vec<Vec<f32>>),
    MessageTexturesZip(String),
    MessageGoldCoins(Vec<(f32, f32)>),
    MessageText(String, Duration),
    MessageTeleport(Position),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    MessagePosition(Position),
    MessageHello(String),
}
