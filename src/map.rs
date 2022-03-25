use std::path::Path;

use ggez::{
    graphics::{self, Color, DrawParam, Mesh, MeshBuilder, Rect},
    Context, GameResult,
};

use crate::player::Player;

pub struct Map {
    pub walls: Vec<u32>,
    pub floors: Vec<u32>,
    pub mesh1: Mesh,
    pub mesh2: Mesh,
}

impl Map {
    pub fn new(ctx: &mut Context, path_walls: &Path, path_floors: &Path) -> GameResult<Self> {
        let (_w, h) = graphics::drawable_size(ctx);
        Ok(Self {
            walls: read_map_walls(ctx, path_walls)?,
            floors: read_map_floors(ctx, path_floors)?,
            mesh1: MeshBuilder::new()
                .rectangle(
                    graphics::DrawMode::stroke(1.0),
                    Rect::new(0.0, h - 16.0 * 16.0, 16.0, 16.0),
                    Color::WHITE,
                )?
                .build(ctx)?,

            mesh2: MeshBuilder::new()
                .rectangle(
                    graphics::DrawMode::fill(),
                    Rect::new(0.0, h - 16.0 * 16.0, 16.0, 16.0),
                    Color::BLUE,
                )?
                .build(ctx)?,
        })
    }
    pub fn draw_map(
        &self,
        ctx: &mut Context,
        map_size: (usize, usize),
        player: &Player,
    ) -> GameResult {
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

        for i in left as usize..right as usize {
            for j in top as usize..bottom as usize {
                if self.walls[i + map_size.0 * j] > 0 {
                    graphics::draw(
                        ctx,
                        &self.mesh2,
                        DrawParam::default().dest([
                            16.0 * (8.0 - player.pos.x + i as f32),
                            16.0 * (8.0 - player.pos.y + j as f32),
                        ]),
                    )?;
                } else {
                    graphics::draw(
                        ctx,
                        &self.mesh1,
                        DrawParam::default().dest([
                            16.0 * (8.0 - player.pos.x + i as f32),
                            16.0 * (8.0 - player.pos.y + j as f32),
                        ]),
                    )?;
                }
            }
        }

        Ok(())
    }
}
pub fn read_map_walls(ctx: &mut Context, path: &Path) -> GameResult<Vec<u32>> {
    let map = graphics::Image::new(ctx, path)?.to_rgba8(ctx)?;
    let walls: Vec<u32> = map
        .chunks(4)
        .map(|r| match r {
            [255, 255, 0, 255] => 1,
            [0, 0, 0, 255] => 2,
            [0, 0, 255, 255] => 3,
            [255, 0, 0, 255] => 4,
            [0, 255, 0, 255] => 5,
            [255, 0, 255, 255] => 6,
            _ => 0,
        })
        .collect();

    Ok(walls)
}

pub fn read_map_floors(ctx: &mut Context, path: &Path) -> GameResult<Vec<u32>> {
    let fmap = graphics::Image::new(ctx, path)?.to_rgba8(ctx)?;
    let floors: Vec<u32> = fmap
        .into_iter()
        .step_by(4)
        .map(|r| if r == 0 { 1 } else { 0 })
        .collect();
    Ok(floors)
}
