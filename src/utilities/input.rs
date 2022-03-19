use ggez::{input, Context, GameResult};

use super::vector2::Vector2;
#[allow(dead_code)]
pub fn mouse_location(context: &mut Context) -> Vector2<f32> {
    let mouse_location = input::mouse::position(context);
    Vector2::<f32>::new(mouse_location.x, mouse_location.y)
}

#[allow(dead_code)]
pub fn mouse_pressed(ctx: &mut Context) -> bool {
    input::mouse::button_pressed(ctx, input::mouse::MouseButton::Left)
}

#[allow(dead_code)]
pub fn mouse_grabbed_and_hidden(ctx: &mut Context, grabbed: bool, hidden: bool) -> GameResult {
    input::mouse::set_cursor_hidden(ctx, hidden);
    input::mouse::set_cursor_grabbed(ctx, grabbed)?;

    Ok(())
}

#[allow(dead_code)]
pub fn set_mouse_location(context: &mut Context, new_position: Vector2<f32>) -> GameResult {
    input::mouse::set_position(context, new_position.to_array())?;
    Ok(())
}

#[allow(dead_code)]
pub fn get_delta(ctx: &mut Context) -> Vector2<f32> {
    let delta = input::mouse::delta(ctx);
    Vector2::<f32>::new(delta.x, delta.y)
}
