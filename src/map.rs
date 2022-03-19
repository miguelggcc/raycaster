use std::path::Path;

use ggez::{graphics, Context, GameResult};

pub struct Map {
    pub walls: Vec<u32>,
    pub floors: Vec<u32>,
}

impl Map {
    pub fn new(ctx: &mut Context, path_walls: &Path, path_floors: &Path) -> GameResult<Self> {
        Ok(Self {
            walls: read_map_walls(ctx, path_walls)?,
            floors: read_map_floors(ctx, path_floors)?,
        })
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
