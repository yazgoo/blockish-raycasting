extern crate rand;
extern crate image;
extern crate crossterm_input;
extern crate laminar;
extern crate bincode;
extern crate reqwest;
#[macro_use] extern crate scan_fmt;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use laminar::{Socket, SocketEvent, Packet};

use std::thread;
use std::time::{Duration, Instant};
use crossterm::terminal;
use image::imageops::FilterType;

use std::net::SocketAddr;
use std::io::Read;
use std::env;
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Position {
    x: f32,
    y: f32,
    dir_x: f32,
    dir_y: f32,
    speed: f32
}

#[derive(Serialize, Deserialize, Debug)]
enum ServerMessage {
    MessagePositions(HashMap<SocketAddr, Position>),
    MessageWorldMap(Vec<Vec<u8>>),
    MessageSprites(Vec<Vec<f32>>),
    MessageTexturesZip(String),
    MessageGoldCoins(Vec<(f32, f32)>),
}

#[derive(Serialize, Deserialize, Debug)]
enum ClientMessage {
    MessagePosition(Position),
    MessageHello,
}

fn render_floor_ceiling(textures: &Vec<Vec<u8>>, tex_width: u32, tex_height: u32, color_buff: &mut Vec<u32>, w: usize, h: usize, pos_x: f32, pos_y: f32, dir_x:f32, dir_y: f32, plane_x: f32, plane_y: f32) {
    for y in 0..h
    {
      // ray_dir for leftmost ray (x = 0) and rightmost ray (x = w)
      let ray_dir_x0 = dir_x - plane_x;
      let ray_dir_y0 = dir_y - plane_y;
      let ray_dir_x1 = dir_x + plane_x;
      let ray_dir_y1 = dir_y + plane_y;

      let screen_height = h as f32;
      let screen_width = w as f32;
      // Current y position compared to the center of the screen (the horizon)
      let p = y as f32 - screen_height / 2.0;

      // Vertical position of the camera.
      let pos_z = 0.5 * screen_height;

      // Horizontal distance from the camera to the floor for the current row.
      // 0.5 is the z position exactly in the middle between floor and ceiling.
      let row_distance = pos_z / p;

      // calculate the real world step vector we have to add for each x (parallel to camera plane)
      // adding step by step avoids multiplications with a weight in the inner loop
      let floor_step_x = row_distance * (ray_dir_x1 - ray_dir_x0) / screen_width;
      let floor_step_y = row_distance * (ray_dir_y1 - ray_dir_y0) / screen_width;

      // real world coordinates of the leftmost column. This will be updated as we step to the right.
      let mut floor_x = pos_x + row_distance * ray_dir_x0;
      let mut floor_y = pos_y + row_distance * ray_dir_y0;

      for x in 0..w
      {
        // the cell coord is simply got from the integer parts of floor_x and floor_y
        let cell_x = floor_x as i32;
        let cell_y = floor_y as i32;

        // get the texture coordinate from the fractional part
        let tx = ((tex_width as f32 * (floor_x - cell_x as f32)) as u32) & (tex_width as u32 - 1);
        let ty = ((tex_height as f32 * (floor_y - cell_y as f32)) as u32) & (tex_height as u32 - 1);

        floor_x += floor_step_x;
        floor_y += floor_step_y;

        // choose texture and draw the pixel
        let floor_texture = 3;
        let ceiling_texture = 6;

        let tex_id = floor_texture as usize;
        let tex_i = tx + ty * tex_width;
        let tex_i = (tex_i * 3) as usize;
        let color = textures[tex_id][tex_i] as u32 |
                        ((textures[tex_id][tex_i + 1] as u32) << 8) |
                        ((textures[tex_id][tex_i + 2] as u32) << 16);
        let color = (color >> 1) & 8355711; // make a bit darker
        color_buff[y as usize * w + x as usize] = color as u32;

        //ceiling (symmetrical, at screen_height - y - 1 instead of y)
        let tex_id = ceiling_texture as usize;
        let color = textures[tex_id][tex_i] as u32 |
                        ((textures[tex_id][tex_i + 1] as u32) << 8) |
                        ((textures[tex_id][tex_i + 2] as u32) << 16);
        let color = (color >> 1) & 8355711; // make a bit darker
        color_buff[(h - y - 1) as usize * w + x as usize] = color as u32;
      }
    }
}

fn render_sprites(sprites: &Vec<Vec<f32>>, textures: &Vec<Vec<u8>>, texture_width: u32, texture_height: u32,color_buff: &mut Vec<u32>, depth_buff: &Vec<f32>, w: usize, h: usize, pos_x: f32, pos_y: f32, dir_x: f32, dir_y: f32, plane_x: f32, plane_y: f32, rgba: bool) {
    let bytesPerPixel = if rgba { 4 } else { 3 };
    let mut sorted_sprites = sprites.into_iter()
        .map( |x| (x, ((pos_x - x[0]) * (pos_x - x[0]) + (pos_y - x[1]) * (pos_y - x[1]))))
        .collect::<Vec<(&Vec<f32>, f32)>>();
    sorted_sprites.sort_by( |a, b| b.1.partial_cmp(&a.1).unwrap());
    let sorted_sprites : Vec<&Vec<f32>> = sorted_sprites
        .into_iter()
        .map(|x| x.0)
        .collect();
    //sqrt not taken, unneeded
    for sprite in sorted_sprites {
        let sprite_x = sprite[0] - pos_x;
        let sprite_y = sprite[1] - pos_y;

        //transform sprite with the inverse camera matrix
        // [ plane_x   dir_x ] -1                                       [ dir_y      -dir_x ]
        // [               ]       =  1/(plane_x*dir_y-dir_x*plane_y) *   [                 ]
        // [ plane_y   dir_y ]                                          [ -plane_y  plane_x ]

        let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y); //required for correct matrix multiplication

        let transform_x = inv_det * (dir_y * sprite_x - dir_x * sprite_y);
        let transform_y = inv_det * (-plane_y * sprite_x + plane_x * sprite_y); //this is actually the depth inside the screen, that what Z is in 3_d

        let sprite_screen_x = ((w as f32 / 2.0) * (1.0 + transform_x / transform_y)) as usize;

        //calculate height of the sprite on screen
        let sprite_height = ((h as f32 / (transform_y)) as i32).abs(); //using 'transform_y' instead of the real distance prevents fisheye
        //calculate lowest and highest pixel to fill in current stripe
        let mut draw_start_y = -sprite_height / 2 + h as i32 / 2;
        if draw_start_y < 0 {
            draw_start_y = 0;
        }
        let mut draw_end_y = sprite_height / 2 + h as i32 / 2;
        if draw_end_y >= h as i32  {
            draw_end_y = h as i32 - 1;
        }

        //calculate width of the sprite
        let sprite_width = ((h as f32/ (transform_y)) as i32).abs();
        let mut draw_start_x = -sprite_width / 2 + sprite_screen_x as i32;
        if draw_start_x < 0 {
            draw_start_x = 0;
        }
        let mut draw_end_x = sprite_width / 2 + sprite_screen_x as i32;
        if draw_end_x >= w as i32 {
            draw_end_x = w as i32 - 1;
        }

        //loop through every vertical stripe of the sprite on screen
        for stripe in draw_start_x..draw_end_x
        {
            let tex_x = (256 * (stripe - (-sprite_width / 2 + sprite_screen_x as i32)) * texture_width as i32 / sprite_width) as i32 / 256;
            
            //the conditions in the if are:
            //1) it's in front of camera plane so you don't see things behind you
            //2) it's on the screen (left)
            //3) it's on the screen (right)
            //4) ZBuffer, with perpendicular distance
            if transform_y > 0.0 && stripe > 0 && stripe < w as i32 && transform_y < depth_buff[stripe as usize] {
                for y in draw_start_y..draw_end_y
                {
                    let d = (y) * 256 - h as i32 * 128 + sprite_height * 128; //256 and 128 factors to avoid floats
                    let tex_y = ((d * texture_height as i32) / sprite_height) / 256;
                    let tex_i = tex_x as usize + tex_y as usize * texture_width as usize;
                    let tex_i = (tex_i * bytesPerPixel) as usize;
                    let tex_id = sprite[2] as usize;
                    let color = textures[tex_id][tex_i] as u32 |
                        ((textures[tex_id][tex_i + 1] as u32) << 8) |
                        ((textures[tex_id][tex_i + 2] as u32) << 16);
                    if (!rgba && (color & 0x00_fFFFFF) != 0) || (rgba && textures[tex_id][tex_i + 2] != 0xff) {
                        color_buff[y as usize * w + stripe as usize] = color as u32
                    }
                }
            }
        }
    }
}

fn render_walls(textures: &Vec<Vec<u8>>, texture_width: u32, texture_height: u32, world_map: &Vec<Vec<u8>>, color_buff: &mut Vec<u32>, depth_buff: &mut Vec<f32>, w: usize, h: usize, pos_x: f32, pos_y: f32, dir_x: f32, dir_y: f32, plane_x: f32, plane_y: f32) {


  for x in 0..w
  {
      //calculate ray position and direction
      let camera_x = 2.0 * x as f32 / w as f32 - 1.0; //x-coordinate in camera space
      let ray_dir_x = dir_x + plane_x * camera_x;
      let ray_dir_y = dir_y + plane_y * camera_x;
      //which box of the map we're in
      let mut map_x = pos_x as i32;
      let mut map_y = pos_y as i32;

      //length of ray from current position to next x or y-side
      let mut side_dist_x;
      let mut side_dist_y;

      //length of ray from one x or y-side to next x or y-side
      let delta_dist_x = (1.0 / ray_dir_x).abs();
      let delta_dist_y = (1.0 / ray_dir_y).abs();
      let perp_wall_dist;

      //what direction to step in x or y-direction (either +1 or -1)
      let step_x : i32;
      let step_y : i32;

      let mut hit = 0; //was there a wall hit?
      let mut side = 0; //was a NS or a EW wall hit?
      //calculate step and initial side_dist
      if ray_dir_x < 0.0
      {
          step_x = -1;
          side_dist_x = (pos_x - map_x as f32) * delta_dist_x;
      }
      else
      {
          step_x = 1;
          side_dist_x = (map_x as f32 + 1.0 - pos_x) * delta_dist_x;
      }
      if ray_dir_y < 0.0
      {
          step_y = -1;
          side_dist_y = (pos_y as f32 - map_y as f32) * delta_dist_y;
      }
      else
      {
          step_y = 1;
          side_dist_y = (map_y  as f32+ 1.0 - pos_y as f32) * delta_dist_y;
      }
      //perform DDA
      while hit == 0
      {
          //jump to next map square, OR in x-direction, OR in y-direction
          if side_dist_x < side_dist_y
          {
              side_dist_x += delta_dist_x;
              map_x += step_x;
              side = 0;
          }
          else
          {
              side_dist_y += delta_dist_y;
              map_y += step_y;
              side = 1;
          }
          //Check if ray has hit a wall
          if world_map[map_x as usize][map_y as usize] > 0 {
              hit = 1;
          }
      }
      //Calculate distance projected on camera direction (Euclidean distance will give fisheye effect!)
      if side == 0 { 
          perp_wall_dist = (map_x as f32 - pos_x + (1.0 - step_x as f32) / 2.0) / ray_dir_x;
      }
      else { 
          perp_wall_dist = (map_y as f32 - pos_y + (1.0 - step_y as f32) / 2.0) / ray_dir_y;
      }

      //Calculate height of line to draw on screen
      let line_height = (h as f32/ perp_wall_dist) as usize;

      //calculate lowest and highest pixel to fill in current stripe
      let mut draw_start = - (line_height as i32) / 2 + h as i32 / 2;
      let draw_start_neg = draw_start;
      if draw_start < 0 { 
          draw_start = 0;
      }
      let mut draw_end = (line_height as i32) / 2 + (h as i32) / 2;
      if draw_end >= h as i32 {
          draw_end = h as i32 - 1;
      }

      //choose wall color
      
      let tex_id = world_map[map_x as usize][map_y as usize] as usize;

      let mut wall_x; //where exactly the wall was hit
      if side == 0 { 
          wall_x = pos_y + perp_wall_dist * ray_dir_y;
      }
      else          { 
          wall_x = pos_x + perp_wall_dist * ray_dir_x;
      }
      wall_x -= wall_x.floor();

      //draw the pixels of the stripe as a vertical line
      let mut tex_x = (wall_x * texture_width as f32) as i32;
      if side == 0 && ray_dir_x > 0.0 {
          tex_x = texture_width as i32 - tex_x - 1;
      }
      if side == 1 && ray_dir_y < 0.0 {
          tex_x = texture_width as i32 - tex_x - 1;
      }
      for y in draw_start..draw_end {
          let tex_y = (y - draw_start_neg) * texture_height as i32 / (line_height as i32);
          let tex_i = tex_x as usize + tex_y as usize * texture_width as usize;
          let tex_i = (tex_i * 3) as usize;
          let tex_id = tex_id - 1;
          color_buff[y as usize * w + x as usize] = textures[tex_id][tex_i] as u32 |
              ((textures[tex_id][tex_i + 1] as u32) << 8) |
              ((textures[tex_id][tex_i + 2] as u32) << 16);
      }
      depth_buff[x as usize] = perp_wall_dist;
  }
}

fn move_player(option_event: Option<crossterm_input::InputEvent>,world_map: &Vec<Vec<u8>>, pos_x: &mut f32, pos_y: &mut f32, dir_x: &mut f32, dir_y: &mut f32, plane_x: &mut f32, plane_y: &mut f32) -> f32 {
    let rot_speed : f32 = 0.1;
    let move_speed : f32 = 0.1;
    let mut res = 0.0;
        match option_event {
            Some(crossterm_input::InputEvent::Keyboard(crossterm_input::KeyEvent::Up)) => {
                if world_map[(*pos_x + *dir_x * move_speed) as usize][(*pos_y) as usize] == 0 { 
                    *pos_x += *dir_x * move_speed;
                }
                if world_map[(*pos_x) as usize][(*pos_y + *dir_y * move_speed) as usize] == 0 {
                    *pos_y += *dir_y * move_speed;
                }
                res = move_speed;
            },
            Some(crossterm_input::InputEvent::Keyboard(crossterm_input::KeyEvent::Down)) => {
                if world_map[(*pos_x - *dir_x * move_speed) as usize][(*pos_y) as usize] == 0 { 
                    *pos_x -= *dir_x * move_speed;
                }
                if world_map[(*pos_x) as usize][(*pos_y - *dir_y * move_speed) as usize] == 0 {
                    *pos_y -= *dir_y * move_speed;
                }
                res = -move_speed;
            },
            Some(crossterm_input::InputEvent::Keyboard(crossterm_input::KeyEvent::Esc)) => {
                std::process::exit(1);
            },
            Some(crossterm_input::InputEvent::Keyboard(crossterm_input::KeyEvent::Right)) => {
                let old_dir_x = *dir_x;
                *dir_x = *dir_x * (-rot_speed).cos() - *dir_y * (-rot_speed).sin();
                *dir_y = old_dir_x * (-rot_speed).sin() + *dir_y * (-rot_speed).cos();
                let old_plane_x = *plane_x;
                *plane_x = *plane_x * (-rot_speed).cos() - *plane_y * (-rot_speed).sin();
                *plane_y = old_plane_x * (-rot_speed).sin() + *plane_y * (-rot_speed).cos();
            },
            Some(crossterm_input::InputEvent::Keyboard(crossterm_input::KeyEvent::Left)) => {
                let old_dir_x = *dir_x;
                *dir_x = *dir_x * (rot_speed).cos()  - *dir_y * (rot_speed).sin();
                *dir_y = old_dir_x * (rot_speed).sin() + *dir_y * (rot_speed).cos();
                let old_plane_x = *plane_x;
                *plane_x = *plane_x * (rot_speed).cos()  - *plane_y * (rot_speed).sin();
                *plane_y = old_plane_x * (rot_speed).sin() + *plane_y * (rot_speed).cos();
            },
            _ => {}
        }
        res
}

fn server(address: String) {
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
        vec![2,0,0,0,0,0,0,0,2,2,2,1,2,2,2,6,6,0,0,5,0,5,0,5],
        vec![2,2,0,0,0,0,0,2,2,2,0,0,0,2,2,0,5,0,5,0,0,0,5,5],
        vec![2,0,0,0,0,0,0,0,2,0,0,0,0,0,2,5,0,5,0,5,0,5,0,5],
        vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,5],
        vec![2,0,0,0,0,0,0,0,2,0,0,0,0,0,2,5,0,5,0,5,0,5,0,5],
        vec![2,2,0,0,0,0,0,2,2,2,0,0,0,2,2,0,5,0,5,0,0,0,5,5],
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
        (20.5, 12.5)
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
                                let mut gold_coins_changed = false;
                                for (key, value) in &positions {
                                    if key != &endpoint {
                                        positions_clone.insert(key.clone(), value.clone());
                                    }
                                    for i in 0..gold_coins.len() {
                                        if (value.x as i32, value.y as i32) == (gold_coins[i].0 as i32, gold_coins[i].1 as i32) {
                                            gold_coins[i].1 -= 1.0;
                                            gold_coins_changed = true;
                                        }
                                    }
                                }
                                if gold_coins_changed {
                                    for (key, _) in &positions {
                                        let textures_message = ServerMessage::MessageGoldCoins(gold_coins.clone());
                                        let message_ser = bincode::serialize(&textures_message).unwrap();
                                        packet_sender.send(Packet::reliable_unordered(key.clone(), message_ser)).unwrap();
                                    }
                                }
                                let positions_message = ServerMessage::MessagePositions(positions_clone);
                                let pos_ser = bincode::serialize(&positions_message).unwrap();
                                packet_sender.send(Packet::reliable_unordered(endpoint, pos_ser)).unwrap();
                            },
                            ClientMessage::MessageHello => {
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

fn load_textures(url: String) -> Vec<Vec<u8>> {
    let texture_size = 64;
    let resp = reqwest::blocking::get(&url).unwrap().bytes().unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(resp)).unwrap();
    let mut textures = HashMap::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if !(&*file.name()).ends_with('/') {
            let mut bytes = vec![0; file.size() as usize];
            file.read(&mut bytes).unwrap();
            let someindex = scan_fmt_some!(file.name(), "pics/{d}.png", usize);
            let index = someindex.unwrap();
            textures.insert(index,
                image::load(std::io::Cursor::new(bytes),  image::ImageFormat::PNG).unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw());
        }
    }
    let mut v: Vec<_> = textures.into_iter().collect();
    v.sort_by(|x,y| x.0.cmp(&y.0));
    v.into_iter().map(|x| x.1).collect()
}

fn client(server_address: String, client_address: String) {
        let window_width = 640;
        let window_height = 320;
        let time_per_frame = 1000/ 60;
        let mut color_buff : Vec<u32> = vec![0; window_width * window_height];
        let mut depth_buff : Vec<f32> = vec![0.0; window_width];


        let mut term_width = 0 as u32;
        let mut term_height = 0 as u32;

        match terminal::size() {
            Ok(res) => {
                term_width = res.0 as u32 * 8;
                term_height = res.1 as u32 * 8 * 2;
            }
            Err(_) => {}
        }

        let mut engine = blockish::ThreadedEngine::new(term_width, term_height, false);
        for i in 0..window_width {
            color_buff[(window_height - 1) * window_width + i] = 36;
        }
        let _screen = crossterm_input::RawScreen::into_raw_mode();
        let input = crossterm_input::input();
        let mut reader = input.read_async();

        let mut dir_x = -1.0;
        let mut dir_y = 0.0;
        let mut pos_x = 22.0;
        let mut pos_y = 12.0;
        let mut plane_x = 0.0;
        let mut plane_y = 0.66; //the 2d raycaster version of camera plane

        let mut world_map=
            vec![
            vec![2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
            vec![2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2]
                ];

            let mut sprites = vec![];

                let texture_size = 64; // must be a power of two so that fractional part in floor ceiling computation work
                let texture_width = 64;
                let texture_height = 64;
                let default_texture = image::open("free-pics/default.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw();
                let mut textures = vec![default_texture; 11];
                let character_textures = vec![
                    image::open("free-pics/character.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw(),
                ];
                let coin_width = 32;
                let coin_height = 32;
                let goldcoin_textures = vec![
                    image::open("goldCoin/goldCoin1.png").unwrap().to_rgba().into_raw(),
                    image::open("goldCoin/goldCoin2.png").unwrap().to_rgba().into_raw(),
                    image::open("goldCoin/goldCoin3.png").unwrap().to_rgba().into_raw(),
                    image::open("goldCoin/goldCoin4.png").unwrap().to_rgba().into_raw(),
                    image::open("goldCoin/goldCoin5.png").unwrap().to_rgba().into_raw(),
                    image::open("goldCoin/goldCoin6.png").unwrap().to_rgba().into_raw(),
                    image::open("goldCoin/goldCoin7.png").unwrap().to_rgba().into_raw(),
                    image::open("goldCoin/goldCoin8.png").unwrap().to_rgba().into_raw(),
                    image::open("goldCoin/goldCoin9.png").unwrap().to_rgba().into_raw(),
                ];

                let mut gold_coins = vec![
                ];
                let mut socket = Socket::bind(client_address.clone()).unwrap();
                let packet_sender = socket.get_packet_sender();
                let event_receiver = socket.get_event_receiver();
                let _thread = thread::spawn(move || socket.start_polling());
                let server = server_address.parse().unwrap();

                let mut startup = true;
                let mut move_speed: f32 = 0.0;
                let mut character_positions = vec![];

                let mut previous = Instant::now();
                loop {
                    let mut stuff_to_read = true;
                    while stuff_to_read {
                        let result = event_receiver.try_recv();
                        match result {
                            Ok(socket_event) => {
                                match socket_event {
                                    SocketEvent::Packet(packet) => {
                                        let received_data: &[u8] = packet.payload();
                                        let message = bincode::deserialize::<ServerMessage>(received_data).unwrap();
                                        match message {
                                            ServerMessage::MessageSprites(s) => {
                                                sprites = s;
                                            },
                                            ServerMessage::MessageTexturesZip(s) => {
                                                textures = load_textures(s);
                                            },
                                            ServerMessage::MessageWorldMap(map) => {
                                                world_map = map;
                                            },
                                            ServerMessage::MessageGoldCoins(gcs) => {
                                                gold_coins = vec![];
                                                for gc in gcs {
                                                    gold_coins.push(vec![gc.0, gc.1, 0.0]);
                                                }
                                            },
                                            ServerMessage::MessagePositions(positions) => {
                                                character_positions = vec![];
                                                for position in positions {
                                                    character_positions.push(position.1);
                                                }
                                            },
                                        }
                                    },
                                    SocketEvent::Connect(_) => { /* a client connected */ },
                                    SocketEvent::Timeout(_) => { /* a client timed out */},
                                }
                            }
                            Err(_) => {
                                stuff_to_read = false;
                            }
                        }
                    }
                    let mut characters = vec![];
                    character_positions = character_positions.iter().map ( |position| {
                        Position {
                            x: position.x + position.speed * position.dir_x * 0.1,
                            y: position.y + position.speed * position.dir_y * 0.1,
                            dir_x: position.dir_x,
                            dir_y: position.dir_y,
                            speed: position.speed
                        }
                    }
                    ).collect::<Vec<Position>>();
                    for position in &character_positions {
                        characters.push(vec![position.x, position.y, 0.0]);
                    }
                    let now = Instant::now();
                    if (now - previous) > Duration::from_millis(500) {
                        if startup {
                            let message = ClientMessage::MessageHello;
                            let message_ser = bincode::serialize(&message).unwrap();
                            packet_sender.send(Packet::reliable_unordered(server, message_ser)).unwrap();
                            startup = false;
                        }
                        let pos = ClientMessage::MessagePosition(Position { x : pos_x, y : pos_y, dir_x: dir_x, dir_y: dir_y, speed: move_speed });
                        let pos_ser = bincode::serialize(&pos).unwrap();
                        packet_sender.send(Packet::reliable_unordered(server, pos_ser)).unwrap();
                        previous = now;
                        for i in 0..gold_coins.len() {
                            gold_coins[i][2] += 1.0;
                            if gold_coins[i][2] > 8.0 {
                                gold_coins[i][2] = 0.0;
                            }
                        }
                    }

                    let start_time = Instant::now();
                    render_floor_ceiling(&textures, texture_width, texture_height, &mut color_buff, window_width, window_height, pos_x, pos_y, dir_x, dir_y, plane_x, plane_y);
                    render_walls(&textures, texture_width, texture_height, &world_map, &mut color_buff, &mut depth_buff, window_width, window_height, pos_x, pos_y, dir_x, dir_y, plane_x, plane_y);
                    render_sprites(&sprites, &textures, texture_width, texture_height, &mut color_buff, &depth_buff, window_width, window_height, pos_x, pos_y, dir_x, dir_y, plane_x, plane_y, false);
                    render_sprites(&characters, &character_textures, texture_width, texture_height, &mut color_buff, &depth_buff, window_width, window_height, pos_x, pos_y, dir_x, dir_y, plane_x, plane_y, false);
                    render_sprites(&gold_coins, &goldcoin_textures, coin_width, coin_height, &mut color_buff, &depth_buff, window_width, window_height, pos_x, pos_y, dir_x, dir_y, plane_x, plane_y, true);
                    print!("\x1b[{};0f", 0);
                    engine.render(&|x, y| {
                        let start = (y * window_height as u32 / term_height * window_width as u32 + (x * window_width as u32 / term_width))
                            as usize;
                        let pixel = color_buff[start];
                        ((pixel & 0xff) as u8, (pixel >> 8 & 0xff) as u8, (pixel >> 16 & 0xff) as u8) 
                    });
                    let end_time = Instant::now();
                    let render_time = end_time - start_time;
                    if render_time < Duration::from_millis(time_per_frame) {
                        let waste_time = Duration::from_millis(time_per_frame) - render_time;
                        thread::sleep(waste_time);
                    }
                    let option_event = reader.next();
                    move_speed = move_player(option_event, &world_map, &mut pos_x, &mut pos_y, &mut dir_x, &mut dir_y, &mut plane_x, &mut plane_y)
                }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 3 {
        client(args[1].clone(), args[2].clone());
    }
    else if args.len() == 2 {
        server(args[1].clone());
    }
    else {
        println!("usage");
        println!("    server: <server address>");
        println!("       e.g:  0.0.0.0:12345");
        println!("    client: <server address> <client address>");
        println!("       e.g:  0.0.0.0:12345    0.0.0.0:12346");
    }
}
