extern crate rand;
extern crate image;
extern crate crossterm_input;
extern crate laminar;
extern crate bincode;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use laminar::{Socket, SocketEvent, Packet};

use std::thread;
use std::time::{Duration, Instant};
use crossterm::terminal;
use image::imageops::FilterType;

use std::net::{TcpStream, SocketAddr};
use std::io::{Read, Write};
use std::env;
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize, Debug)]
enum ServerMessage {
    MessagePositions(HashMap<SocketAddr, Position>),
    MessageWorldMap(Vec<Vec<u8>>),
    MessageSprites(Vec<Vec<f32>>),
}

#[derive(Serialize, Deserialize, Debug)]
enum ClientMessage {
    MessagePosition(Position),
    MessageHello,
}

fn render_floor_ceiling(textures: &Vec<Vec<u8>>, texWidth: u32, texHeight: u32, color_buff: &mut Vec<u32>, w: usize, h: usize, posX: f32, posY: f32, dirX:f32, dirY: f32, planeX: f32, planeY: f32) {
    for y in 0..h
    {
      // rayDir for leftmost ray (x = 0) and rightmost ray (x = w)
      let rayDirX0 = dirX - planeX;
      let rayDirY0 = dirY - planeY;
      let rayDirX1 = dirX + planeX;
      let rayDirY1 = dirY + planeY;

      let screenHeight = h as f32;
      let screenWidth = w as f32;
      // Current y position compared to the center of the screen (the horizon)
      let p = y as f32 - screenHeight / 2.0;

      // Vertical position of the camera.
      let posZ = 0.5 * screenHeight;

      // Horizontal distance from the camera to the floor for the current row.
      // 0.5 is the z position exactly in the middle between floor and ceiling.
      let rowDistance = posZ / p;

      // calculate the real world step vector we have to add for each x (parallel to camera plane)
      // adding step by step avoids multiplications with a weight in the inner loop
      let floorStepX = rowDistance * (rayDirX1 - rayDirX0) / screenWidth;
      let floorStepY = rowDistance * (rayDirY1 - rayDirY0) / screenWidth;

      // real world coordinates of the leftmost column. This will be updated as we step to the right.
      let mut floorX = posX + rowDistance * rayDirX0;
      let mut floorY = posY + rowDistance * rayDirY0;

      for x in 0..w
      {
        // the cell coord is simply got from the integer parts of floorX and floorY
        let cellX = floorX as i32;
        let cellY = floorY as i32;

        // get the texture coordinate from the fractional part
        let tx = ((texWidth as f32 * (floorX - cellX as f32)) as u32) & (texWidth as u32 - 1);
        let ty = ((texHeight as f32 * (floorY - cellY as f32)) as u32) & (texHeight as u32 - 1);

        floorX += floorStepX;
        floorY += floorStepY;

        // choose texture and draw the pixel
        let floorTexture = 3;
        let ceilingTexture = 6;

        let texId = floorTexture as usize;
        let texI = tx + ty * texWidth;
        let texI = (texI * 3) as usize;
        let color = (textures[texId][texI] as u32 |
                        ((textures[texId][texI + 1] as u32) << 8) |
                        ((textures[texId][texI + 2] as u32) << 16));
        let color = (color >> 1) & 8355711; // make a bit darker
        color_buff[y as usize * w + x as usize] = color as u32;

        //ceiling (symmetrical, at screenHeight - y - 1 instead of y)
        let texId = ceilingTexture as usize;
        let color = (textures[texId][texI] as u32 |
                        ((textures[texId][texI + 1] as u32) << 8) |
                        ((textures[texId][texI + 2] as u32) << 16));
        let color = (color >> 1) & 8355711; // make a bit darker
        color_buff[(h - y - 1) as usize * w + x as usize] = color as u32;
      }
    }
}

fn render_sprites(sprites: &Vec<Vec<f32>>, textures: &Vec<Vec<u8>>, texture_width: u32, texture_height: u32,color_buff: &mut Vec<u32>, depth_buff: &Vec<f32>, w: usize, h: usize, posX: f32, posY: f32, dirX: f32, dirY: f32, planeX: f32, planeY: f32) {
    let mut sortedSprites = sprites.into_iter()
        .map( |x| (x, ((posX - x[0]) * (posX - x[0]) + (posY - x[1]) * (posY - x[1]))))
        .collect::<Vec<(&Vec<f32>, f32)>>();
    sortedSprites.sort_by( |a, b| b.1.partial_cmp(&a.1).unwrap());
    let sortedSprites : Vec<&Vec<f32>> = sortedSprites
        .into_iter()
        .map(|x| x.0)
        .collect();
    //sqrt not taken, unneeded
    for sprite in sortedSprites {
        let spriteX = sprite[0] - posX;
        let spriteY = sprite[1] - posY;

        //transform sprite with the inverse camera matrix
        // [ planeX   dirX ] -1                                       [ dirY      -dirX ]
        // [               ]       =  1/(planeX*dirY-dirX*planeY) *   [                 ]
        // [ planeY   dirY ]                                          [ -planeY  planeX ]

        let invDet = 1.0 / (planeX * dirY - dirX * planeY); //required for correct matrix multiplication

        let transformX = invDet * (dirY * spriteX - dirX * spriteY);
        let transformY = invDet * (-planeY * spriteX + planeX * spriteY); //this is actually the depth inside the screen, that what Z is in 3D

        let spriteScreenX = ((w as f32 / 2.0) * (1.0 + transformX / transformY)) as usize;

        //calculate height of the sprite on screen
        let spriteHeight = ((h as f32 / (transformY)) as i32).abs(); //using 'transformY' instead of the real distance prevents fisheye
        //calculate lowest and highest pixel to fill in current stripe
        let mut drawStartY = -spriteHeight / 2 + h as i32 / 2;
        if drawStartY < 0 {
            drawStartY = 0;
        }
        let mut drawEndY = spriteHeight / 2 + h as i32 / 2;
        if drawEndY >= h as i32  {
            drawEndY = h as i32 - 1;
        }

        //calculate width of the sprite
        let spriteWidth = ((h as f32/ (transformY)) as i32).abs();
        let mut drawStartX = -spriteWidth / 2 + spriteScreenX as i32;
        if drawStartX < 0 {
            drawStartX = 0;
        }
        let mut drawEndX = spriteWidth / 2 + spriteScreenX as i32;
        if drawEndX >= w as i32 {
            drawEndX = w as i32 - 1;
        }

        //loop through every vertical stripe of the sprite on screen
        for stripe in drawStartX..drawEndX
        {
            let texX = (256 * (stripe - (-spriteWidth / 2 + spriteScreenX as i32)) * texture_width as i32 / spriteWidth) as i32 / 256;
            
            //the conditions in the if are:
            //1) it's in front of camera plane so you don't see things behind you
            //2) it's on the screen (left)
            //3) it's on the screen (right)
            //4) ZBuffer, with perpendicular distance
            if transformY > 0.0 && stripe > 0 && stripe < w as i32 && transformY < depth_buff[stripe as usize] {
                for y in drawStartY..drawEndY
                {
                    let d = (y) * 256 - h as i32 * 128 + spriteHeight * 128; //256 and 128 factors to avoid floats
                    let texY = ((d * texture_height as i32) / spriteHeight) / 256;
                    let texI = texX as usize + texY as usize * texture_width as usize;
                    let texI = (texI * 3) as usize;
                    let texId = sprite[2] as usize;
                    let color = (textures[texId][texI] as u32 |
                        ((textures[texId][texI + 1] as u32) << 8) |
                        ((textures[texId][texI + 2] as u32) << 16));
                    if (color & 0x00FFFFFF) != 0 {
                        color_buff[y as usize * w + stripe as usize] = color as u32
                    }
                }
            }
        }
    }
}

fn render_walls(textures: &Vec<Vec<u8>>, texture_width: u32, texture_height: u32, worldMap: &Vec<Vec<u8>>, color_buff: &mut Vec<u32>, depth_buff: &mut Vec<f32>, w: usize, h: usize, posX: f32, posY: f32, dirX: f32, dirY: f32, planeX: f32, planeY: f32) {


  for x in 0..w
  {
      //calculate ray position and direction
      let cameraX = 2.0 * x as f32 / w as f32 - 1.0; //x-coordinate in camera space
      let rayDirX = dirX + planeX * cameraX;
      let rayDirY = dirY + planeY * cameraX;
      //which box of the map we're in
      let mut mapX = posX as i32;
      let mut mapY = posY as i32;

      //length of ray from current position to next x or y-side
      let mut sideDistX = 0.0;
      let mut sideDistY = 0.0;

      //length of ray from one x or y-side to next x or y-side
      let deltaDistX = (1.0 / rayDirX).abs();
      let deltaDistY = (1.0 / rayDirY).abs();
      let mut perpWallDist = 0.0;

      //what direction to step in x or y-direction (either +1 or -1)
      let mut stepX : i32 = 0;
      let mut stepY : i32 = 0;

      let mut hit = 0; //was there a wall hit?
      let mut side = 0; //was a NS or a EW wall hit?
      //calculate step and initial sideDist
      if rayDirX < 0.0
      {
          stepX = -1;
          sideDistX = (posX - mapX as f32) * deltaDistX;
      }
      else
      {
          stepX = 1;
          sideDistX = (mapX as f32 + 1.0 - posX) * deltaDistX;
      }
      if rayDirY < 0.0
      {
          stepY = -1;
          sideDistY = (posY as f32 - mapY as f32) * deltaDistY;
      }
      else
      {
          stepY = 1;
          sideDistY = (mapY  as f32+ 1.0 - posY as f32) * deltaDistY;
      }
      //perform DDA
      while hit == 0
      {
          //jump to next map square, OR in x-direction, OR in y-direction
          if sideDistX < sideDistY
          {
              sideDistX += deltaDistX;
              mapX += stepX;
              side = 0;
          }
          else
          {
              sideDistY += deltaDistY;
              mapY += stepY;
              side = 1;
          }
          //Check if ray has hit a wall
          if worldMap[mapX as usize][mapY as usize] > 0 {
              hit = 1;
          }
      }
      //Calculate distance projected on camera direction (Euclidean distance will give fisheye effect!)
      if side == 0 { 
          perpWallDist = (mapX as f32 - posX + (1.0 - stepX as f32) / 2.0) / rayDirX;
      }
      else { 
          perpWallDist = (mapY as f32 - posY + (1.0 - stepY as f32) / 2.0) / rayDirY;
      }

      //Calculate height of line to draw on screen
      let lineHeight = (h as f32/ perpWallDist) as usize;

      //calculate lowest and highest pixel to fill in current stripe
      let mut drawStart = - (lineHeight as i32) / 2 + h as i32 / 2;
      let drawStartNeg = drawStart;
      if drawStart < 0 { 
          drawStart = 0;
      }
      let mut drawEnd = (lineHeight as i32) / 2 + (h as i32) / 2;
      if drawEnd >= h as i32 {
          drawEnd = h as i32 - 1;
      }

      //choose wall color
      
      let texId = worldMap[mapX as usize][mapY as usize] as usize;
      let mut color = match worldMap[mapX as usize][mapY as usize]
      {
          1=>  0xff0000,
          2=>  0x00ff00,
          3=>  0x0000ff,
          4=>  0xffffff,
          _=> 0x00ffff,
      };

      //give x and y sides different brightness
      if side == 1 {color = color / 2;}

      let mut wallX = 0.0; //where exactly the wall was hit
      if(side == 0) { 
          wallX = posY + perpWallDist * rayDirY;
      }
      else          { 
          wallX = posX + perpWallDist * rayDirX;
      }
      wallX -= wallX.floor();

      //draw the pixels of the stripe as a vertical line
      let mut texX = (wallX * texture_width as f32) as i32;
      if side == 0 && rayDirX > 0.0 {
          texX = texture_width as i32 - texX - 1;
      }
      if side == 1 && rayDirY < 0.0 {
          texX = texture_width as i32 - texX - 1;
      }
      for y in drawStart..drawEnd {
          let texY = (y - drawStartNeg) * texture_height as i32 / (lineHeight as i32);
          let texI = texX as usize + texY as usize * texture_width as usize;
          let texI = (texI * 3) as usize;
          let texId = texId - 1;
          color_buff[y as usize * w + x as usize] = (textures[texId][texI] as u32 |
              ((textures[texId][texI + 1] as u32) << 8) |
              ((textures[texId][texI + 2] as u32) << 16));
      }
      depth_buff[x as usize] = perpWallDist;
  }
}

fn move_player(option_event: Option<crossterm_input::InputEvent>,worldMap: &Vec<Vec<u8>>, posX: &mut f32, posY: &mut f32, dirX: &mut f32, dirY: &mut f32, planeX: &mut f32, planeY: &mut f32) {
    let rotSpeed : f32 = 0.1;
    let moveSpeed: f32 = 0.1;
        match option_event {
            Some(crossterm_input::InputEvent::Keyboard(crossterm_input::KeyEvent::Up)) => {
                if worldMap[(*posX + *dirX * moveSpeed) as usize][(*posY) as usize] == 0 { 
                    *posX += *dirX * moveSpeed;
                }
                if worldMap[(*posX) as usize][(*posY + *dirY * moveSpeed) as usize] == 0 {
                    *posY += *dirY * moveSpeed;
                }
            },
            Some(crossterm_input::InputEvent::Keyboard(crossterm_input::KeyEvent::Down)) => {
                if worldMap[(*posX - *dirX * moveSpeed) as usize][(*posY) as usize] == 0 { 
                    *posX -= *dirX * moveSpeed;
                }
                if worldMap[(*posX) as usize][(*posY - *dirY * moveSpeed) as usize] == 0 {
                    *posY -= *dirY * moveSpeed;
                }
            },
            Some(crossterm_input::InputEvent::Keyboard(crossterm_input::KeyEvent::Esc)) => {
                std::process::exit(1);
            },
            Some(crossterm_input::InputEvent::Keyboard(crossterm_input::KeyEvent::Right)) => {
                let oldDirX = *dirX;
                *dirX = *dirX * (-rotSpeed).cos() - *dirY * (-rotSpeed).sin();
                *dirY = oldDirX * (-rotSpeed).sin() + *dirY * (-rotSpeed).cos();
                let oldPlaneX = *planeX;
                *planeX = *planeX * (-rotSpeed).cos() - *planeY * (-rotSpeed).sin();
                *planeY = oldPlaneX * (-rotSpeed).sin() + *planeY * (-rotSpeed).cos();
            },
            Some(crossterm_input::InputEvent::Keyboard(crossterm_input::KeyEvent::Left)) => {
                let oldDirX = *dirX;
                *dirX = *dirX * (rotSpeed).cos()  - *dirY * (rotSpeed).sin();
                *dirY = oldDirX * (rotSpeed).sin() + *dirY * (rotSpeed).cos();
                let oldPlaneX = *planeX;
                *planeX = *planeX * (rotSpeed).cos()  - *planeY * (rotSpeed).sin();
                *planeY = oldPlaneX * (rotSpeed).sin() + *planeY * (rotSpeed).cos();
            },
            _ => {}
        }
}

fn server(address: String) {
        let mut world_map : Vec<Vec<u8>> =
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
            let mut sprites = 
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
    // Creates the socket
    let mut socket = Socket::bind(address).unwrap();
    let packet_sender = socket.get_packet_sender();
    let event_receiver = socket.get_event_receiver();
    // Starts the socket, which will start a poll mechanism to receive and send messages.
    let _thread = thread::spawn(move || socket.start_polling());

    let mut positions = HashMap::new();

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
                                positions.insert(endpoint, pos);
                                println!("positions {:?}", positions);
                                let mut positionsClone = HashMap::new();
                                for (key, value) in &positions {
                                    if key != &endpoint {
                                        positionsClone.insert(key.clone(), value.clone());
                                    }
                                }
                                let positionsMessage = ServerMessage::MessagePositions(positionsClone);
                                let posSer = bincode::serialize(&positionsMessage).unwrap();
                                packet_sender.send(Packet::reliable_unordered(endpoint, posSer)).unwrap();
                            },
                            ClientMessage::MessageHello => {
                                let map_message = ServerMessage::MessageWorldMap(world_map.clone());
                                let message_ser = bincode::serialize(&map_message).unwrap();
                                packet_sender.send(Packet::reliable_unordered(endpoint, message_ser)).unwrap();
                                let sprites_message = ServerMessage::MessageSprites(sprites.clone());
                                let message_ser = bincode::serialize(&sprites_message).unwrap();
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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 3 {

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

        let mut dirX = -1.0;
        let mut dirY = 0.0;
        let mut posX = 22.0;
        let mut posY = 12.0;
        let mut planeX = 0.0;
        let mut planeY = 0.66; //the 2d raycaster version of camera plane

        let mut worldMap=
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
                let img = image::open("pics/eagle.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb();
                let texture_width = img.width();
                let texture_height = img.height();
                let raw = img.into_raw();
                let textures = vec![
                    image::open("pics/eagle.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw(),
                    image::open("pics/redbrick.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw(),
                    image::open("pics/purplestone.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw(),
                    image::open("pics/greystone.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw(),
                    image::open("pics/bluestone.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw(),
                    image::open("pics/mossy.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw(),
                    image::open("pics/wood.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw(),
                    image::open("pics/colorstone.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw(),
                    image::open("pics/barrel.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw(),
                    image::open("pics/pillar.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw(),
                    image::open("pics/greenlight.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw(),
                    image::open("free-pics/character.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw(),
                ];

                let mut socket = Socket::bind(args[2].clone()).unwrap();
                let packet_sender = socket.get_packet_sender();
                let event_receiver = socket.get_event_receiver();
                let _thread = thread::spawn(move || socket.start_polling());
                let server = args[1].parse().unwrap();

                let mut i = 0;
                let mut startup = true;
                let mut characters : Vec<Vec<f32>> = vec![];
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
                                            ServerMessage::MessageWorldMap(map) => {
                                                worldMap = map;
                                            },
                                            ServerMessage::MessagePositions(positions) => {
                                                characters = vec![];
                                                for position in positions {
                                                    characters.push(vec![position.1.x, position.1.y, 11.0]);
                                                }
                                            },
                                        }
                                    },
                                    SocketEvent::Connect(connect_event) => { /* a client connected */ },
                                    SocketEvent::Timeout(timeout_event) => { /* a client timed out */},
                                }
                            }
                            Err(_) => {
                                stuff_to_read = false;
                            }
                        }
                    }
                    if i > 30 {
                        if startup {
                            let message = ClientMessage::MessageHello;
                            let messageSer = bincode::serialize(&message).unwrap();
                            packet_sender.send(Packet::reliable_unordered(server, messageSer)).unwrap();
                            startup = false;
                        }
                        let pos = ClientMessage::MessagePosition(Position { x : posX, y : posY });
                        let posSer = bincode::serialize(&pos).unwrap();
                        packet_sender.send(Packet::reliable_unordered(server, posSer)).unwrap();
                        i = 0;
                    }
                    i += 1;

                    let start_time = Instant::now();
                    print!("\x1b[{};0f", 0);
                    render_floor_ceiling(&textures, texture_width, texture_height, &mut color_buff, window_width, window_height, posX, posY, dirX, dirY, planeX, planeY);
                    render_walls(&textures, texture_width, texture_height, &worldMap, &mut color_buff, &mut depth_buff, window_width, window_height, posX, posY, dirX, dirY, planeX, planeY);
                    render_sprites(&sprites, &textures, texture_width, texture_height, &mut color_buff, &depth_buff, window_width, window_height, posX, posY, dirX, dirY, planeX, planeY);
                    render_sprites(&characters, &textures, texture_width, texture_height, &mut color_buff, &depth_buff, window_width, window_height, posX, posY, dirX, dirY, planeX, planeY);
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
                    move_player(option_event, &worldMap, &mut posX, &mut posY, &mut dirX, &mut dirY, &mut planeX, &mut planeY)
                }
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
