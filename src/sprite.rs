use ggez::{
    graphics::{self},
    Context,
};

use crate::{player::Player, screen::Screen, utilities::vector2::Vector2};
const PI: f32 = std::f32::consts::PI;
const TEX_SIZE: usize = 128;
pub struct Sprite {
    pub stype: usize,
    pub pos: Vector2<f32>,
    pub height: f32,
    pub visible: bool,
    time: f32,
    pub bounds: Bounds,
    pub distance2: f32,
    shade: f32,
}

impl Sprite {
    pub fn new(stype: SpriteType, pos: Vector2<f32>, height: f32) -> Self {
        Self {
            stype: stype as usize,
            pos,
            height,
            visible: false,
            time: 0.0,
            bounds: Bounds::default(),
            distance2: 0.0,
            shade: 0.0,
        }
    }
    pub fn calculate_distance_2(&self, player: &Player) -> f32 {
        (player.pos.x - self.pos.x) * (player.pos.x - self.pos.x)
            + (player.pos.y - self.pos.y) * (player.pos.y - self.pos.y) //square root not necessary
    }

    pub fn update(&mut self, time: f32) {
        self.time = time;
    }

    pub fn set_drawing_bounds(&mut self, ctx: &mut Context, player: &Player, rays_per_pixel: f32) {
        let (w, h) = graphics::drawable_size(ctx);
        let sprite_delpos = self.pos - player.pos;
        let inv_det =
            1.0 / (player.plane.x * player.dir_norm.y - player.dir_norm.x * player.plane.y);
        self.distance2 = self.calculate_distance_2(player);
        let transform_x =
            inv_det * (player.dir_norm.y * sprite_delpos.x - player.dir_norm.x * sprite_delpos.y);
        let transform_y =
            inv_det * (-player.plane.y * sprite_delpos.x + player.plane.x * sprite_delpos.y);
        let sprite_screen_x = (w / rays_per_pixel * 0.5) * (1.0 + transform_x / transform_y);
        let sprite_size = (player.planedist / transform_y).abs() / rays_per_pixel;
        let sprite_size_y = sprite_size * rays_per_pixel;
        let mut start_y = -sprite_size_y * 0.5
            + h * 0.5
            + player.pitch
            + (player.jump + self.height) / transform_y;
        if start_y < 0.0 {
            start_y = 0.0;
        }
        let mut end_y = sprite_size_y * 0.5
            + h * 0.5
            + player.pitch
            + (player.jump + self.height) / transform_y;
        if end_y > h - 1.0 {
            end_y = h - 1.0;
        }

        let mut start_x = -sprite_size * 0.5 + sprite_screen_x;
        if start_x < 0.0 {
            start_x = 0.0;
        }
        let mut end_x = sprite_size * 0.5 + sprite_screen_x;
        if end_x > w - 1.0 {
            end_x = w - 1.0;
        }

        let mut sty = Vec::new();

        if transform_y > 0.0 && start_x < w && end_x > 0.0 && end_y > 0.0 && start_y < h {
            self.visible = true;

            let denominator = TEX_SIZE as f32 / sprite_size_y;
            sty = (start_y as usize..1 + end_y as usize)
                .map(|y| {
                    //for every pixel of the current stripe
                    let d = (y as f32) - h * 0.5 + sprite_size_y * 0.5
                        - player.pitch
                        - (player.jump + self.height) / transform_y;

                    (d * denominator) as usize
                })
                .collect();

            self.shade = {
                if self.stype == SpriteType::Torch as usize {
                    1.0
                } else {
                    num::clamp(5.0 / self.distance2, 0.2, 1.0)
                }
            };
        } else {
            self.visible = false;
        }

        self.bounds = Bounds::new(
            start_y,
            end_y,
            start_x,
            end_x,
            sprite_screen_x,
            sty,
            sprite_size,
        );
    }

    pub fn draw(
        &self,
        slice: &mut [u8],
        player: &Player,
        j: usize,
        screen: &Screen,
        distance: f32,
    ) {
        let stripe = j as f32;

        if self.visible && stripe >= self.bounds.start_x && stripe < self.bounds.end_x {
            let sprite_delpos = self.pos - player.pos;
            let mut angle = (sprite_delpos.y).atan2(sprite_delpos.x);
            let angle_player = player.dir_norm.angle();
            let delta_angle = angle - angle_player;
            let cos = delta_angle.cos();
            let mut sprite_rotation = 0;
            if self.stype == SpriteType::Bat as usize {
                if angle < 0.0 {
                    angle += 2.0 * PI;
                }
                sprite_rotation = (angle / (2.0 * PI) * 8.0).round() as usize;

                sprite_rotation -= 7;
            } else if self.stype == SpriteType::Torch as usize {
                sprite_rotation = (self.time * 1.1 % 1.0 * 8.0) as usize;
            }
            if sprite_rotation > 7 {
                sprite_rotation = 0;
            }

            let stx = ((stripe - (-self.bounds.size as f32 * 0.5 + self.bounds.sprite_screen_x))
                * TEX_SIZE as f32
                / self.bounds.size) as usize;
            if (distance * distance) / (cos * cos) > self.distance2 {
                for y in self.bounds.start_y as usize..1 + self.bounds.end_y as usize {
                    screen.draw_sprite(
                        slice,
                        [
                            sprite_rotation * TEX_SIZE + stx,
                            self.stype * TEX_SIZE
                                + self.bounds.sty[y - self.bounds.start_y as usize],
                        ],
                        y,
                        self.shade,
                    );
                }
            }
        }
    }
}

pub struct Bounds {
    start_y: f32,
    end_y: f32,
    start_x: f32,
    end_x: f32,
    sprite_screen_x: f32,
    sty: Vec<usize>,
    size: f32,
}

impl Bounds {
    pub fn new(
        start_y: f32,
        end_y: f32,
        start_x: f32,
        end_x: f32,
        sprite_screen_x: f32,
        sty: Vec<usize>,
        size: f32,
    ) -> Self {
        Self {
            start_y,
            end_y,
            start_x,
            end_x,
            sprite_screen_x,
            sty,
            size,
        }
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, 0.0, vec![], 0.0)
    }
}
#[allow(dead_code)]
pub enum SpriteType {
    Armor = 0,
    CandleHolder = 1,
    Bat = 2,
    Torch = 3,
    Gore = 4,
}
