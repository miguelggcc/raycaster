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
use lighting::{Lighting, Torch};
use map::{Map, Type};
use num::clamp;
use player::Player;
use rayon::prelude::*;
use screen::Screen;
use sprite::Sprite;
use utilities::input::{mouse_grabbed_and_hidden, set_mouse_location};
use utilities::math::ffmin;
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
    lighting: Lighting,
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

        let lighting = lighting::Lighting::new(
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
            lighting,
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

        self.player.pitch = clamp(self.player.pitch, -300.0, 300.0);

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

    pub fn calculate_ray(
        &self,
        ray_dir_player: Vector2<f32>,
        theta: f32,
    ) -> (Intersection, Vec<Intersection>) {
        let ray_dir_norm = Vector2::rotate(ray_dir_player, theta);
        let ray_unitstep_size = Vector2::new(
            (1.0 + (ray_dir_norm.y / ray_dir_norm.x) * (ray_dir_norm.y / ray_dir_norm.x)).sqrt(),
            (1.0 + (ray_dir_norm.x / ray_dir_norm.y) * (ray_dir_norm.x / ray_dir_norm.y)).sqrt(),
        );
        let startv = self.player.pos;

        let mut map_checkv = Vector2::new(startv.x.floor(), startv.y.floor());
        let mut ray_length1_d = Vector2::new(0.0f32, 0.0);
        let mut orientation = Orientation::N;
        let mut wall_type = Type::TiledFloor;
        let mut transparent_walls = vec![];
        let mut stepv = Vector2::new(0.0f32, 0.0);
        let mut last_was_door = false;

        if ray_dir_norm.x < 0.0 {
            stepv.x = -1.0;
            ray_length1_d.x = (startv.x - map_checkv.x) * ray_unitstep_size.x;
        } else {
            stepv.x = 1.0;
            ray_length1_d.x = (map_checkv.x + 1.0 - startv.x) * ray_unitstep_size.x;
        }

        if ray_dir_norm.y < 0.0 {
            stepv.y = -1.0;
            ray_length1_d.y = (startv.y - map_checkv.y) * ray_unitstep_size.y;
        } else {
            stepv.y = 1.0;
            ray_length1_d.y = (map_checkv.y + 1.0 - startv.y) * ray_unitstep_size.y;
        }

        let mut tilefound = false;
        let mut distance = 0.0;

        while !tilefound && distance < 100.0 {
            //arbitrary max distance

            if ray_length1_d.x < ray_length1_d.y {
                map_checkv.x += stepv.x;
                distance = ray_length1_d.x;
                ray_length1_d.x += ray_unitstep_size.x;

                if ray_dir_norm.x < 0.0 {
                    orientation = Orientation::W;
                } else {
                    orientation = Orientation::E;
                }
            } else {
                map_checkv.y += stepv.y;
                distance = ray_length1_d.y;
                ray_length1_d.y += ray_unitstep_size.y;

                if ray_dir_norm.y < 0.0 {
                    orientation = Orientation::S;
                } else {
                    orientation = Orientation::N;
                }
            }
            if map_checkv.x >= 0.0
                && map_checkv.x < self.map_size.0 as f32
                && map_checkv.y >= 0.0
                && map_checkv.y < self.map_size.1 as f32
            {
                wall_type =
                    self.map.walls[map_checkv.y as usize * self.map_size.0 + map_checkv.x as usize];

                if last_was_door && wall_type as usize> 0 {
                    wall_type = Type::FrameWoodenDoor;
                }
                last_was_door = false;
                if wall_type == Type::WoodenDoor {
                    //door
                    let door_offset = self
                        .map
                        .doors
                        .get(&(map_checkv.y as usize * self.map_size.0 + map_checkv.x as usize))
                        .expect("error finding door")
                        .offset;

                    tilefound = true;
                    if orientation == Orientation::N || orientation == Orientation::S {
                        if ray_length1_d.y - 0.5 * ray_unitstep_size.y <= ray_length1_d.x {
                            distance = ray_length1_d.y - ray_unitstep_size.y * 0.5;

                            if door_offset < 1.0 {
                                let pos_x = (startv.x + ray_dir_norm.x * distance).fract();
                                if pos_x > door_offset * 0.5 && 1.0 - pos_x > door_offset * 0.5 {
                                    last_was_door = true;
                                    tilefound = false;
                                }
                            }
                        } else {
                            // side wall
                            if ray_dir_norm.x < 0.0 {
                                orientation = Orientation::W;
                                map_checkv.x -= 1.0;
                            } else {
                                orientation = Orientation::E;
                                map_checkv.x += 1.0;
                            }
                            wall_type = Type::FrameWoodenDoor;
                            distance = ray_length1_d.x;
                        }
                    } else {
                        if ray_length1_d.x - 0.5 * ray_unitstep_size.x <= ray_length1_d.y {
                            distance = ray_length1_d.x - ray_unitstep_size.x * 0.5;
                            if door_offset < 1.0 {
                                let pos_y = (startv.y + ray_dir_norm.y * distance).fract();
                                if pos_y > door_offset * 0.5 && 1.0 - pos_y > door_offset * 0.5 {
                                    last_was_door = true;
                                    tilefound = false;
                                }
                            }
                        } else {
                            if ray_dir_norm.y < 0.0 {
                                orientation = Orientation::S;
                                map_checkv.y -= 1.0;
                            } else {
                                orientation = Orientation::N;
                                map_checkv.y += 1.0;
                            }
                            wall_type = Type::FrameWoodenDoor;
                            distance = ray_length1_d.y;
                        }
                    }
                } else if wall_type == Type::Cowbeb || wall_type == Type::MetalBars {
                    let mut offset = 0.0;
                    if orientation == Orientation::N || orientation == Orientation::S {
                        if ray_length1_d.y - 0.5 * ray_unitstep_size.y <= ray_length1_d.x {
                            distance = ray_length1_d.y - ray_unitstep_size.y * 0.5;
                        } else {
                            if ray_dir_norm.x < 0.0 {
                                orientation = Orientation::W;
                                offset = -1.0;
                            } else {
                                orientation = Orientation::E;
                                offset = 1.0;
                            }
                            distance = ray_length1_d.x;
                        }
                        transparent_walls.push(Intersection::new(
                            (startv + ray_dir_norm * distance).to_array(),
                            distance,
                            (map_checkv.y) as usize * self.map_size.0
                                + (map_checkv.x + offset) as usize,
                            orientation.clone(),
                            self.map.walls[(map_checkv.y) as usize * self.map_size.0
                                + (map_checkv.x + offset) as usize] as usize,
                        ));
                    } else {
                        if ray_length1_d.x - 0.5 * ray_unitstep_size.x <= ray_length1_d.y {
                            distance = ray_length1_d.x - ray_unitstep_size.x * 0.5;
                        } else {
                            if ray_dir_norm.y < 0.0 {
                                orientation = Orientation::S;
                                offset = -1.0;
                            } else {
                                orientation = Orientation::N;
                                offset = 1.0;
                            }
                            distance = ray_length1_d.y;
                        }
                        transparent_walls.push(Intersection::new(
                            (startv + ray_dir_norm * distance).to_array(),
                            distance,
                            (map_checkv.y + offset) as usize * self.map_size.0
                                + (map_checkv.x) as usize,
                            orientation.clone(),
                            self.map.walls[(map_checkv.y + offset) as usize * self.map_size.0
                                + (map_checkv.x) as usize] as usize,
                        ));
                    }
                } else if wall_type == Type::Stairs  {
                    transparent_walls.push(Intersection::new(
                        (startv + ray_dir_norm * distance).to_array(),
                        distance,
                        (map_checkv.y) as usize * self.map_size.0 + (map_checkv.x) as usize,
                        orientation.clone(),
                        wall_type as usize,
                    ));
                } else if wall_type as usize> 0 && wall_type != Type::Stairs2 {
                    tilefound = true;
                }
                if ((orientation == Orientation::W || orientation == Orientation::E)
                    && self.map.walls[map_checkv.y as usize * self.map_size.0
                        + (map_checkv.x - stepv.x) as usize]
                        == Type::WoodenDoor)
                    || ((orientation == Orientation::N || orientation == Orientation::S)
                        && self.map.walls[(map_checkv.y - stepv.y) as usize * self.map_size.0
                            + map_checkv.x as usize]
                            == Type::WoodenDoor)
                {
                    wall_type = Type::FrameWoodenDoor;
                }

                if self.map.walls[startv.x as usize + startv.y as usize*self.map_size.0]==Type::Stairs{
                    let m = ray_dir_norm.y/ray_dir_norm.x;
                    let pos_x0 = self.player.pos.x.floor();
                    let iy = m * (pos_x0-self.player.pos.x)+self.player.pos.y; // intersection y with the first step
                            let distance =-self.player.pos.x.fract()*(1.0
                                + m *m)
                                .sqrt();
                    transparent_walls.push(Intersection::new(
                        [self.player.pos.x.floor(), iy],
                        distance,
                        (startv.y) as usize * self.map_size.0 + (startv.x) as usize,
                        orientation.clone(),
                        12,
                    ));
                }
                if tilefound {
                    break;
                }
            }
        }
        let int_point = startv + ray_dir_norm * distance;
        (
            Intersection::new(
                int_point.to_array(),
                distance,
                map_checkv.y as usize * self.map_size.0 + map_checkv.x as usize,
                orientation,
                wall_type as usize,
            ),
            transparent_walls,
        )
    }

    fn draw_slice(&self, slice: &mut [u8], j: usize, h: f32) {
        let (intersection, transparent_walls) =
            self.calculate_ray(self.player.dir_norm, self.angles[j]);

        let corrected_distance = intersection.distance * self.angles[j].cos();
        let pos_z = self.player.jump / corrected_distance + self.player.pitch;

        let rect_h = (self.player.planedist / corrected_distance * 100.0).round() / 100.0;
        let rect_ceiling = (h - rect_h) * 0.5;
        let rect_floor = (h + rect_h) * 0.5;

        self.draw_wall(slice, h, &intersection, h * 0.5, rect_h, &pos_z);

        //draw floor
        for y in (pos_z + rect_floor) as usize..(h) as usize {
            if !(j > 24 / RAYSPERPIXEL && j < 308 / RAYSPERPIXEL && y > 805) {
                // Don't draw the floor behind the minimap image
                let current_dist = self.buffer_floors[y]; // Use a buffer since they're always the same values
                let weight = current_dist / corrected_distance;

                let rhs = self.player.pos * (1.0 - weight);
                let current_floor_x = weight * intersection.point[0] + rhs.x;
                let current_floor_y = weight * intersection.point[1] + rhs.y;

                let location =
                unsafe{
                    current_floor_x.to_int_unchecked::<usize>() + current_floor_y.to_int_unchecked::<usize>() * self.map_size.0}; //Cant be negative
                let floor_type = self.map.floors[location];
                let ftx =  unsafe { (current_floor_x * 128.0).to_int_unchecked::<usize>() % 128 }; //Cant be negative
                let fty = unsafe { (current_floor_y * 128.0).to_int_unchecked::<usize>() % 128 }; //Cant be negative
                let lighting = self.lighting.get_lighting_floor(
                    ftx as f32,
                    fty as f32,
                    location,
                );
                self.screen.draw_texture(
                    slice,
                    [ftx, (floor_type * 128) + fty],
                    y,
                    self.torch.intensity * lighting,
                    ffmin(3.0 / (current_dist * current_dist),1.5),
                )
            }
        }
        //draw ceiling
        let mut rect_top_draw = rect_ceiling;
        if rect_ceiling + pos_z > h {
            rect_top_draw = h - pos_z;
        }
        for y in 0..(rect_top_draw + pos_z) as usize {
            let current_dist = self.buffer_floors[y];
            let weight = current_dist / corrected_distance;

            let rhs = self.player.pos * (1.0 - weight);
            let current_floor_x = weight * intersection.point[0] + rhs.x;
            let current_floor_y = weight * intersection.point[1] + rhs.y;

            let location =
                unsafe{
                    current_floor_x.to_int_unchecked::<usize>() + current_floor_y.to_int_unchecked::<usize>() * self.map_size.0};

                let ftx =  unsafe { (current_floor_x * 128.0).to_int_unchecked::<usize>() % 128 };
                let fty = unsafe { (current_floor_y * 128.0).to_int_unchecked::<usize>() % 128 };

            self.screen.draw_texture(
                slice,
                [ftx, 1152 + fty],
                y,
                self.torch.intensity
                    * self.lighting.get_lighting_floor(
                        ftx as f32,
                        fty as f32,
                        location,
                    ),
                ffmin(3.0 / (current_dist * current_dist),1.5),
            );
            //self.screen.draw_pixel(slice, y as usize, &[0, 0, 0, 0]);
        }
        if !&transparent_walls.is_empty() {
            let mut twandsp = transparent_walls
                .into_iter()
                .map(|e| TWandSprites::TW(e))
                .collect::<Vec<_>>();
            twandsp.extend(self.sprites.iter().map(|e| TWandSprites::Sprites(e)));

            twandsp.sort_by(|a, b| {
                b.distance2()
                    .partial_cmp(&a.distance2())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            twandsp.iter().for_each(|e| {
                match e {
                    TWandSprites::Sprites(sprite) => {
                        sprite.draw(slice, &self.player, j, &self.screen, corrected_distance)
                    }
                    TWandSprites::TW(tw) => {
                        let tw_corrected_distance = tw.distance * self.angles[j].cos();
                        if tw.wall_type == 10 || tw.wall_type == 11 {
                            self.draw_wall(
                                slice,
                                h,
                                &tw,
                                h * 0.5,
                                self.player.planedist / tw_corrected_distance,
                                &(self.player.pitch + self.player.jump / tw_corrected_distance),
                            )
                        } else if tw.wall_type == 12
                            && (tw.orientation == Orientation::W
                                || tw.orientation == Orientation::E)
                        {
                            let signum = if tw.orientation == Orientation::E {
                                1.0
                            } else {
                                -1.0
                            };
                            let m = (self.player.pos.y - tw.point[1])
                                / (self.player.pos.x - tw.point[0]); //slope of the line
                            let iy = m * (tw.point[0] + signum / 4.0 - self.player.pos.x)
                                + self.player.pos.y; // intersection y with the first step
                            let delta_distance = (1.0 / (4.0 * 4.0)
                                + (iy - tw.point[1]) * (iy - tw.point[1]))
                                .sqrt(); // distance between steps
                                let start =  if self.map.walls[self.player.pos.x as usize +self.player.pos.y as usize *self.map_size.0]==Type::Stairs{
                                    (self.player.pos.x.fract()*4.0).ceil() as usize
                                }else{
                                        1
                                    };
                            for i in (start..8).rev() {
                                let tw2 = Intersection::new(
                                    [
                                        tw.point[0] + signum * (i as f32) / 4.0,
                                        m * (tw.point[0] + signum * (i as f32) / 4.0
                                            - self.player.pos.x)
                                            + self.player.pos.y,
                                    ],
                                    tw.distance + delta_distance * (i as f32),
                                    tw.map_checkv,
                                    tw.orientation,
                                    12,
                                );
                                if tw2.point[1] > tw.point[1] -5.0
                                    && tw2.point[1] < tw.point[1] +5.0
                                {
                                    if self.map.walls[tw2.point[0] as usize
                                        + tw2.point[1] as usize * self.map_size.0]
                                        == Type::Stairs
                                        || self.map.walls[tw2.point[0] as usize
                                            + tw2.point[1] as usize * self.map_size.0]
                                            == Type::Stairs2
                                    {
                                        self.draw_wall(
                                            slice,
                                            h,
                                            &tw2,
                                            h * 0.5
                                                + self.player.planedist
                                                    / (tw2.distance * self.angles[j].cos())
                                                    * ((3.0 - i as f32) / 8.0 + 1.0 / 16.0),
                                            1.0 / 8.0 * self.player.planedist
                                                / (tw2.distance * self.angles[j].cos()),
                                            &(self.player.pitch
                                                + self.player.jump
                                                    / (tw2.distance * self.angles[j].cos())),
                                        );
                                        for y in (self.player.pitch
                                            + self.player.jump
                                                / (tw2.distance * self.angles[j].cos())
                                            + h * 0.5
                                            + self.player.planedist
                                                / (tw2.distance * self.angles[j].cos())
                                                * ((3.0 - i as f32) / 8.0 + 1.0 / 8.0))
                                            as usize
                                            ..(self.player.pitch
                                                + self.player.jump
                                                    / ((tw2.distance - delta_distance)
                                                        * self.angles[j].cos())
                                                + h * 0.5
                                                + self.player.planedist
                                                    / ((tw2.distance - delta_distance)
                                                        * self.angles[j].cos())
                                                    * ((4.0 - i as f32) / 8.0)).min(h)
                                                as usize
                                        {
                                            let current_dist = (self.player.planedist
                                                * (1.0 - (i as f32) / 4.0)
                                                + 2.0 * (self.player.jump))
                                                / (2.0 * (-self.player.pitch + y as f32) - (h));
                                            let weight = current_dist / corrected_distance;

                                            let rhs = self.player.pos * (1.0 - weight);
                                            let current_floor_x =
                                                weight * intersection.point[0] + rhs.x;
                                            let current_floor_y =
                                                weight * intersection.point[1] + rhs.y;

                                            let ftx =
                                                (current_floor_x * self.cell_size) as usize % 128;
                                            let fty =
                                                (current_floor_y * self.cell_size) as usize % 128;
                                            /*let lighting = self.lighting.get_lighting_floor(
                                                ftx as f32 / 128.0,
                                                fty as f32 / 128.0,
                                                location,
                                            );*/
                                            self.screen.draw_texture(
                                                slice,
                                                [ftx, (0 * 128) + fty],
                                                y,
                                                0.05,
                                                (3.0 / (current_dist * current_dist)).min(1.5),
                                            )
                                        }
                                    } else if self.player.planedist * (1.0 - (i as f32) / 4.0)
                                        > -2.0 * (self.player.jump)
                                    {
                                        for y in (pos_z + h * 0.5 + rect_h * (4.0 - i as f32) / 8.0)
                                            as usize
                                            ..(self.player.pitch
                                                + self.player.jump
                                                    / ((tw2.distance - delta_distance)
                                                        * self.angles[j].cos())
                                                + h * 0.5
                                                + self.player.planedist
                                                    / ((tw2.distance - delta_distance)
                                                        * self.angles[j].cos())
                                                    * ((4.0 - i as f32) / 8.0))
                                                .min(h)
                                                as usize
                                        {
                                            let current_dist = (self.player.planedist
                                                * (1.0 - (i as f32) / 4.0)
                                                + 2.0 * (self.player.jump))
                                                / (2.0 * (-self.player.pitch + y as f32) - h); // Use a buffer since they're always the same values
                                            let weight = current_dist / corrected_distance;

                                            let rhs = self.player.pos * (1.0 - weight);
                                            let current_floor_x =
                                                weight * intersection.point[0] + rhs.x;
                                            let current_floor_y =
                                                weight * intersection.point[1] + rhs.y;

                                            let ftx =
                                                (current_floor_x * self.cell_size) as usize % 128;
                                            let fty =
                                                (current_floor_y * self.cell_size) as usize % 128;
                                            /*let lighting = self.lighting.get_lighting_floor(
                                                ftx as f32 / 128.0,
                                                fty as f32 / 128.0,
                                                location,
                                            );*/
                                            self.screen.draw_texture(
                                                slice,
                                                [ftx, (0 * 128) + fty],
                                                y,
                                                0.05,
                                                ffmin(3.0 / (current_dist * current_dist),1.5),
                                            )
                                        }
                                    }
                                }
                            }
                            self.draw_wall(
                                slice,
                                h,
                                &tw,
                                h * 0.5
                                    + self.player.planedist / tw_corrected_distance * 7.0 / 16.0,
                                1.0 / 8.0 * self.player.planedist / tw_corrected_distance,
                                &(self.player.pitch + self.player.jump / tw_corrected_distance),
                            );
                        }
                    }
                }
            });
        } else {
            self.sprites.iter().for_each(|sprite| {
                sprite.draw(slice, &self.player, j, &self.screen, corrected_distance)
            });
        }
    }

    fn draw_wall(
        &self,
        slice: &mut [u8],
        h: f32,
        intersection: &Intersection,
        center: f32,
        height: f32,
        pos_z: &f32,
    ) {
        //TODO: REMOVE THIS IF
        let ty_step = {
            if intersection.wall_type == 12 {
                (self.cell_size) / (8.0 * height)
            } else {
                (self.cell_size) / (height)
            }
        };

        let inter_x = intersection.point[0].fract();
        let inter_y = intersection.point[1].fract();

        let rect_top = center - height * 0.5;
        let rect_bottom = center + height * 0.5;

        let rect_bottom_draw = {
            if pos_z + rect_bottom >= h {
                h - pos_z
            } else {
                rect_bottom
            }
        };

        //draw walls
        let mut ty = {
            if rect_top + pos_z <= 0.0 {
                -(pos_z + rect_top) * ty_step
            } else if rect_top + pos_z >= h {
                -(pos_z + rect_bottom) * ty_step
            } else {
                0.0
            }
        };

        let mut tx;
        match intersection.orientation {
            Orientation::N => {
                tx = inter_x * self.cell_size;
                tx = self.cell_size - 1.0 - tx.floor();
            }
            Orientation::E => {
                tx = inter_y * self.cell_size;
            }
            Orientation::S => {
                tx = inter_x * self.cell_size;
            }
            Orientation::W => {
                tx = inter_y * self.cell_size;
                tx = self.cell_size - 1.0 - tx.floor();
            }
        }
        if intersection.wall_type == 6 {
            let offset = 1.0
                - self
                    .map
                    .doors
                    .get(&intersection.map_checkv)
                    .expect("error to draw door")
                    .offset;
            match intersection.orientation {
                Orientation::N => {
                    if inter_x < 0.5 {
                        tx -= offset * 64.0;
                    } else {
                        tx += offset * 64.0;
                    }
                }
                Orientation::E => {
                    if inter_y > 0.5 {
                        tx -= offset * 64.0;
                    } else {
                        tx += offset * 64.0;
                    }
                }
                Orientation::S => {
                    if inter_x > 0.5 {
                        tx -= offset * 64.0;
                    } else {
                        tx += offset * 64.0;
                    }
                }
                Orientation::W => {
                    if inter_y < 0.5 {
                        tx -= offset * 64.0;
                    } else {
                        tx += offset * 64.0;
                    }
                }
            }
        }

        for y in (pos_z + rect_top) as usize..(pos_z + rect_bottom_draw) as usize {
            //TODO: FIX THIS FLOAT POINT ROUNDING ERROR
            if ty >= 128.0 {
                dbg!(
                    height,
                    rect_top,
                    rect_bottom,
                    ty,
                    pos_z + rect_top,
                    pos_z + rect_bottom_draw,
                    self.player.pitch
                );
                ty = 127.0;
            }

            self.screen.draw_texture(
                slice,
                [tx as usize, intersection.wall_type * 128 + ty as usize],
                y,
                self.torch.intensity
                    * self.lighting.get_lighting_wall(
                        tx ,
                        ty * 3.0, //*3.0/128.0
                        intersection.map_checkv,
                        &intersection.orientation,
                    ),
                ffmin(3.0 / (intersection.distance * intersection.distance),1.5),
            );
            ty += ty_step;
        }
    }
}

impl EventHandler for MainState {
    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        match keycode {
            KeyCode::L => self.lighting.switch = !self.lighting.switch,
            KeyCode::K => self.lighting.smooth_switch = !self.lighting.smooth_switch,
            KeyCode::Escape => ggez::event::quit(ctx),
            _ => (),
        }
    }
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let time = timer::time_since_start(ctx).as_secs_f32();
        let dt = ggez::timer::delta(ctx).as_secs_f32();

        self.handle_input(ctx);

        self.player.walk_animation(&self.buffer_walking, time);

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
            if y < (self.player.pitch + h * 0.5) as usize {
                // Calculate ceiling y buffer
                self.buffer_floors[y] = (self.player.planedist - 2.0 * self.player.jump)
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
                self.draw_slice(slice, w as usize / RAYSPERPIXEL - j - 1, h);
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

enum TWandSprites<'a> {
    TW(Intersection),
    Sprites(&'a Sprite),
}

impl TWandSprites<'_> {
    fn distance2(&self) -> f32 {
        match self {
            TWandSprites::TW(tw) => tw.distance * tw.distance,
            TWandSprites::Sprites(sprite) => sprite.distance2,
        }
    }
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