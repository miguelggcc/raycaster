use ggez::{
    graphics::{self, Color, DrawMode, DrawParam, Mesh, MeshBuilder},
    Context, GameResult,
};

use crate::utilities::vector2::Vector2;

pub struct Player {
    pub pos: Vector2<f32>,
    pub dir_norm: Vector2<f32>,
    pub plane: Vector2<f32>,
    pub planedist: f32,
    pub pitch: f32,
    pub jump: f32,
    pub walking: bool,
    pub height: f32,
    pub mesh: Mesh,
}

impl Player {
    pub fn new(
        ctx: &mut Context,
        pos: Vector2<f32>,
        dir_norm: Vector2<f32>,
        plane: Vector2<f32>,
        planedist: f32,
        pitch: f32,
        jump: f32,
    ) -> GameResult<Self> {
        let (_w, h) = graphics::drawable_size(ctx);
        let mesh = MeshBuilder::new()
            .circle(
                DrawMode::fill(),
                [11.0 * 16.0, h - 8.0 * 16.0],
                4.0,
                0.1,
                Color::new(145.0 / 255.0, 25.0 / 255.0, 16.0 / 255.0, 1.0),
            )?
            .build(ctx)?;
        Ok(Self {
            pos,
            dir_norm,
            plane,
            planedist,
            pitch,
            mesh,
            jump,
            height: 0.0,
            walking: false,
        })
    }

    pub fn draw_circle(&self, ctx: &mut Context) -> GameResult<()> {
        graphics::draw(ctx, &self.mesh, DrawParam::default())?;
        Ok(())
    }

    pub fn walk_animation(&mut self, buffer_walking: &[f32], time: f32) {
        self.jump = self.height;
        if self.walking {
            let delta_jump = buffer_walking[(time % 0.5 * 300.0) as usize];
            self.jump += delta_jump * 25.0;
        }
    }
}
