use std::path::Path;

use ggez::audio::SoundSource;
use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, Color, DrawParam};
use ggez::input::keyboard::is_key_pressed;
use ggez::{audio, timer, Context, GameResult};
mod door;
mod lighting;
mod map;
mod player;
mod screen;
mod sprite;
mod utilities;
mod render;
use lighting::{Lighting, Torch};
use map::{Map, Type};
use num::clamp;
use player::Player;
use rayon::prelude::*;
use screen::Screen;
use sprite::Sprite;
use utilities::input::{mouse_grabbed_and_hidden, set_mouse_location};
use utilities::vector2::Vector2;
//https://mynoise.net/NoiseMachines/dungeonRPGSoundscapeGenerator.php?l=32343600005816020035&mt=1&tm=1
use crate::utilities::input::get_delta;

const PI: f32 = std::f32::consts::PI;
const RAYSPERPIXEL: usize = 2;
const FOV: f32 = 45.0;
#[allow(dead_code)]
pub struct MainState {
    player: Player,
    map_size: (usize, usize),
    cell_size: f32,
    map: Map,
    angles: Vec<f32>,
    buffer_floors: Vec<f32>,
    buffer_walking: Vec<f32>,
    sky: Sky,
    screen: Screen,
    sprites: Vec<Sprite>,
    lighting_1: Lighting,
    lighting_2: Lighting,
    torch: Torch,
    sounds: Sound,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let (w, h) = graphics::drawable_size(ctx);
        graphics::set_fullscreen(ctx, ggez::conf::FullscreenType::Desktop)?;
        let pos = Vector2::new(8.5, 12.5);
        let dir_norm = Vector2::new(0.0f32, -1.0); // Player direction
        let plane = Vector2::new((FOV.to_radians() * 0.5).tan(), 0.0); //Camera plane vector
        let map_size = (33, 25);
        let cell_size = 128.0;
        let minimap = graphics::Image::new(ctx, "/minimap.png")?;
        let minimap_sb =
            graphics::spritebatch::SpriteBatch::new(graphics::Image::new(ctx, "/sb.png")?);
        let map = Map::new(
            ctx,
            Path::new("/map.png"),
            Path::new("/floor.png"),
            minimap,
            minimap_sb,
            map_size,
        )?;

        let player = Player::new(
            ctx,
            pos,
            dir_norm,
            plane,
            (w * 0.5) / ((FOV.to_radians() * 0.5).tan()), //distance from the player to the projection plane
            0.0,
            150.0,
        )?;

        set_mouse_location(ctx, Vector2::new(w * 0.5, h * 0.5)).unwrap();

        let angles: Vec<f32> = (0..w as usize / RAYSPERPIXEL)
            .map(|r: usize| {
                ((r as f32 * w / (w / (RAYSPERPIXEL as f32) - 1.0) - w * 0.5) / player.planedist)
                    .atan()
            })
            .collect();

        let buffer_floors = (0..h as usize)
            .map(|y| player.planedist / (2.0 * y as f32 - h))
            .collect();

        let buffer_walking = (0..150)
            .map(|i| ((i as f32) / 150.0 * 2.0 * PI).sin())
            .collect();

        let mut skyimg = graphics::Image::new(ctx, "/sky2.png")?;
        skyimg.set_wrap(graphics::WrapMode::Tile, graphics::WrapMode::Mirror);
        skyimg.set_filter(graphics::FilterMode::Nearest);
        let mut sb = graphics::spritebatch::SpriteBatch::new(skyimg);
        let idx = sb.add(DrawParam::default());
        let sky = Sky { sb, idx };

        let wall_textures = graphics::Image::new(ctx, "/wall128.png")?.to_rgba8(ctx)?;

        let sprite_textures = graphics::Image::new(ctx, "/sprite128.png")?.to_rgba8(ctx)?;

        let mut screen = Screen::new(h, w, 128, 128 * 8);
        screen.textures(wall_textures, sprite_textures);

        let sprites = vec![
            Sprite::new(sprite::SpriteType::Armor, Vector2::new(7.5, 7.5)),
            //Sprite::new(sprite::SpriteType::Armor, Vector2::new(7.5, 9.5)),
            //Sprite::new(sprite::SpriteType::CandleHolder, Vector2::new(12.5, 12.5)),
            //Sprite::new(sprite::SpriteType::Bat, Vector2::new(6.5, 12.5)),
            Sprite::new(sprite::SpriteType::Torch, Vector2::new(13.5, 1.048)),
            Sprite::new(sprite::SpriteType::Torch, Vector2::new(8.5, 24.0 - 0.048)),
            Sprite::new(sprite::SpriteType::Torch, Vector2::new(2.048, 3.5)),
            Sprite::new(sprite::SpriteType::Torch, Vector2::new(16.0 - 0.048, 6.5)),
            Sprite::new(sprite::SpriteType::Torch, Vector2::new(28.5, 24.0 - 0.048)),
            Sprite::new(sprite::SpriteType::Torch, Vector2::new(24.5, 1.048)),
            Sprite::new(sprite::SpriteType::Torch, Vector2::new(30.5, 1.048)),
            Sprite::new(sprite::SpriteType::Torch, Vector2::new(27.0 - 0.048, 8.5)),
            Sprite::new(sprite::SpriteType::Torch, Vector2::new(28.048, 8.5)),
            Sprite::new(sprite::SpriteType::Torch, Vector2::new(32.0 - 0.048, 8.5)),
            //Sprite::new(sprite::SpriteType::Gore, Vector2::new(13.0, 3.0)),
        ];

        let lighting_1 = lighting::Lighting::new(
            vec![
                2 + map_size.0 * 3,
                15 + map_size.0 * 6,
                8 + map_size.0 * 23,
                28 + map_size.0 * 23,
                24 + map_size.0 * 1,
                30 + map_size.0 * 1,
                13 + map_size.0 * 1,
                26 + map_size.0 * 8,
                28 + map_size.0 * 8,
                31 + map_size.0 * 8,
            ],
            &map.solid,
            map_size,
        );

        let lighting_2 = lighting::Lighting::from_floor(&lighting_1, map_size);

        let torch = lighting::Torch::default();

        let mut sounds = Sound::new(ctx)?;
        sounds.walking.set_volume(0.02);

        Ok(Self {
            player,
            map_size,
            cell_size,
            map,
            angles,
            buffer_floors,
            buffer_walking,
            sky,
            screen,
            sprites,
            lighting_1,
            lighting_2,
            torch,
            sounds,
        })
    }

    pub fn handle_input(&mut self, ctx: &mut Context) {
        let (w, h) = graphics::drawable_size(ctx);
        let dt = ggez::timer::delta(ctx).as_secs_f32();
        mouse_grabbed_and_hidden(ctx, false, true).unwrap();

        let mut delta_mouse_loc_x = get_delta(ctx).x;
        let mut delta_mouse_loc_y = get_delta(ctx).y;

        set_mouse_location(ctx, Vector2::new(w * 0.5, h * 0.5)).unwrap();

        let mut angle_of_rot = 0.0f32;
        if delta_mouse_loc_x != 0.0 {
            delta_mouse_loc_x -= 240.0; // resolution width middle minus window width middle
        }

        if delta_mouse_loc_y != 0.0 {
            delta_mouse_loc_y -= 135.0;
        }
        self.player.pitch -= delta_mouse_loc_y * 0.75;

        self.player.pitch = clamp(self.player.pitch, -400.0, 400.0);

        angle_of_rot += 0.085 * delta_mouse_loc_x;
        self.player.plane = Vector2::rotate(self.player.plane, angle_of_rot.to_radians());
        self.player.dir_norm = Vector2::rotate(self.player.dir_norm, angle_of_rot.to_radians());

        let mut dir = self.player.dir_norm * (2.5 * dt);
        let yoffset = 0.3125;

        self.player.walking = false;

        if is_key_pressed(ctx, KeyCode::W) {
            let check_pos_y =
                self.player.pos + Vector2::new(0.0, self.player.dir_norm.y.signum() * yoffset);
            let check_pos_x =
                self.player.pos + Vector2::new(self.player.dir_norm.x.signum() * yoffset, 0.0);

            if self.map.solid[(check_pos_y.x) as usize + (check_pos_y.y) as usize * self.map_size.0]
            {
                dir.y = 0.0;
            }
            if self.map.solid[(check_pos_x.x) as usize + (check_pos_x.y) as usize * self.map_size.0]
            {
                dir.x = 0.0;
            }
            self.player.pos += dir;
            self.player.walking = true;
        }
        if is_key_pressed(ctx, KeyCode::S) {
            let check_pos_y =
                self.player.pos + Vector2::new(0.0, -self.player.dir_norm.y.signum() * yoffset);
            let check_pos_x =
                self.player.pos + Vector2::new(-self.player.dir_norm.x.signum() * yoffset, 0.0);

            if self.map.solid[(check_pos_y.x) as usize + (check_pos_y.y) as usize * self.map_size.0]
            {
                dir.y = 0.0;
            }
            if self.map.solid[(check_pos_x.x) as usize + (check_pos_x.y) as usize * self.map_size.0]
            {
                dir.x = 0.0;
            }
            self.player.pos -= dir;
            self.player.walking = true;
        }

        if is_key_pressed(ctx, KeyCode::A) {
            let mut perp_dir = Vector2::new(dir.y, -dir.x);
            let check_pos_y =
                self.player.pos + Vector2::new(0.0, -self.player.dir_norm.x.signum() * yoffset);
            let check_pos_x =
                self.player.pos + Vector2::new(self.player.dir_norm.y.signum() * yoffset, 0.0);

            if self.map.solid[(check_pos_y.x) as usize + (check_pos_y.y) as usize * self.map_size.0]
            {
                perp_dir.y = 0.0;
            }
            if self.map.solid[(check_pos_x.x) as usize + (check_pos_x.y) as usize * self.map_size.0]
            {
                perp_dir.x = 0.0;
            }
            if is_key_pressed(ctx, KeyCode::W) && !is_key_pressed(ctx, KeyCode::D) {
                self.player.pos +=
                    dir * (-1.0) + (dir + perp_dir) * (std::f32::consts::SQRT_2 / (2.0));
            } else if is_key_pressed(ctx, KeyCode::S) && !is_key_pressed(ctx, KeyCode::D) {
                self.player.pos += dir + (dir * -1.0 + perp_dir) * (std::f32::consts::SQRT_2 / 2.0);
            } else {
                self.player.pos += perp_dir;
            }
            self.player.walking = true;
        }
        if is_key_pressed(ctx, KeyCode::D) {
            let mut perp_dir = Vector2::new(-dir.y, dir.x);
            let check_pos_y =
                self.player.pos + Vector2::new(0.0, self.player.dir_norm.x.signum() * yoffset);
            let check_pos_x =
                self.player.pos + Vector2::new(-self.player.dir_norm.y.signum() * yoffset, 0.0);

            if self.map.solid[(check_pos_y.x) as usize + (check_pos_y.y) as usize * self.map_size.0]
            {
                perp_dir.y = 0.0;
            }
            if self.map.solid[(check_pos_x.x) as usize + (check_pos_x.y) as usize * self.map_size.0]
            {
                perp_dir.x = 0.0;
            }
            if is_key_pressed(ctx, KeyCode::W) && !is_key_pressed(ctx, KeyCode::A) {
                self.player.pos +=
                    dir * (-1.0) + (dir + perp_dir) * (std::f32::consts::SQRT_2 / 2.0);
            } else if is_key_pressed(ctx, KeyCode::S) && !is_key_pressed(ctx, KeyCode::A) {
                self.player.pos += dir + (dir * -1.0 + perp_dir) * (std::f32::consts::SQRT_2 / 2.0);
            } else {
                self.player.pos += perp_dir;
            }
            self.player.walking = true;
        }

        if is_key_pressed(ctx, KeyCode::Space) {
            let check_front = self.player.pos + self.player.dir_norm * 1.5;
            let pos_door = (check_front.x) as usize + (check_front.y) as usize * self.map_size.0;

            if self.map.walls[pos_door] == Type::WoodenDoor {
                let door = self.map.doors.get_mut(&pos_door).expect("Cant find door");
                if !door.opening {
                    door.timer = timer::time_since_start(ctx).as_secs_f32();
                    door.opening = true;
                }
            }
        }

        if is_key_pressed(ctx, KeyCode::LControl) {
            if self.player.height > -300.0 {
                self.player.height -= 30.0;
            }
        } else if self.player.height < 150.0 {
            self.player.height += 30.0;
        }

        if is_key_pressed(ctx, KeyCode::Q) {
            self.player.jump += 10.0;
        }

        if is_key_pressed(ctx, KeyCode::E) {
            self.player.jump -= 10.0;
        }
    }

  
}

impl EventHandler for MainState {
    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        match keycode {
            KeyCode::L => {self.lighting_1.switch = !self.lighting_1.switch;
                self.lighting_2.switch = !self.lighting_2.switch;},
            KeyCode::K => {self.lighting_1.smooth_switch = !self.lighting_1.smooth_switch;
                self.lighting_2.smooth_switch = !self.lighting_2.smooth_switch;}
            KeyCode::Escape => ggez::event::quit(ctx),
            _ => (),
        }
    }
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let time = timer::time_since_start(ctx).as_secs_f32();
        let dt = ggez::timer::delta(ctx).as_secs_f32();

        self.handle_input(ctx);

        self.player.update(self.map.walls[self.player.pos.x as usize +self.player.pos.y as usize *self.map_size.0],&self.buffer_walking, time);

        if self.map.walls[self.player.pos.x as usize + self.player.pos.y as usize*self.map_size.0]==Type::Stairs{
            self.player.height = (self.player.pos.x.fract())*self.player.planedist;
        }

        if self.player.walking {
            if self.sounds.walking.paused() {
                self.sounds.walking.resume();
            } else if !self.sounds.walking.playing() {
                self.sounds.walking.play_later().unwrap();
            }
        } else {
            if self.sounds.walking.playing() {
                self.sounds.walking.pause();
            }
        }

        self.sprites
            .iter_mut()
            .for_each(|sprite| sprite.update(time));

        self.map.doors.iter_mut().for_each(|(_, d)| {
            if d.opening {
                d.update(dt, &mut self.map.solid)
            }
        });

        self.torch.update_intensity(time);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let (w, h) = graphics::drawable_size(ctx);
        graphics::clear(ctx, Color::BLACK);
        /*let mut corr_angle = self.player.dir_norm.angle();
        if corr_angle < 0.0 {
            corr_angle += 2.0 * PI;
        }
        let draw_param = graphics::DrawParam {
            src: graphics::Rect::new(
                360.0 / FOV * corr_angle / (2.0 * PI),
                0.4 - self.player.pitch / 864.0,
                1.0,
                1.0,
            ),
            ..Default::default()
        };
        self.sky.sb.set(self.sky.idx, draw_param)?;
        graphics::draw(ctx, &self.sky.sb, draw_param)?;*/

        (0..h as usize).for_each(|y| {
            if y < (self.player.pitch + h ) as usize {
                // Calculate ceiling y buffer
                self.buffer_floors[y] = (3.0*self.player.planedist - 2.0 * self.player.jump)
                    / (-2.0 * (y as f32 - self.player.pitch) + h);
            }
            if y > (h * 0.5 + self.player.pitch) as usize {
                // Calculate floor y buffer
                self.buffer_floors[y] = (self.player.planedist + 2.0 * self.player.jump)
                    / (2.0 * (-self.player.pitch + y as f32) - h);
            }
        });

        self.sprites
            .iter_mut()
            .for_each(|sprite| sprite.set_drawing_bounds(ctx, &self.player, RAYSPERPIXEL as f32));

        self.sprites.sort_by(|a: &Sprite, b: &Sprite| {
            b.calculate_distance_2(&self.player)
                .partial_cmp(&a.distance2)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut img_arr = std::mem::take(&mut self.screen.img_arr);

        img_arr
            .par_chunks_mut(h as usize * 4 * RAYSPERPIXEL)
            .enumerate()
            .for_each(|(j, slice)| {
                render::draw_slice(&self, slice, w as usize / RAYSPERPIXEL - j - 1, h);
                let (slice1, slice2) = slice.split_at_mut(h as usize * 4);
                slice2
                    .chunks_mut(h as usize * 4)
                    .for_each(|sub_slice2| sub_slice2.copy_from_slice(&slice1))
            });
        /*img_arr.par_chunks_mut(h as usize * 4 * RAYSPERPIXEL). for_each(|slice|{
            let (slice1, slice2) =  slice.split_at_mut(h as usize * 2 * RAYSPERPIXEL);
        slice2.copy_from_slice(&slice1)});*/
        self.screen.img_arr = img_arr;

        let img = self.screen.arr_to_rgba(ctx)?;

        graphics::draw(
            ctx,
            &img,
            DrawParam::default()
                .offset([0.5, 0.5])
                .rotation(std::f32::consts::FRAC_PI_2)
                .dest([w * 0.5, h * 0.5]),
        )?;

        draw_fps_counter(ctx)?;

        self.map.draw_minimap(ctx, self.map_size, &self.player)?;

        self.player.draw_circle(ctx)?;

        graphics::present(ctx)
    }
}

pub fn draw_fps_counter(ctx: &mut Context) -> GameResult<()> {
    let fps = timer::fps(ctx);
    let delta = timer::delta(ctx);
    let stats_display = graphics::Text::new(format!("FPS: {:.3}, delta: {:.3?}", fps, delta));

    graphics::draw(
        ctx,
        &stats_display,
        DrawParam::new()
            .dest([0.0, 0.0])
            .color(graphics::Color::WHITE),
    )
}
pub struct Intersection {
    point: [f32; 2],
    distance: f32,
    map_checkv: usize,
    orientation: Orientation,
    wall_type: usize,
}

impl Intersection {
    pub fn new(
        point: [f32; 2],
        distance: f32,
        map_checkv: usize,
        orientation: Orientation,
        wall_type: usize,
    ) -> Self {
        Self {
            point,
            distance,
            map_checkv,
            orientation,
            wall_type,
        }
    }
}
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Orientation {
    N = 1,
    E = 2,
    S = 3,
    W = 4,
}


#[allow(dead_code)]
pub struct Sky {
    sb: graphics::spritebatch::SpriteBatch,
    idx: graphics::spritebatch::SpriteIdx,
}

pub struct Sound {
    walking: audio::Source,
}

impl Sound {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        Ok(Self {
            walking: audio::Source::new(ctx, "/sounds/walking.ogg")?,
        })
    }
}