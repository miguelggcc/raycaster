use std::{collections::HashMap, path::Path};

use ggez::{
    graphics::{self, DrawParam, Image, Rect},
    Context, GameResult,
};

use crate::player::Player;

pub struct Map {
    pub walls: Vec<usize>,
    pub floors: Vec<usize>,
    pub solid: Vec<bool>,
    pub doors: HashMap<usize, Door>,
    pub minimap: Image,
    pub sb: graphics::spritebatch::SpriteBatch,
}

impl Map {
    pub fn new(
        ctx: &mut Context,
        path_walls: &Path,
        path_floors: &Path,
        minimap: Image,
        sb: graphics::spritebatch::SpriteBatch,
        map_size: (usize, usize),
    ) -> GameResult<Self> {
        let mut solid = vec![true; map_size.0 * map_size.1];
        let mut doors = HashMap::new();
        Ok(Self {
            walls: read_map_walls(ctx, path_walls, &mut solid, &mut doors)?,
            floors: read_map_floors(ctx, path_floors)?,
            solid,
            doors,
            minimap,
            sb,
        })
    }
    pub fn draw_minimap(
        &mut self,
        ctx: &mut Context,
        map_size: (usize, usize),
        player: &Player,
    ) -> GameResult {
        let (_w, h) = graphics::drawable_size(ctx);
        graphics::draw(ctx, &self.minimap, DrawParam::default().dest([0.0, 525.0]))?;
        let mut left = player.pos.x - 8.0;
        if left < 0.0 {
            left = 0.0;
        }
        let mut right = player.pos.x + 8.0;
        if right > map_size.0 as f32 {
            right = map_size.0 as f32;
        }
        let mut top = player.pos.y - 8.0;
        if top < 0.0 {
            top = 0.0;
        }
        let mut bottom = player.pos.y + 8.0;
        if bottom > map_size.1 as f32 {
            bottom = map_size.1 as f32;
        }

        let mut sprite_offset;
        for i in left as usize..(right).ceil() as usize {
            for j in top as usize..bottom.floor() as usize {
                if self.walls[i + map_size.0 * j] > 0 {
                    sprite_offset = 0.0;
                } else {
                    sprite_offset = 0.5;
                }

                if i == left as usize && j == top as usize {
                    self.sb.add(get_drawparam(
                        player,
                        left,
                        top,
                        left % 1.0 * 0.5 + sprite_offset,
                        top % 1.0,
                        0.5 * (1.0 - left % 1.0),
                        1.0 - top % 1.0,
                    ));
                } else if i == right as usize && j == top as usize {
                    self.sb.add(get_drawparam(
                        player,
                        (right).floor(),
                        top,
                        sprite_offset,
                        top % 1.0,
                        0.5 * (right % 1.0),
                        1.0 - top % 1.0,
                    ));
                } else if i == left as usize {
                    self.sb.add(get_drawparam(
                        player,
                        left,
                        j as f32,
                        left % 1.0 * 0.5 + sprite_offset,
                        0.0,
                        0.5 * (1.0 - left % 1.0),
                        1.0,
                    ));
                } else if i == (right) as usize {
                    self.sb.add(get_drawparam(
                        player,
                        (right).floor(),
                        j as f32,
                        sprite_offset,
                        0.0,
                        0.5 * (right % 1.0),
                        1.0,
                    ));
                } else if j == top as usize {
                    self.sb.add(get_drawparam(
                        player,
                        i as f32,
                        top,
                        sprite_offset,
                        top % 1.0,
                        0.5,
                        1.0 - top % 1.0,
                    ));
                } else {
                    self.sb.add(get_drawparam(
                        player,
                        i as f32,
                        j as f32,
                        sprite_offset,
                        0.0,
                        0.5,
                        1.0,
                    ));
                }
            }
        }
        graphics::draw(ctx, &self.sb, DrawParam::new().dest([0.0, h - 16.0 * 16.0]))?;
        self.sb.clear();

        Ok(())
    }
}
pub fn read_map_walls(
    ctx: &mut Context,
    path: &Path,
    can_pass: &mut Vec<bool>,
    door_offset: &mut HashMap<usize, Door>,
) -> GameResult<Vec<usize>> {
    let map = graphics::Image::new(ctx, path)?.to_rgba8(ctx)?;
    let walls: Vec<usize> = map
        .chunks(4)
        .enumerate()
        .map(|(i, color)| match color {
            [255, 255, 0, 255] => 1,
            [0, 0, 0, 255] => 2,
            [0, 0, 255, 255] => 3,
            [255, 0, 0, 255] => 4,
            [0, 255, 0, 255] => 5,
            [255, 0, 255, 255] => {
                let door = Door::new(1.0, false, 0.0, i);
                door_offset.insert(i, door);
                6
            }
            _ => {
                can_pass[i] = false;
                0
            }
        })
        .collect();

    Ok(walls)
}

pub fn read_map_floors(ctx: &mut Context, path: &Path) -> GameResult<Vec<usize>> {
    let fmap = graphics::Image::new(ctx, path)?.to_rgba8(ctx)?;
    let floors: Vec<usize> = fmap
        .into_iter()
        .step_by(4)
        .map(|r| if r == 0 { 1 } else { 0 })
        .collect();
    Ok(floors)
}

pub struct Door {
    pub offset: f32,
    pub opening: bool,
    pub timer: f32,
    pub pos: usize,
}

impl Door {
    pub fn new(offset: f32, opening: bool, timer: f32, pos: usize) -> Self {
        Self {
            offset,
            opening,
            timer,
            pos,
        }
    }

    pub fn update(&mut self, time: f32, trigger_time: f32, solid: &mut Vec<bool>) {
        if time - self.timer > trigger_time {
            self.timer = 0.0;
            if self.offset > 0.001 {
                self.offset -= 0.01;
            } else {
                self.opening = false;
                solid[self.pos] = false;
            }
        }
    }
}

fn get_drawparam(
    player: &Player,
    x_offset: f32,
    y_offset: f32,
    x_start: f32,
    y_start: f32,
    width: f32,
    height: f32,
) -> DrawParam {
    DrawParam::default()
        .dest([
            16.0 * (10.0 - player.pos.x + x_offset),
            16.0 * (9.0 - player.pos.y + y_offset),
        ])
        .src(Rect::new(x_start, y_start, width, height))
}
