use crate::protocol::*;
use std::io::Read;
use crossterm::terminal;
use image::imageops::FilterType;
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::source::SystemSource;
use pathfinder_geometry::transform2d::Transform2F;
use font_kit::hinting::HintingOptions;
use pathfinder_geometry::vector::{Vector2F, Vector2I};
use laminar::{Socket, SocketEvent, Packet};
use std::time::{Duration, Instant};
use std::thread;
use std::fs::File;
use std::io::BufReader;
use rodio::Source;
use std::collections::HashMap;
use std::io;
use std::io::Write;

fn flush_stdout() {
    let _ = io::stdout().flush().unwrap();
}


fn smcup() {
    print!("\x1b[?1049h");
    flush_stdout();
}

fn rmcup() {
    print!("\x1b[?1049l");
    flush_stdout();
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

fn render_sprites(all_sprites_and_textures: &Vec<(&Vec<Vec<f32>>, &Vec<Vec<u8>>, u32, u32, bool, bool)>, color_buff: &mut Vec<u32>, depth_buff: &Vec<f32>, w: usize, h: usize, pos_x: f32, pos_y: f32, dir_x: f32, dir_y: f32, plane_x: f32, plane_y: f32, t: i32) -> bool {
    let mut rendering_occured = false;
    let mut portal_takes_full_screen = true;
    for sprites_and_textures in all_sprites_and_textures {
        let (sprites, textures, texture_width, texture_height, rgba, portal_mapping) =  *sprites_and_textures;
        let bytes_per_pixel = if rgba { 4 } else { 3 };
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
            let draw_start_y_no_limit = -sprite_height / 2 + h as i32 / 2;
            let mut draw_start_y = draw_start_y_no_limit;
            if draw_start_y < 0 {
                draw_start_y = 0;
            }
            let draw_end_y_no_limit = sprite_height / 2 + h as i32 / 2;
            let mut draw_end_y = draw_end_y_no_limit;
            if draw_end_y >= h as i32  {
                draw_end_y = h as i32 - 1;
            }

            //calculate width of the sprite
            let sprite_width = ((h as f32/ (transform_y)) as i32).abs();
            let draw_start_x_no_limit = -sprite_width / 2 + sprite_screen_x as i32;
            let mut draw_start_x = draw_start_x_no_limit;
            if draw_start_x < 0 {
                draw_start_x = 0;
            }
            let draw_end_x_no_limit = sprite_width / 2 + sprite_screen_x as i32;
            let mut draw_end_x = draw_end_x_no_limit;
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
                        rendering_occured = true;
                        if portal_mapping && (stripe <= draw_start_x_no_limit + 3 || stripe >= draw_end_x_no_limit - 3) {
                            portal_takes_full_screen = false;
                            let r = ((y + t) * 10) % 0xff;
                            color_buff[y as usize * w + stripe as usize] = (0xffff00 | r) as u32 
                        }
                        else {
                            let d = (y) * 256 - h as i32 * 128 + sprite_height * 128; //256 and 128 factors to avoid floats
                            let tex_y = ((d * texture_height as i32) / sprite_height) / 256;
                            let tex_i = if portal_mapping {
                                stripe as usize + y as usize * texture_width as usize
                            }
                            else {
                                tex_x as usize + tex_y as usize * texture_width as usize
                            };
                            let tex_i = (tex_i * bytes_per_pixel) as usize;
                            let tex_id = sprite[2] as usize;
                            let color = textures[tex_id][tex_i] as u32 |
                                ((textures[tex_id][tex_i + 1] as u32) << 8) |
                                ((textures[tex_id][tex_i + 2] as u32) << 16);
                            if rgba {
                                let alpha = textures[tex_id][tex_i + 3] as u32;
                                let tran = 0xff - alpha;
                                let text = &textures[tex_id];
                                let cbi = y as usize * w + stripe as usize;
                                color_buff[cbi] = 
                                    ((text[tex_i] as u32 & alpha) | (color_buff[cbi] & tran))
                                    + (((((text[tex_i + 1] as u32) & alpha) << 8)) | ((color_buff[cbi]) & (tran << 8)))
                                    + (((((text[tex_i + 2] as u32) & alpha ) << 16))| ((color_buff[cbi]) & (tran << 16)));
                            }
                            else if (color & 0x00_fFFFFF) != 0 {
                                color_buff[y as usize * w + stripe as usize] = color as u32
                            }
                        }
                    }
                }
            }
        }
    }
    rendering_occured && portal_takes_full_screen
}

fn render_walls(textures: &Vec<Vec<u8>>, texture_width: u32, texture_height: u32, world_map: &Vec<Vec<u8>>, color_buff: &mut Vec<u32>, depth_buff: &mut Vec<f32>, w: usize, h: usize, pos_x: f32, pos_y: f32, dir_x: f32, dir_y: f32, plane_x: f32, plane_y: f32, start_dist: f32) {


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
      let mut ray_out_of_map = false;
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
          if (side_dist_x * side_dist_x + side_dist_y * side_dist_y).sqrt() >= start_dist {
              //Check if ray has hit a wall
              if map_x as usize >= world_map.len() || map_y as usize >= world_map[map_x as usize].len() {
                  ray_out_of_map = true;
                  break;
              }
              if world_map[map_x as usize][map_y as usize] > 0 {
                  hit = 1;
              }
          }
      }
      if ray_out_of_map {
          continue;
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
                rmcup();
                let _screen = crossterm_input::RawScreen::disable_raw_mode();
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
            Some(crossterm_input::InputEvent::Keyboard(crossterm_input::KeyEvent::CtrlRight)) => {
                let _dir_x = *dir_x * (3.14/2.0 as f32).cos()  - *dir_y * (3.14/2.0 as f32).sin();
                let _dir_y = *dir_x * (3.14/2.0 as f32).sin() + *dir_y * (3.14/2.0 as f32).cos();
                if world_map[(*pos_x - _dir_x * move_speed) as usize][(*pos_y) as usize] == 0 { 
                    *pos_x -= _dir_x * move_speed;
                }
                if world_map[(*pos_x) as usize][(*pos_y - _dir_y * move_speed) as usize] == 0 {
                    *pos_y -= _dir_y * move_speed;
                }
                res = -move_speed;
            },
            Some(crossterm_input::InputEvent::Keyboard(crossterm_input::KeyEvent::CtrlLeft)) => {
                let _dir_x = *dir_x * (3.14/2.0 as f32).cos()  - *dir_y * (3.14/2.0 as f32).sin();
                let _dir_y = *dir_x * (3.14/2.0 as f32).sin() + *dir_y * (3.14/2.0 as f32).cos();
                if world_map[(*pos_x - _dir_x * move_speed) as usize][(*pos_y) as usize] == 0 { 
                    *pos_x += _dir_x * move_speed;
                }
                if world_map[(*pos_x) as usize][(*pos_y - _dir_y * move_speed) as usize] == 0 {
                    *pos_y += _dir_y * move_speed;
                }
                res = -move_speed;
            },
            _ => {}
        }
        res
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

fn generate_text(text: String, text_width: i32, text_height: i32) -> Vec<u32> {
    let font = SystemSource::new()
        .select_by_postscript_name("DejaVuSans")
        .unwrap()
        .load()
        .unwrap();
    let mut canvas = Canvas::new(Vector2I::new(text_width, text_height), Format::A8);
    let mut i = 0;
    for c in text.char_indices() {
        let glyph_id = font.glyph_for_char(c.1).unwrap();
        font.rasterize_glyph(
            &mut canvas,
            glyph_id,
            32.0, // chosen font size for this example
            Transform2F::from_translation(Vector2F::new(25.0 * i as f32, 32.0)),
            HintingOptions::None,
            RasterizationOptions::GrayscaleAa,
        ).unwrap();
        i += 1;
    }
    canvas.pixels.iter().map(|i| *i as u32).collect()
}

fn play_sound(sound_device: &rodio::Device, path: String) {
    let file = File::open(path).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    let coin_sound_samples = source.convert_samples();
    rodio::play_raw(&sound_device, coin_sound_samples);
}

fn render_portals(sound_device: &rodio::Device, portals: &Vec<Vec<f32>>, portals_dests: &Vec<Vec<f32>>, portal_color_buff: &mut Vec<u32>, portal_depth_buff: &mut Vec<f32>, portal_width: usize, portal_height: usize, portals_textures: &mut Vec<Vec<u8>>, textures: &Vec<Vec<u8>>, character_textures: &Vec<Vec<u8>>, goldcoin_textures: &Vec<Vec<u8>>,torch_textures: &Vec<Vec<u8>>, sprites: &Vec<Vec<f32>>, characters: &Vec<Vec<f32>>, gold_coins: &Vec<Vec<f32>>, torches: &Vec<Vec<f32>>, tex_width: u32, tex_height: u32, coin_width: u32, coin_height: u32, torch_width: u32, torch_height: u32, color_buff: &mut Vec<u32>, depth_buff: &mut Vec<f32>, world_map: &Vec<Vec<u8>>, window_width: usize, window_height: usize, pos_x: &mut f32, pos_y: &mut f32, dir_x:f32, dir_y: f32, plane_x: f32, plane_y: f32, t: i32) {
    for i in 0..portals.len() {
        let dist_x = *pos_x - portals[i][0];
        let dist_y = *pos_y - portals[i][1];
        let dest_pos_x = portals_dests[i][0] + dist_x;
        let dest_pos_y = portals_dests[i][1] + dist_y;
        let start_dist = (dist_x * dist_x + dist_y * dist_y).sqrt();
        let should_render_portal = start_dist < 7.0;
        if should_render_portal {
            render(&textures, &character_textures, &goldcoin_textures, &torch_textures, &sprites, &characters, &gold_coins, &torches, tex_width, tex_height, coin_width, coin_height, torch_width, torch_height, portal_color_buff, portal_depth_buff, &world_map, portal_width, portal_height, dest_pos_x, dest_pos_y, dir_x, dir_y, plane_x, plane_y, start_dist, t);
            for y in 0..portal_height {
                for x in 0..portal_width {
                    let base32 = y * portal_width + x;
                    let base8 = (y * portal_width + x) * 4;
                    portals_textures[i][base8] = (portal_color_buff[base32] & 0xff) as u8;
                    portals_textures[i][base8 + 1] = ((portal_color_buff[base32] >> 8) & 0xff) as u8;
                    portals_textures[i][base8 + 2] = ((portal_color_buff[base32] >> 16) & 0xff) as u8;
                    portals_textures[i][base8 + 3] = 0xff;
                }
            }
            let sprites_and_textures = vec![
                (portals, &*portals_textures, portal_width as u32, portal_height as u32, true, true),
            ];
            if render_sprites(&sprites_and_textures, color_buff, &depth_buff, window_width, window_height, *pos_x, *pos_y, dir_x, dir_y, plane_x, plane_y, t) {

                *pos_x = portals_dests[i][0];
                *pos_y = portals_dests[i][1];
                play_sound(&sound_device, String::from("sound/teleport.mp3"));
            }
        }
    }
}

fn render(textures: &Vec<Vec<u8>>, character_textures: &Vec<Vec<u8>>, goldcoin_textures: &Vec<Vec<u8>>,torch_textures: &Vec<Vec<u8>>, sprites: &Vec<Vec<f32>>, characters: &Vec<Vec<f32>>, gold_coins: &Vec<Vec<f32>>, torches: &Vec<Vec<f32>>, tex_width: u32, tex_height: u32, coin_width: u32, coin_height: u32, torch_width: u32, torch_height: u32, color_buff: &mut Vec<u32>, depth_buff: &mut Vec<f32>, world_map: &Vec<Vec<u8>>, w: usize, h: usize, pos_x: f32, pos_y: f32, dir_x:f32, dir_y: f32, plane_x: f32, plane_y: f32, start_dist: f32, t: i32) {
    render_floor_ceiling(&textures, tex_width, tex_height, color_buff, w, h, pos_x, pos_y, dir_x, dir_y, plane_x, plane_y);
    render_walls(&textures, tex_width, tex_height, &world_map, color_buff, depth_buff, w, h, pos_x, pos_y, dir_x, dir_y, plane_x, plane_y, start_dist);
    let sprites_and_textures = vec![
        (sprites, textures, tex_width, tex_height, false, false),
        (characters, character_textures, tex_width, tex_height, false, false),
        (gold_coins, goldcoin_textures, coin_width, coin_height, true, false),
        (torches, torch_textures, torch_width, torch_height, true, false),
    ];
    render_sprites(&sprites_and_textures, color_buff, &depth_buff, w, h, pos_x, pos_y, dir_x, dir_y, plane_x, plane_y, t);
}

pub fn client(server_address: String, client_address: String, nickname: String) {
    smcup();
    let window_width = 640;
    let window_height = 320;
    let portal_width = window_width;
    let portal_height = window_height;
    let time_per_frame = 1000/ 60;
    let mut color_buff : Vec<u32> = vec![0; window_width * window_height];
    let mut depth_buff : Vec<f32> = vec![0.0; window_width];
    let mut portal_color_buff : Vec<u32> = vec![0; portal_width * portal_height];
    let mut portal_color_buff_u8 : Vec<u8> = vec![0; portal_width * portal_height * 4];
    let mut portal_depth_buff : Vec<f32> = vec![0.0; portal_width];


    let mut term_width = 0 as u32;
    let mut term_height = 0 as u32;

    let sound_device = rodio::default_output_device().unwrap();

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
    for i in 0..portal_width {
        portal_color_buff[(portal_height - 1) * portal_width + i] = 36;
    }
    let _screen = crossterm_input::RawScreen::into_raw_mode();
    let input = crossterm_input::input();
    let mut reader = input.read_async();

    let mut dir_x = -1.0;
    let mut dir_y = 0.0;
    let mut pos_x = 22.0;
    let mut pos_y = 12.0;
    let mut previous_pos_x = 22.0;
    let mut previous_pos_y = 12.0;
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
            image::open("free-pics/character2.png").unwrap().resize(texture_size, texture_size, FilterType::Nearest).to_rgb().into_raw(),
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
        let torch_width = 32;
        let torch_height = 32;
        let torch_textures = vec![
            image::open("torch/Torch-00.png").unwrap().to_rgba().into_raw(),
            image::open("torch/Torch-01.png").unwrap().to_rgba().into_raw(),
            image::open("torch/Torch-02.png").unwrap().to_rgba().into_raw(),
            image::open("torch/Torch-03.png").unwrap().to_rgba().into_raw(),
            image::open("torch/Torch-04.png").unwrap().to_rgba().into_raw(),
            image::open("torch/Torch-05.png").unwrap().to_rgba().into_raw(),
        ];

        let mut torches = vec![
            vec![22.0, 10.1, 0.0]
        ];

        let mut portals = vec![
        ];

        let mut portals_dests = vec![
        ];

        let mut portals_textures = vec![
        ];

        /* font stuff */
        let mut text = String::from("loading...");
        let text_width = 300;
        let text_height = 40;
        let mut text_expires = None;
        let mut text_buff = generate_text(text, text_width, text_height);
        /* end font struff */

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

        let mut t = 0;

        loop {
            t += 1;
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
                                    ServerMessage::MessageTeleport(pos) => {
                                        dir_x = pos.dir_x;
                                        dir_y = pos.dir_y;
                                        pos_x = pos.x;
                                        pos_y = pos.y;
                                    }
                                    ServerMessage::MessageText(txt, duration) => {
                                        text = txt;
                                        text_buff = generate_text(text, text_width, text_height);
                                        text_expires = Some(Instant::now() + duration);
                                    },
                                    ServerMessage::MessageWorldMap(map) => {
                                        world_map = map;
                                    },
                                    ServerMessage::MessageGoldCoins(gcs) => {
                                        play_sound(&sound_device, String::from("sound/picked-coin-echo.mp3"));
                                        gold_coins = vec![];
                                        for gc in gcs {
                                            gold_coins.push(vec![gc.0, gc.1, 0.0]);
                                        }
                                    },
                                    ServerMessage::MessagePortals(pt, ptdst) => {
                                        portals = pt;
                                        portals_dests = ptdst;
                                        portals_textures = vec![];
                                        for i in 0..portals.len() {
                                            portals_textures.push(portal_color_buff_u8.clone());
                                        }
                                    }
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
            if let Some(expiration) = text_expires {
                if now > expiration {
                    text = String::from("");
                    text_buff = generate_text(text, text_width, text_height);
                    text_expires = None;
                }
            }
            if (now - previous) > Duration::from_millis(500) {
                if startup {
                    let message = ClientMessage::MessageHello(nickname.clone());
                    let message_ser = bincode::serialize(&message).unwrap();
                    packet_sender.send(Packet::reliable_unordered(server, message_ser)).unwrap();
                    startup = false;
                }
                if previous_pos_x != pos_x || previous_pos_y != pos_y {
                    // play_sound(&sound_device, String::from("sound/wood03.ogg"));
                }
                let pos = ClientMessage::MessagePosition(Position { x : pos_x, y : pos_y, dir_x: dir_x, dir_y: dir_y, speed: move_speed });
                previous_pos_x = pos_x;
                previous_pos_y = pos_y;
                let pos_ser = bincode::serialize(&pos).unwrap();
                packet_sender.send(Packet::reliable_unordered(server, pos_ser)).unwrap();
                previous = now;
                for i in 0..gold_coins.len() {
                    gold_coins[i][2] += 1.0;
                    if gold_coins[i][2] as usize >= goldcoin_textures.len()  {
                        gold_coins[i][2] = 0.0;
                    }
                }
                for i in 0..torches.len() {
                    torches[i][2] += 1.0;
                    if torches[i][2] as usize >= torch_textures.len() {
                        torches[i][2] = 0.0;
                    }
                }
            }

            let start_time = Instant::now();

            render(&textures, &character_textures, &goldcoin_textures, &torch_textures, &sprites, &characters, &gold_coins, &torches, texture_width, texture_height, coin_width, coin_height, torch_width, torch_height, &mut color_buff, &mut depth_buff, &world_map, window_width, window_height, pos_x, pos_y, dir_x, dir_y, plane_x, plane_y, 0.0, t);
            render_portals(&sound_device, &portals, &portals_dests, &mut portal_color_buff, &mut portal_depth_buff, portal_width, portal_height, &mut portals_textures, &textures, &character_textures, &goldcoin_textures, &torch_textures, &sprites, &characters, &gold_coins, &torches, texture_width, texture_height, coin_width, coin_height, torch_width, torch_height, &mut color_buff, &mut depth_buff, &world_map, portal_width, portal_height, &mut pos_x, &mut pos_y, dir_x, dir_y, plane_x, plane_y, t);

            for y in 0..text_height {
                for x in 0..text_width {
                    let pixel_i_dest = (y * window_width as i32 + x) as usize;
                    let pixel_i_src = (y * text_width as i32 + x) as usize;
                    color_buff[pixel_i_dest] |= text_buff[pixel_i_src];
                }
            }
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

