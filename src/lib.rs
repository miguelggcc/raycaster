use std::path::Path;

use ggez::event::{EventHandler, KeyCode};
use ggez::graphics::{self, Color, DrawParam, Mesh, MeshBuilder};
use ggez::input::keyboard::is_key_pressed;
use ggez::{timer, Context, GameResult};
mod map;
mod player;
mod screen;
mod sprite;
mod utilities;
use map::Map;
use num::clamp;
use player::Player;
use rayon::prelude::*;
use screen::Screen;
use sprite::Sprite;
use utilities::input::{mouse_grabbed_and_hidden, set_mouse_location};
use utilities::vector2::Vector2;

use crate::utilities::input::get_delta;

const PI: f32 = std::f32::consts::PI;
const DEG2RAD: f32 = PI / 180.0;
const NUMOFRAYS: usize = 1200;
pub struct MainState {
    background_color: Color,
    player: Player,
    map_size: (usize, usize),
    cell_size: f32,
    map: Map,
    angles: Vec<f32>,
    buffer_floors: Vec<f32>,
    sky: graphics::spritebatch::SpriteBatch,
    sky_sprite: graphics::spritebatch::SpriteIdx,
    intersections: Intersections,
    mesh_line: Option<Mesh>,
    screen: Screen,
    sprites: Vec<Sprite>,
    time: f32,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let background_color = graphics::Color::BLACK;
        let (w, h) = graphics::drawable_size(ctx);

        let pos = Vector2::new(8.5, 12.5);
        let dir_norm = Vector2::new(0.0f32, -1.0); // Player direction
        let plane = Vector2::new((30.0 * DEG2RAD).tan(), 0.0); //Camera plane vector
        let map_size = (16, 25);
        let cell_size = 32.0;
        let map = Map::new(ctx, Path::new("/map.png"), Path::new("/floor.png"))?;

        let player = Player::new(
            ctx,
            pos,
            dir_norm,
            plane,
            (w * 0.5) / ((30.0 * DEG2RAD).tan()), //distance from the player to the projection plane
            0.0,
        )?;

        set_mouse_location(ctx, Vector2::new(w * 0.5, h * 0.5)).unwrap();

        let angles: Vec<f32> = (0..NUMOFRAYS)
            .map(|r: usize| {
                ((r as f32 * w / (NUMOFRAYS as f32 - 1.0) - w * 0.5) / player.planedist).atan()
            })
            .collect();

        let buffer_floors = (0..h as usize)
            .map(|y| player.planedist / (2.0 * y as f32 - h))
            .collect();

        let mut skyimg = graphics::Image::new(ctx, "/sky2.png")?;
        skyimg.set_wrap(graphics::WrapMode::Tile, graphics::WrapMode::Mirror);
        skyimg.set_filter(graphics::FilterMode::Nearest);
        let mut sky = graphics::spritebatch::SpriteBatch::new(skyimg);
        let sky_sprite = sky.add(DrawParam::default());

        let intersections = Intersections::new();

        let mesh_line = None;

        let wall_textures = graphics::Image::new(ctx, "/wall.png")?.to_rgba8(ctx)?;

        let sprite_textures = graphics::Image::new(ctx, "/sprite.png")?.to_rgba8(ctx)?;

        let mut screen = Screen::new(h, w);
        screen.textures(wall_textures, sprite_textures);

        let sprites = vec![
            Sprite::new(sprite::SpriteType::Armor, Vector2::new(7.5, 7.5)),
            Sprite::new(sprite::SpriteType::Armor, Vector2::new(7.5, 9.5)),
            Sprite::new(sprite::SpriteType::CandleHolder, Vector2::new(12.5, 12.5)),
            Sprite::new(sprite::SpriteType::Bat, Vector2::new(6.5, 12.5)),
            Sprite::new(sprite::SpriteType::Torch, Vector2::new(9.0, 14.897)),
            Sprite::new(sprite::SpriteType::Torch, Vector2::new(8.103, 12.0)),
            Sprite::new(sprite::SpriteType::Gore, Vector2::new(13.0, 3.0)),
        ];

        Ok(Self {
            background_color,
            player,
            map_size,
            cell_size,
            map,
            angles,
            buffer_floors,
            sky,
            sky_sprite,
            intersections,
            mesh_line,
            screen,
            sprites,
            time: 0.0,
        })
    }

    pub fn handle_input(&mut self, ctx: &mut Context, ray_dir_norm: Vector2<f32>) {
        let (w, h) = graphics::drawable_size(ctx);
        let dt = ggez::timer::delta(ctx).as_secs_f32();
        mouse_grabbed_and_hidden(ctx, false, true).unwrap();

        let mut delta_mouse_loc_x = get_delta(ctx).x;
        let mut delta_mouse_loc_y = get_delta(ctx).y;

        set_mouse_location(ctx, Vector2::new(w * 0.5, h * 0.5)).unwrap();

        let mut angle_of_rot = 0.0f32;
        if delta_mouse_loc_x != 0.0 {
            delta_mouse_loc_x -= 150.0; // resolution width middle minus window width middle
        }

        if delta_mouse_loc_y != 0.0 {
            delta_mouse_loc_y -= 100.0;
        }
        self.player.pitch -= delta_mouse_loc_y * 0.7;

        self.player.pitch = clamp(self.player.pitch, -300.0, 300.0);

        angle_of_rot += 0.085 * delta_mouse_loc_x;
        self.player.plane = Vector2::rotate(self.player.plane, angle_of_rot * DEG2RAD);
        self.player.dir_norm = Vector2::rotate(self.player.dir_norm, angle_of_rot * DEG2RAD);

        let mut dir = ray_dir_norm * (3.125 * dt);
        let yoffset = 0.3125;

        if is_key_pressed(ctx, KeyCode::W) && self.player.pos.y - 0.16 >= 0.0 {
            let check_pos_y =
                self.player.pos + Vector2::new(0.0, self.player.dir_norm.y.signum() * yoffset);
            let check_pos_x =
                self.player.pos + Vector2::new(self.player.dir_norm.x.signum() * yoffset, 0.0);

            let cell_check_y = self.map.walls
                [(check_pos_y.x) as usize + (check_pos_y.y) as usize * self.map_size.0];
            let cell_check_x = self.map.walls
                [(check_pos_x.x) as usize + (check_pos_x.y) as usize * self.map_size.0];

            if cell_check_y > 0 {
                dir.y = 0.0;
            }
            if cell_check_x > 0 {
                dir.x = 0.0;
            }
            self.player.pos += dir;
        } else if is_key_pressed(ctx, KeyCode::S)
            && self.player.pos.y + 0.16 < self.map_size.1 as f32
        {
            let check_pos_y =
                self.player.pos + Vector2::new(0.0, -self.player.dir_norm.y.signum() * yoffset);
            let check_pos_x =
                self.player.pos + Vector2::new(-self.player.dir_norm.x.signum() * yoffset, 0.0);

            let cell_check_y = self.map.walls
                [(check_pos_y.x) as usize + (check_pos_y.y) as usize * self.map_size.0];
            let cell_check_x = self.map.walls
                [(check_pos_x.x) as usize + (check_pos_x.y) as usize * self.map_size.0];

            if cell_check_y > 0 {
                dir.y = 0.0;
            }
            if cell_check_x > 0 {
                dir.x = 0.0;
            }
            self.player.pos -= dir;
        }

        if is_key_pressed(ctx, KeyCode::A) && self.player.pos.x >= 0.0 {
            let mut perp_dir = Vector2::new(dir.y, -dir.x);
            let check_pos_y =
                self.player.pos + Vector2::new(0.0, -self.player.dir_norm.x.signum() * yoffset);
            let check_pos_x =
                self.player.pos + Vector2::new(self.player.dir_norm.y.signum() * yoffset, 0.0);

            let cell_check_y = self.map.walls
                [(check_pos_y.x) as usize + (check_pos_y.y) as usize * self.map_size.0];
            let cell_check_x = self.map.walls
                [(check_pos_x.x) as usize + (check_pos_x.y) as usize * self.map_size.0];

            if cell_check_y > 0 {
                perp_dir.y = 0.0;
            }
            if cell_check_x > 0 {
                perp_dir.x = 0.0;
            }
            self.player.pos += perp_dir;
        } else if is_key_pressed(ctx, KeyCode::D)
            && self.player.pos.x / self.cell_size < self.map_size.0 as f32
        {
            let mut perp_dir = Vector2::new(-dir.y, dir.x);
            let check_pos_y =
                self.player.pos + Vector2::new(0.0, self.player.dir_norm.x.signum() * yoffset);
            let check_pos_x =
                self.player.pos + Vector2::new(-self.player.dir_norm.y.signum() * yoffset, 0.0);

            let cell_check_y = self.map.walls
                [(check_pos_y.x) as usize + (check_pos_y.y) as usize * self.map_size.0];
            let cell_check_x = self.map.walls
                [(check_pos_x.x) as usize + (check_pos_x.y) as usize * self.map_size.0];

            if cell_check_y > 0 {
                perp_dir.y = 0.0;
            }
            if cell_check_x > 0 {
                perp_dir.x = 0.0;
            }
            self.player.pos += perp_dir;
        }

        if is_key_pressed(ctx, KeyCode::Q) {
            self.player.pitch += 1.0;
        }

        if is_key_pressed(ctx, KeyCode::E) {
            self.player.pitch -= 1.0;
        }
    }

    pub fn calculate_ray(&mut self, ray_dir_player: Vector2<f32>, theta: f32) -> GameResult {
        let ray_dir_norm = Vector2::rotate(ray_dir_player, theta);
        let ray_unitstep_size = Vector2::new(
            (1.0 + (ray_dir_norm.y / ray_dir_norm.x) * (ray_dir_norm.y / ray_dir_norm.x)).sqrt(),
            (1.0 + (ray_dir_norm.x / ray_dir_norm.y) * (ray_dir_norm.x / ray_dir_norm.y)).sqrt(),
        );
        let startv = self.player.pos;

        let mut map_checkv = Vector2::new(startv.x.floor(), startv.y.floor());
        let mut ray_length1_d = Vector2::new(0.0f32, 0.0);
        let mut orientation;
        let mut stepv = Vector2::new(0.0f32, 0.0);

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
                let mut wall_type =
                    self.map.walls[(map_checkv.y * self.map_size.0 as f32 + map_checkv.x) as usize];

                if wall_type == 6 {
                    //door
                    tilefound = true;
                    if orientation == Orientation::N || orientation == Orientation::S {
                        if ray_length1_d.y - 0.5 * ray_unitstep_size.y < ray_length1_d.x {
                            ray_length1_d.y -= ray_unitstep_size.y * 0.5;
                            distance = ray_length1_d.y;
                        } else {
                            orientation = Orientation::W;
                            wall_type = 7;
                            distance = ray_length1_d.x;
                        }
                    } else if orientation == Orientation::E || orientation == Orientation::W {
                        if ray_length1_d.x - 0.5 * ray_unitstep_size.x < ray_length1_d.y {
                            ray_length1_d.x -= ray_unitstep_size.x * 0.5;
                            distance = ray_length1_d.x;
                        } else {
                            orientation = Orientation::N;
                            wall_type = 7;
                            distance = ray_length1_d.y;
                        }
                    }
                } else if wall_type > 0 {
                    tilefound = true;
                }

                let intersection = startv + ray_dir_norm * distance;

                if tilefound {
                    self.intersections.points.push(intersection.to_array());
                    self.intersections.distance_fisheye.push(distance);
                    distance *= (theta).cos();

                    self.intersections.distances.push(distance);
                    self.intersections.wall_type.push(wall_type as usize);
                    self.intersections.orientation.push(orientation);
                    self.intersections.line_points.push([0.0, 0.0]);
                    self.intersections
                        .line_points
                        .push(((intersection - self.player.pos) * 16.0).to_array());
                }
            }
        }
        Ok(())
    }

    fn draw_slice(&self, slice: &mut [u8], j: usize, w: f32, h: f32) {
        let rect_w = w as usize / NUMOFRAYS;
        let rect_h = self.player.planedist / (self.intersections.distances[j]);
        let rect_top = (h - rect_h) * 0.5;
        let rect_bottom = (h + rect_h) * 0.5;
        let ty_step = self.cell_size / rect_h;
        let posint = self.intersections.points[j];

        let xint = posint[0] - posint[0].floor();
        let yint = posint[1] - posint[1].floor();

        let wall_type = self.intersections.wall_type[j];

        //draw walls
        let mut ty = {
            if rect_bottom - self.player.pitch > h {
                (-self.player.pitch - rect_top) * ty_step
            } else if rect_top + self.player.pitch < 0.0 {
                (-self.player.pitch - rect_bottom) * ty_step
            } else {
                0.0f32
            }
        };
        let mut tx;
        let shade;
        match self.intersections.orientation[j] {
            Orientation::N => {
                tx = xint * self.cell_size;
                tx = self.cell_size - 1.0 - tx.floor();
                shade = 1.0;
            }
            Orientation::E => {
                tx = yint * self.cell_size;
                shade = 0.7
            }
            Orientation::S => {
                tx = xint * self.cell_size;
                shade = 1.0;
            }
            Orientation::W => {
                tx = yint * self.cell_size;
                tx = self.cell_size - 1.0 - tx.floor();
                shade = 0.7;
            }
        }
        let mut rect_bottom_draw = rect_bottom;
        if self.player.pitch + rect_bottom - 1.0 > h {
            rect_bottom_draw = h - self.player.pitch - 1.0;
        }

        for y in (self.player.pitch + rect_top) as usize
            ..(self.player.pitch + rect_bottom_draw - 1.0) as usize
        {
            self.screen.draw_texture(
                slice,
                [tx as usize, wall_type * 32 + ty as usize],
                [0, y],
                rect_w,
                num::clamp(
                    shade * 3.9 / (self.intersections.distance_fisheye[j]),
                    0.2,
                    1.5,
                ),
                32,
            );
            ty += ty_step;
        }

        //draw floor

        for y in (self.player.pitch + rect_bottom) as usize..(h) as usize {
            let current_dist = self.player.planedist / (2.0 * (y as f32 - self.player.pitch) - h);
            //let current_dist = self.buffer_floors[y+self.player.pitch as usize]; // Use buffer since they're always the same values
            let weight = current_dist / (self.intersections.distances[j]);

            let rhs = (self.player.pos * (1.0 - weight)).to_array();
            let current_floor = [
                weight * self.intersections.points[j][0] + rhs[0],
                weight * self.intersections.points[j][1] + rhs[1],
            ];

            let floor_type = self.map.floors
                [current_floor[0] as usize + current_floor[1] as usize * self.map_size.0];

            let ftx = (current_floor[0] * self.cell_size) as usize % 32;
            let fty = (current_floor[1] * self.cell_size) as usize % 32;

            self.screen.draw_texture(
                slice,
                [ftx, (floor_type << 5) as usize + fty],
                [0, y],
                rect_w,
                num::clamp(3.5 / (current_dist), 0.2, 1.2),
                32,
            );
            //draw ceiling
            /*                   self.screen.draw_texture(
                                    slice,
                &self.textures,
                [ftx, fty],
                [0,  h as usize - y],
                rect_w as usize,
                num::clamp(3.5 / (current_dist), 0.2, 1.2),
                32,
            ); */
        }
        let mut rect_top_draw = rect_top;
        if rect_top + self.player.pitch > h {
            rect_top_draw = h - self.player.pitch;
        }
        for y in 0..(rect_top_draw + self.player.pitch) as usize {
            for k in 0..rect_w {
                self.screen.draw_pixel(slice, k, y as usize, &[0, 0, 0, 0]);
            }
        }
    }
}
impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.handle_input(ctx, self.player.dir_norm);
        let ray_dir_norm = self.player.dir_norm;

        self.intersections.clear();

        for angle in self.angles.clone() {
            self.calculate_ray(ray_dir_norm, angle)?;
        }
        self.time += timer::delta(ctx).as_secs_f32();
        self.sprites
            .iter_mut()
            .for_each(|sprite| sprite.update(self.time));

        if self.intersections.points.len() > 2 {
            self.mesh_line = Some({
                MeshBuilder::new()
                    .line(&self.intersections.line_points, 1.0, graphics::Color::WHITE)?
                    .build(ctx)?
            });
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let (w, h) = graphics::drawable_size(ctx);
        let rect_w = w as usize / NUMOFRAYS;
        graphics::clear(ctx, self.background_color);
        let mut corr_angle = self.player.dir_norm.angle();
        if corr_angle < 0.0 {
            corr_angle += 2.0 * PI;
        }
        let draw_param = graphics::DrawParam {
            src: graphics::Rect::new(
                6.0 * corr_angle / (2.0 * PI),
                0.4 - self.player.pitch / 864.0,
                1.0,
                1.0,
            ),
            ..Default::default()
        };
        self.sky.set(self.sky_sprite, draw_param)?;
        graphics::draw(ctx, &self.sky, draw_param)?;

        let mut img_arr = std::mem::take(&mut self.screen.img_arr);

        img_arr
            .chunks_mut(h as usize * 4)
            .enumerate()
            .for_each(|(j, slice)| self.draw_slice(slice, w as usize - 1 - j, w, h));

        self.screen.img_arr = img_arr;

        self.sprites.sort_by(|a: &Sprite, b: &Sprite| {
            b.calculate_distance_2(&self.player)
                .partial_cmp(&a.calculate_distance_2(&self.player))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.sprites.iter_mut().for_each(|sprite| {
            sprite.draw(
                ctx,
                &self.player,
                &mut self.screen,
                &self.intersections.distances,
                rect_w,
            )
        });
        let img = self.screen.arr_to_rgba(ctx)?;
        graphics::draw(
            ctx,
            &img,
            DrawParam::default()
                .offset([0.5, 0.5])
                .rotation(PI * 0.5)
                .dest([w * 0.5, h * 0.5]),
        )?;

        draw_fps_counter(ctx)?;

        self.map.draw_map(ctx, self.map_size, &self.player)?;
        if let Some(mesh_line) = &self.mesh_line {
            graphics::draw(
                ctx,
                mesh_line,
                DrawParam::default().dest([8.0 * 16.0, h - 8.0 * 16.0]),
            )?;
        }

        self.player.draw(ctx)?;

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

pub struct Intersections {
    points: Vec<[f32; 2]>,
    distances: Vec<f32>,
    distance_fisheye: Vec<f32>,
    orientation: Vec<Orientation>,
    wall_type: Vec<usize>,
    line_points: Vec<[f32; 2]>,
}

impl Default for Intersections {
    fn default() -> Self {
        Self::new()
    }
}

impl Intersections {
    pub fn new() -> Self {
        Self {
            points: Vec::with_capacity(NUMOFRAYS),
            distances: Vec::with_capacity(NUMOFRAYS),
            distance_fisheye: Vec::with_capacity(NUMOFRAYS),
            orientation: Vec::with_capacity(NUMOFRAYS),
            wall_type: Vec::with_capacity(NUMOFRAYS),
            line_points: Vec::with_capacity(NUMOFRAYS),
        }
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.distances.clear();
        self.distance_fisheye.clear();
        self.orientation.clear();
        self.wall_type.clear();
        self.line_points.clear();
    }
}
#[derive(PartialEq)]
pub enum Orientation {
    N = 1,
    E = 2,
    S = 3,
    W = 4,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
