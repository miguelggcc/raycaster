use ggez::{
    graphics::{self, DrawParam, Mesh},
    Context, GameResult,
};

use crate::utilities::vector2::Vector2;

pub struct Player {
    pub pos: Vector2<f32>,
    pub dir_norm: Vector2<f32>,
    pub plane: Vector2<f32>,
    pub planedist: f32,
    pub pitch: f32,
    pub mesh: Mesh,
}

impl Player {
    pub fn new(
        pos: Vector2<f32>,
        dir_norm: Vector2<f32>,
        plane: Vector2<f32>,
        planedist: f32,
        pitch: f32,
        mesh: Mesh,
    ) -> Self {
        Self {
            pos,
            dir_norm,
            plane,
            planedist,
            pitch,
            mesh,
        }
    }

    pub fn draw(&self, ctx: &mut Context, scale: f32) -> GameResult<()> {
        graphics::draw(
            ctx,
            &self.mesh,
            DrawParam::default().dest((self.pos * scale).to_array()),
        )?;
        Ok(())
    }
}
