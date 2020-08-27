extern crate rand;
extern crate image;
extern crate crossterm_input;

use std::thread;
use std::time::{Duration, Instant};
use crossterm::terminal;
use image::imageops::FilterType;


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

fn render_walls(textures: &Vec<Vec<u8>>, texture_width: u32, texture_height: u32, worldMap: &Vec<Vec<u32>>, color_buff: &mut Vec<u32>, depth_buff: &mut Vec<f32>, w: usize, h: usize, posX: f32, posY: f32, dirX: f32, dirY: f32, planeX: f32, planeY: f32) {

  for x in 0..w
  {
      for y in 0..h/2
      {
          color_buff[y * w + x] = 0x444444;
      }
      for y in h/2..h
      {
          color_buff[y * w + x] = 0x888888;
      }
  }

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
          texX = 100 - texX - 1;
      }
      if side == 1 && rayDirY < 0.0 {
          texX = 100 - texX - 1;
      }
      for y in drawStart..drawEnd {
          let texY = (y - drawStartNeg) * texture_height as i32 / (lineHeight as i32);
          let texI = texX as usize + texY as usize * texture_width as usize;
          let texI = (texI * 3) as usize;
          color_buff[y as usize * w + x as usize] = (textures[texId][texI] as u32 |
              ((textures[texId][texI + 1] as u32) << 8) |
              ((textures[texId][texI + 2] as u32) << 16));
      }
      depth_buff[x as usize] = perpWallDist;
  }
}

fn move_player(option_event: Option<crossterm_input::InputEvent>,worldMap: &Vec<Vec<u32>>, posX: &mut f32, posY: &mut f32, dirX: &mut f32, dirY: &mut f32, planeX: &mut f32, planeY: &mut f32) {
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

fn main() {
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
      vec![1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
      vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,0,0,0,0,0,2,2,2,2,2,0,0,0,0,3,0,3,0,3,0,0,0,1],
      vec![1,0,0,0,0,0,2,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,0,0,0,0,0,2,0,0,0,2,0,0,0,0,3,0,0,0,3,0,0,0,1],
      vec![1,0,0,0,0,0,2,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,0,0,0,0,0,2,2,0,2,2,0,0,0,0,3,0,3,0,3,0,0,0,1],
      vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,4,4,4,4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,4,0,4,0,0,0,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,4,0,0,0,0,5,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,4,0,4,0,0,0,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,4,0,4,4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,4,4,4,4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
      vec![1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]
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
    let img = image::open("pics/eagle.png").unwrap().resize(100, 100, FilterType::Nearest).to_rgb();
    let texture_width = img.width();
    let texture_height = img.height();
    let raw = img.into_raw();
    let textures = vec![
        image::open("pics/eagle.png").unwrap().resize(100, 100, FilterType::Nearest).to_rgb().into_raw(),
        image::open("pics/redbrick.png").unwrap().resize(100, 100, FilterType::Nearest).to_rgb().into_raw(),
        image::open("pics/purplestone.png").unwrap().resize(100, 100, FilterType::Nearest).to_rgb().into_raw(),
        image::open("pics/greystone.png").unwrap().resize(100, 100, FilterType::Nearest).to_rgb().into_raw(),
        image::open("pics/bluestone.png").unwrap().resize(100, 100, FilterType::Nearest).to_rgb().into_raw(),
        image::open("pics/mossy.png").unwrap().resize(100, 100, FilterType::Nearest).to_rgb().into_raw(),
        image::open("pics/wood.png").unwrap().resize(100, 100, FilterType::Nearest).to_rgb().into_raw(),
        image::open("pics/colorstone.png").unwrap().resize(100, 100, FilterType::Nearest).to_rgb().into_raw(),
        image::open("pics/barrel.png").unwrap().resize(100, 100, FilterType::Nearest).to_rgb().into_raw(),
        image::open("pics/pillar.png").unwrap().resize(100, 100, FilterType::Nearest).to_rgb().into_raw(),
        image::open("pics/greenlight.png").unwrap().resize(100, 100, FilterType::Nearest).to_rgb().into_raw(),
    ];

    loop {
        let start_time = Instant::now();
        print!("\x1b[{};0f", 0);
        render_walls(&textures, texture_width, texture_height, &worldMap, &mut color_buff, &mut depth_buff, window_width, window_height, posX, posY, dirX, dirY, planeX, planeY);
        render_sprites(&sprites, &textures, texture_width, texture_height, &mut color_buff, &depth_buff, window_width, window_height, posX, posY, dirX, dirY, planeX, planeY);
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
