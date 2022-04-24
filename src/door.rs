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

    pub fn update(&mut self, dt: f32, solid: &mut Vec<bool>) {
            if self.offset > 0.001 {
                self.offset -= 0.5*dt;
            } else {
                self.opening = false;
                solid[self.pos] = false;
            }
    }
}
