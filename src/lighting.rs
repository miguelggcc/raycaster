use rand::Rng;

use crate::Orientation;

use std::arch::x86_64::*;
use std::collections::VecDeque;

pub struct Lighting {
    vertices: Vec<Vertex>,
    lighting: Vec<f32>,
    map_size: (usize, usize),
    pub switch: bool,
    pub smooth_switch: bool,
}

impl Lighting {
    pub fn new(torches_pos: Vec<usize>, map: &[bool], map_size: (usize, usize)) -> Self {
        let light_int = calculate_lighting(torches_pos, map, map_size);
        let lighting: Vec<f32> = light_int
            .iter()
            .map(|l| 0.7f32.powf(0.8 * (15 - l) as f32) / (127.0 * 127.0))
            .collect();
        let mut all_vertices = vec![];
        for k in 0..2 {
            for j in 0..map_size.1 + 1 {
                for i in 0..map_size.0 + 1 {
                    all_vertices.push(Vertex::new([i, j, k], map_size, &lighting));
                }
            }
        }

        let mut vertices = vec![Vertex::default(); map_size.0 * map_size.1 * 4 * 2];

        vertices.chunks_mut(4).enumerate().for_each(|(pos, chunk)| {
            let z = pos / (map_size.0 * map_size.1);
            let offset = (pos / map_size.0) + z * (map_size.0 + 1);
            let tl = all_vertices[pos + offset];
            let tr = all_vertices[pos + 1 + offset];
            let bl = all_vertices[pos + map_size.0 + 1 + offset];
            let br = all_vertices[pos + map_size.0 + 2 + offset];
            chunk.copy_from_slice(&[tl, tr, bl, br]);
        });

        Self {
            vertices,
            lighting,
            map_size,
            switch: true,
            smooth_switch: true,
        }
    }
    #[inline(always)]
    pub fn get_lighting_floor(&self, x: f32, y: f32, pos: usize) -> f32 {
        if self.switch {
            let (tl, tr, bl, br) = get_vertices(pos, &self.vertices);
            if self.smooth_switch {
                unsafe {
                    bilerp(
                        x,
                        127.0 - y,
                        bl.lighting,
                        br.lighting,
                        tl.lighting,
                        tr.lighting,
                    )
                }
            } else {
                127.0 * 127.0 * self.lighting[pos]
            }
        } else {
            1.0
        }
    }

    #[inline(always)]
    pub unsafe fn get_lighting_wall(
        &self,
        x: f32,
        y: f32,
        pos: usize,
        orientation: &Orientation,
        is_up: bool,
    ) -> f32 {
        if self.switch {
            if self.smooth_switch {
                if !is_up {
                    match orientation {
                        Orientation::N => {
                            let (tl, tr, bl, br) =
                                get_vertices(pos - self.map_size.0, &self.vertices);

                            if y > 256.0 {
                                bilerp(
                                    127.0 - x,
                                    384.0 - y,
                                    bl.lighting,
                                    br.lighting,
                                    tl.lighting,
                                    tr.lighting,
                                )
                            } else {
                                lerp(127.0 - x, tl.lighting, tr.lighting)
                            }
                        }
                        Orientation::S => {
                            let (tl, tr, bl, br) =
                                get_vertices(pos + self.map_size.0, &self.vertices);
                            if y > 256.0 {
                                bilerp(
                                    x,
                                    384.0 - y,
                                    tl.lighting,
                                    tr.lighting,
                                    bl.lighting,
                                    br.lighting,
                                )
                            } else {
                                lerp(x, bl.lighting, br.lighting)
                            }
                        }
                        Orientation::E => {
                            let (tl, tr, bl, br) = get_vertices(pos - 1, &self.vertices);
                            if y > 256.0 {
                                bilerp(
                                    x,
                                    384.0 - y,
                                    tr.lighting,
                                    br.lighting,
                                    tl.lighting,
                                    bl.lighting,
                                )
                            } else {
                                lerp(x, tl.lighting, bl.lighting)
                            }
                        }
                        Orientation::W => {
                            let (tl, tr, bl, br) = get_vertices(pos + 1, &self.vertices);
                            if y > 256.0 {
                                bilerp(
                                    x,
                                    384.0 - y,
                                    bl.lighting,
                                    tl.lighting,
                                    br.lighting,
                                    tr.lighting,
                                )
                            } else {
                                lerp(x, br.lighting, tr.lighting)
                            }
                        }
                    }
                } else {
                    match orientation {
                        Orientation::N => {
                            let (tl, tr, bl, br) =
                                get_vertices(pos - self.map_size.0, &self.vertices);
                            let (btl, btr, _, _) = get_vertices(
                                pos - self.map_size.0 - self.map_size.0 * self.map_size.1,
                                &self.vertices,
                            );
                            if y > 256.0 {
                                bilerp(
                                    127.0 - x,
                                    384.0 - y,
                                    btl.lighting,
                                    btr.lighting,
                                    tl.lighting,
                                    tr.lighting,
                                )
                            } else if y > 127.0 {
                                lerp(127.0 - x, tl.lighting, tr.lighting)
                            } else {
                                bilerp(
                                    127.0 - x,
                                    127.0 - y,
                                    tl.lighting,
                                    tr.lighting,
                                    bl.lighting,
                                    br.lighting,
                                )
                            }
                        }
                        Orientation::S => {
                            let (tl, tr, bl, br) =
                                get_vertices(pos + self.map_size.0, &self.vertices);
                            let (_, _, bbl, bbr) = get_vertices(
                                pos + self.map_size.0 - self.map_size.0 * self.map_size.1,
                                &self.vertices,
                            );

                            if y > 256.0 {
                                bilerp(
                                    x,
                                    384.0 - y,
                                    bbl.lighting,
                                    bbr.lighting,
                                    bl.lighting,
                                    br.lighting,
                                )
                            } else if y > 127.0 {
                                lerp(x, bl.lighting, br.lighting)
                            } else {
                                bilerp(
                                    x,
                                    127.0 - y,
                                    bl.lighting,
                                    br.lighting,
                                    tl.lighting,
                                    tr.lighting,
                                )
                            }
                        }
                        Orientation::E => {
                            let (tl, tr, bl, br) = get_vertices(pos - 1, &self.vertices);
                            let (btl, _, bbl, _) = get_vertices(
                                pos - 1 - self.map_size.0 * self.map_size.1,
                                &self.vertices,
                            );

                            if y > 256.0 {
                                bilerp(
                                    x,
                                    384.0 - y,
                                    btl.lighting,
                                    bbl.lighting,
                                    tl.lighting,
                                    bl.lighting,
                                )
                            } else if y > 127.0 {
                                lerp(x, tl.lighting, bl.lighting)
                            } else {
                                bilerp(
                                    x,
                                    127.0 - y,
                                    tl.lighting,
                                    bl.lighting,
                                    tr.lighting,
                                    br.lighting,
                                )
                            }
                        }
                        Orientation::W => {
                            let (tl, tr, bl, br) = get_vertices(pos + 1, &self.vertices);
                            let (_, btr, _, bbr) = get_vertices(
                                pos + 1 - self.map_size.0 * self.map_size.1,
                                &self.vertices,
                            );

                            if y > 256.0 {
                                bilerp(
                                    x,
                                    384.0 - y,
                                    bbr.lighting,
                                    btr.lighting,
                                    br.lighting,
                                    tr.lighting,
                                )
                            } else if y > 127.0 {
                                lerp(x, br.lighting, tr.lighting)
                            } else {
                                bilerp(
                                    x,
                                    127.0 - y,
                                    br.lighting,
                                    tr.lighting,
                                    bl.lighting,
                                    tl.lighting,
                                )
                            }
                        }
                    }
                }
            } else {
                let location = match orientation {
                    Orientation::N => pos - self.map_size.0,
                    Orientation::S => pos + self.map_size.0,
                    Orientation::E => pos - 1,
                    Orientation::W => pos + 1,
                };

                127.0 * 127.0 * self.lighting[location]
            }
        } else {
            1.0
        }
    }
}

pub fn calculate_lighting(
    torches_pos: Vec<usize>,
    map: &[bool],
    map_size: (usize, usize),
) -> Vec<u8> {
    let map = [map, map].concat();
    let mut lightq = VecDeque::new();
    let mut light_int: Vec<u8> = vec![0; map_size.0 * map_size.1 * 2];
    torches_pos.into_iter().for_each(|light_pos| {
        lightq.push_front(light_pos);
        light_int[light_pos] = 15;
    });

    while !lightq.is_empty() {
        let node = *lightq.front().expect("Queue is empty");
        lightq.pop_front();
        let x = node % (map_size.0);
        let y = (node / map_size.0) % map_size.1;
        let z = node / (map_size.0 * map_size.1);
        let light_node = light_int[node];

        //negative x neighbor
        if x > 0 {
            let neighbor = x - 1 + y * map_size.0 + z * map_size.0 * map_size.1;
            if !map[neighbor]
                && light_int[neighbor] <= light_node - 2
                && light_node != 0
                && light_node != 1
            {
                light_int[neighbor] = light_node - 1;
                lightq.push_back(neighbor);
            }
        }

        //Positive x neighbor
        if x < map_size.0 - 1 {
            let neighbor = x + 1 + y * map_size.0 + z * map_size.0 * map_size.1;
            if !map[neighbor] && light_int[neighbor] <= light_node - 2 && light_node != 1 {
                light_int[neighbor] = light_node - 1;
                lightq.push_back(neighbor);
            }
        }

        //negative y neighbor
        if y > 0 {
            let neighbor = x + (y - 1) * map_size.0 + z * map_size.0 * map_size.1;
            if !map[neighbor] && light_int[neighbor] <= light_node - 2 && light_node != 1 {
                light_int[neighbor] = light_node - 1;
                lightq.push_back(neighbor);
            }
        }

        //Positive y neighbor
        if y < map_size.1 - 1 {
            let neighbor = x + (y + 1) * map_size.0 + z * map_size.0 * map_size.1;
            if !map[neighbor] && light_int[neighbor] <= light_node - 2 && light_node != 1 {
                light_int[neighbor] = light_node - 1;
                lightq.push_back(neighbor);
            }
        }
        // up neighbor
        if z > 0 {
            let neighbor = x + y * map_size.0 + (z - 1) * map_size.0 * map_size.1;
            if !map[neighbor] && light_int[neighbor] <= light_node - 2 && light_node != 1 {
                light_int[neighbor] = light_node - 1;
                lightq.push_back(neighbor);
            }
        }
        // down neighbor
        if z < 1 {
            let neighbor = x + y * map_size.0 + (z + 1) * map_size.0 * map_size.1;
            if !map[neighbor] && light_int[neighbor] <= light_node - 2 && light_node != 1 {
                light_int[neighbor] = light_node - 1;
                lightq.push_back(neighbor);
            }
        }
    }
    light_int
}
#[inline(always)]
unsafe fn bilerp(x: f32, y: f32, v1: f32, v2: f32, v3: f32, v4: f32) -> f32 {
    let x2 = 127.0 - x;
    let y2 = 127.0 - y;

    let v_a1 = _mm_set_ps(v4, v3, v2, v1); //[bottom left,bottom right, top left, top right]
    let v_a2 = _mm_set_ps(x, y, x, x2);
    let v_a3 = _mm_set_ps(y, x2, y2, y2);

    let m = _mm_mul_ps(v_a1, _mm_mul_ps(v_a2, v_a3));
    let t1 = _mm_hadd_ps(m, m);
    _mm_cvtss_f32(_mm_hadd_ps(t1, t1))
}

#[inline(always)]
unsafe fn lerp(x: f32, l: f32, r: f32) -> f32 {
    let x2 = 127.0 - x;
    let v_a1 = _mm_set_ps(0.0, 0.0, r, l);
    let v_a2 = _mm_set_ps(0.0, 0.0, x, x2);
    let t1 = _mm_mul_ps(v_a1, v_a2);
    _mm_cvtss_f32(_mm_hadd_ps(t1, t1)) * 127.0

    //(l * x2 + x * r) * 127.0
}

#[inline(always)]
fn get_vertices(pos: usize, vertices: &[Vertex]) -> (Vertex, Vertex, Vertex, Vertex) {
    let tl = vertices[pos * 4];
    let tr = vertices[pos * 4 + 1];
    let bl = vertices[pos * 4 + 2];
    let br = vertices[pos * 4 + 3];
    (tl, tr, bl, br)
}
#[derive(Copy, Clone)]
pub struct Vertex {
    lighting: f32,
}

impl Vertex {
    pub fn new(pos: [usize; 3], map_size: (usize, usize), lighting: &[f32]) -> Self {
        let x = pos[0];
        let y = pos[1];
        let z = pos[2];
        let neighbor1 = {
            if x > 0 && y < map_size.1 {
                lighting[x - 1 + map_size.0 * y + z * map_size.0 * map_size.1]
            } else {
                0.0
            }
        };
        let neighbor2 = {
            if x < map_size.0 && y < map_size.1 {
                lighting[x + map_size.0 * y + z * map_size.0 * map_size.1]
            } else {
                0.0
            }
        };
        let neighbor3 = {
            if y > 0 && x < map_size.0 {
                lighting[x + map_size.0 * (y - 1) + z * map_size.0 * map_size.1]
            } else {
                0.0
            }
        };
        let neighbor4 = {
            if y > 0 && x > 0 {
                lighting[x - 1 + map_size.0 * (y - 1) + z * map_size.0 * map_size.1]
            } else {
                0.0
            }
        };

        let lighting = num::clamp(
            (neighbor1 + neighbor2 + neighbor3 + neighbor4) / 4.0,
            0.0,
            1.0,
        );

        Self { lighting }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self { lighting: 0.0 }
    }
}

pub struct Torch {
    pub intensity: f32,
    flicker_r_low: f32,
    flicker_r_high: f32,
    flicker_r_hold: f32,
    flicker_r_pause: f32,
    state: u8,
    timer: f32,
}

impl Torch {
    pub fn update_intensity(&mut self, timer: f32) {
        match self.state {
            0 => {
                // low
                if self.intensity > self.flicker_r_high {
                    self.state = 1;
                    self.flicker_r_hold =
                        rand::thread_rng().gen_range(80.0 / 1000.0..800.0 / 1000.0);
                    self.timer = timer;
                } else {
                    self.intensity += 0.005;
                }
            }
            1 => {
                // hold between low and high
                if timer - self.timer > self.flicker_r_hold {
                    self.state = 2;
                    self.flicker_r_low = rand::thread_rng().gen_range(220.0 / 256.0..240.0 / 256.0);
                }
            }
            2 => {
                //high
                if self.intensity < self.flicker_r_low {
                    self.state = 3;
                    self.flicker_r_pause = rand::thread_rng().gen_range(0.04..0.14);
                    self.timer = timer;
                } else {
                    self.intensity -= 0.008;
                }
            }
            _ => {
                //pause
                if timer - self.timer > self.flicker_r_pause {
                    self.state = 0;
                    self.flicker_r_high = rand::thread_rng().gen_range(230.0 / 256.0..1.0);
                }
            }
        }
    }
}

impl Default for Torch {
    fn default() -> Self {
        let flicker_low_min = 200.0 / 256.0;
        let flicker_low_max = 240.0 / 256.0;
        let flicker_high_min = 230.0 / 256.0;
        let flicker_high_max = 1.0;

        Self {
            flicker_r_low: rand::thread_rng().gen_range(flicker_low_min..flicker_low_max),
            flicker_r_high: rand::thread_rng().gen_range(flicker_high_min..flicker_high_max),
            flicker_r_hold: rand::thread_rng().gen_range(40.0..80.0),
            flicker_r_pause: rand::thread_rng().gen_range(100.0..200.0),
            state: 0,
            intensity: rand::thread_rng().gen_range(flicker_low_min..flicker_low_max),
            timer: 0.0,
        }
    }
}
