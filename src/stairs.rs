pub struct Stairs {
    pub height: bool,
    pub hit1: f32,
    pub hit2: f32,
    pub hit3: f32,
}

impl Door {
    pub fn new(offset: f32, opening: bool, timer: f32, pos: usize) -> Self {
        Self {
            height,
            opening,
            timer,
            pos,
        }
    }

    pub fn update(&mut self, dt: f32, solid: &mut Vec<bool>) {
        if self.offset > 0.001 {
            self.offset -= 0.5 * dt;
        } else {
            self.opening = false;
            solid[self.pos] = false;
        }
    }
}
