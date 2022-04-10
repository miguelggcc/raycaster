use crate::{Orientation};
use std::collections::VecDeque;

pub struct Lighting {
    vertices: Vec<Vertex>,
    lighting: Vec<f32>,
    map_size: (usize, usize),
    pub switch: bool,
    pub smooth_switch: bool
}

impl Lighting {
    pub fn new(torches_pos: Vec<usize>, map: &Vec<bool>, map_size: (usize, usize)) -> Self {
        let lighting = lighting(torches_pos, map, map_size);
        let mut all_vertices = vec![];
        for j in 0..map_size.1 + 1 {
            for i in 0..map_size.0 + 1 {
                all_vertices.push(Vertex::new([i, j], map_size, &lighting));
            }
        }

        let mut vertices = vec![Vertex::default(); map_size.0 * map_size.1 * 4];

        vertices.chunks_mut(4).enumerate().for_each(|(pos, chunk)| {
            let tl = all_vertices[pos + (pos / map_size.0)];
            let tr = all_vertices[pos + 1 + (pos / map_size.0)];
            let bl = all_vertices[pos + map_size.0 + 1 + (pos / map_size.0)];
            let br = all_vertices[pos + map_size.0 + 2 + (pos / map_size.0)];
            chunk.copy_from_slice(&[tl, tr, bl, br]);
        });

        Self {
            vertices,
            lighting,
            map_size,
            switch: false,
            smooth_switch: true,
        }
    }
    pub fn get_lighting_floor(&self, x: f32, y: f32, pos: usize) -> f32 {
        let (tl, tr, bl, br) = get_vertices(pos, &self.vertices);
        if !self.switch {
            if!self.smooth_switch{
            bilerp(
                x,
                1.0 - y,
                tl.lighting,
                tr.lighting,
                bl.lighting,
                br.lighting,
            )
        }else{
            self.lighting[pos]
        }
        } else {
            1.0
        }
    }

    pub fn get_lighting_wall(&self, x: f32, y: f32, pos: usize, orientation: &Orientation) -> f32 {
        if !self.switch {
            if !self.smooth_switch{
            match orientation {
                Orientation::N => {
                    let location = pos - self.map_size.0;
                    let (tl, tr, bl, br) = get_vertices(location, &self.vertices);
                    if y > 2.0 {
                        bilerp(
                            1.0 - x,
                            3.0 - y ,
                            tl.lighting,
                            tr.lighting,
                            bl.lighting,
                            br.lighting,
                        )
                    }else if y > 1.0 {
                        bilerp(
                            1.0 - x,
                            2.0 - y,
                            tl.lighting,
                            tr.lighting,
                            tl.lighting,
                            tr.lighting,
                        )
                    } else {
                        bilerp(
                            1.0 - x,
                            1.0 - y,
                            bl.lighting,
                            br.lighting,
                            tl.lighting,
                            tr.lighting,
                        )
                    }
                }
                Orientation::S => {
                    let location = pos + self.map_size.0;
                    let (tl, tr, bl, br) = get_vertices(location, &self.vertices);
                    if y >2.0 {
                        bilerp(
                            x,
                            3.0 - y,
                            bl.lighting,
                            br.lighting,
                            tl.lighting,
                            tr.lighting,
                        )
                    } else if y>1.0{
                        bilerp(
                            x,
                            2.0 - y,
                            bl.lighting,
                            br.lighting,
                            bl.lighting,
                            br.lighting,
                        )
                    } else {
                        bilerp(
                            x,
                            1.0 - y,
                            tl.lighting,
                            tr.lighting,
                            bl.lighting,
                            br.lighting,
                        )
                    }
                }
                Orientation::E => {
                    let location = pos - 1;
                    let (tl, tr, bl, br) = get_vertices(location, &self.vertices);
                    if y >2.0 {
                        bilerp(
                            x,
                            3.0 - y,
                            tl.lighting,
                            bl.lighting,
                            tr.lighting,
                            br.lighting,
                        )
                    } else if y>1.0{
                        bilerp(
                            x,
                            2.0 - y,
                            tl.lighting,
                            bl.lighting,
                            tl.lighting,
                            bl.lighting,
                        )
                    }else {
                        bilerp(
                            x,
                            1.0 - y,
                            tr.lighting,
                            br.lighting,
                            tl.lighting,
                            bl.lighting,
                        )
                    }
                }
                Orientation::W => {
                    let location = pos + 1;
                    let (tl, tr, bl, br) = get_vertices(location, &self.vertices);
                    if y > 2.0 {
                        bilerp(
                            x,
                            3.0 - y,
                            br.lighting,
                            tr.lighting,
                            bl.lighting,
                            tl.lighting,
                        )
                    }else if y > 1.0 {
                        bilerp(
                            x,
                            2.0 - y,
                            br.lighting,
                            tr.lighting,
                            br.lighting,
                            tr.lighting,
                        )
                    }  else {
                        bilerp(
                            x,
                            1.0 - y,
                            bl.lighting,
                            tl.lighting,
                            br.lighting,
                            tr.lighting,
                        )
                    }
                }
            }
        }else{
            let location;
            match orientation {
                Orientation::N =>  location = pos - self.map_size.0,
                Orientation::S => location = pos + self.map_size.0,
                Orientation::E =>location = pos-1,
                Orientation::W => location = pos+1,
            }

            self.lighting[location]
        }
        } else {
            1.0
        }
    }
}

pub fn lighting(torches_pos: Vec<usize>, map: &Vec<bool>, map_size: (usize, usize)) -> Vec<f32> {
    let mut lightq = VecDeque::new();
    let mut light_int: Vec<u8> = vec![0; map_size.0 * map_size.1];
    torches_pos.into_iter().for_each(|light_pos| {
        lightq.push_front(light_pos);
        light_int[light_pos] = 15;
    });

    while !lightq.is_empty() {
        let node = *lightq.front().expect("Queue is empty");
        lightq.pop_front();
        let x = node % map_size.0;
        let y = node / map_size.0;
        let light_node = light_int[node];

        //negative x neighbor
        if x > 0 {
            let neighbor = x - 1 + y * map_size.0;
            if !map[neighbor] && light_int[neighbor] <= light_node - 2 && light_node != 0 {
                if light_node != 1 {
                    light_int[neighbor] = light_node - 1;
                    lightq.push_back(neighbor);
                }
            }
        }

        //Positive x neighbor
        if x < map_size.0 - 1 {
            let neighbor = x + 1 + y * map_size.0;
            if !map[neighbor] && light_int[neighbor] <= light_node - 2 {
                if light_node != 1 {
                    light_int[neighbor] = light_node - 1;
                    lightq.push_back(neighbor);
                }
            }
        }

        //negative y neighbor
        if y > 0 {
            let neighbor = x + (y - 1) * map_size.0;
            if !map[neighbor] && light_int[neighbor] <= light_node - 2 {
                if light_node != 1 {
                    light_int[neighbor] = light_node - 1;
                    lightq.push_back(neighbor);
                }
            }
        }

        //Positive y neighbor
        if y < map_size.1 - 1 {
            let neighbor = x + (y + 1) * map_size.0;
            if !map[neighbor] && light_int[neighbor] <= light_node - 2 {
                if light_node != 1 {
                    light_int[neighbor] = light_node - 1;
                    lightq.push_back(neighbor);
                }
            }
        }
    }
    let mut light = Vec::new();
    for i in light_int {
        light.push(0.8f32.powf((15 - i) as f32));
    }
    light
}

fn bilerp(x: f32, y: f32, tl: f32, tr: f32, bl: f32, br: f32) -> f32 {
    bl * (1.0 - x) * (1.0 - y) + br * x * (1.0 - y) + tl * y * (1.0 - x) + tr * x * y
}

fn get_vertices(pos: usize, vertices: &Vec<Vertex>) -> (Vertex, Vertex, Vertex, Vertex) {
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
    pub fn new(pos: [usize; 2], map_size: (usize, usize), lighting: &Vec<f32>) -> Self {
        let x = pos[0];
        let y = pos[1];
        let neighbor1 = {
            if x > 0 && y < map_size.1 {
                lighting[x - 1 + map_size.0 * y]
            } else {
                0.0
            }
        };
        let neighbor2 = {
            if x < map_size.0 && y < map_size.1 {
                lighting[x + map_size.0 * y]
            } else {
                0.0
            }
        };
        let neighbor3 = {
            if y > 0 && x < map_size.0 {
                lighting[x + map_size.0 * (y - 1)]
            } else {
                0.0
            }
        };
        let neighbor4 = {
            if y > 0 && x > 0 {
                lighting[x - 1 + map_size.0 * (y - 1)]
            } else {
                0.0
            }
        };
        if pos[0]==2usize && pos[1]==2usize{
            dbg!(neighbor1, neighbor2, neighbor3, neighbor4);
        }
        let lighting = num::clamp(
            (neighbor1 + neighbor2 + neighbor3 + neighbor4) / 4.0,
            0.0,
            1.0,
        );
        


        Self { lighting}
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self { lighting: 0.0 }
    }
}
